#!/usr/bin/env python3
"""Validate the evidence-scaled maturity dossier and bounded status claims."""
from __future__ import annotations

from datetime import date, datetime
import json
from pathlib import Path
import re
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
STATUS_SNAPSHOT = ROOT / "verification/receipts/project-status-snapshot.json"
README = ROOT / "README.md"
PROJECT_STATUS = ROOT / "PROJECT_STATUS.md"
EXPECTED = {
    "contracts", "compiler", "determinism", "providers", "methodology",
    "security", "interfaces", "migration", "usability", "operations",
    "github_control_plane", "downstream_compatibility", "access_and_tenancy",
    "backup_restore_incidents", "sdk_and_adoption", "pilots", "registries",
}
READY_STATES = {"passed", "externally_validated", "publicly_accepted"}
DECISIONS = {"not_ready", "ready"}
READY_EVIDENCE_FIELDS = {
    "accountable_reviewer", "exact_git_commit", "release_candidate", "sbom",
    "attestations", "downstream_canaries", "pilot_exits", "rollback_plan",
    "support_plan",
}
STATUS_SCHEMA = "org.searchright.project-status-snapshot.v1"
SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")
OBSERVATION_CONCLUSIONS = {
    "success", "failure", "neutral", "cancelled", "skipped", "timed_out",
    "action_required", "stale", "startup_failure",
}
REQUIRED_DYNAMIC_ASSERTIONS = {
    "local_main_equals_origin_main",
    "working_tree_clean",
    "no_open_pull_requests",
    "no_delivery_branches",
}
STALE_STATUS_PHRASES = {
    "not produced Rust compilation, a committed `Cargo.lock`":
        "README denies compiler or committed lockfile evidence",
    "Local `main` and `origin/main` are identical":
        "PROJECT_STATUS asserts volatile branch equality",
    "The working tree is clean":
        "PROJECT_STATUS asserts volatile working-tree state",
    "there are no open pull requests":
        "PROJECT_STATUS asserts volatile pull-request state",
    "no pull request is open":
        "PROJECT_STATUS asserts volatile pull-request state",
}


def _non_negative_integer(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _valid_iso_datetime(value: Any) -> bool:
    if not isinstance(value, str) or not value:
        return False
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return False
    return True


def validate(data: Any, *, check_documents: bool = True) -> list[str]:
    """Return deterministic maturity-dossier errors without promoting evidence."""
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["dossier must be a JSON object"]
    if data.get("schema_version") != "org.searchright.maturity-dossier.v1":
        errors.append("unsupported maturity dossier schema_version")
    if data.get("decision") not in DECISIONS:
        errors.append("decision must be not_ready or ready")
    domains = data.get("domains")
    if not isinstance(domains, list):
        errors.append("domains must be an array")
        domains = []
    valid_domains = [row for row in domains if isinstance(row, dict)]
    if len(valid_domains) != len(domains):
        errors.append("every maturity domain must be an object")
    names = [row.get("domain") for row in valid_domains]
    if set(names) != EXPECTED or len(names) != len(set(names)):
        errors.append(f"maturity domains differ: {sorted(set(names) ^ EXPECTED)}")

    blockers: list[str] = []
    for domain in valid_domains:
        name = domain.get("domain")
        if not isinstance(domain.get("state"), str) or not domain.get("state"):
            errors.append(f"domain {name!r} requires a non-empty state")
        if type(domain.get("critical_blocker")) is not bool:
            errors.append(f"domain {name!r} requires a boolean critical_blocker")
        elif domain["critical_blocker"]:
            blockers.append(str(name))

    decision = data.get("decision")
    if decision == "not_ready" and not blockers:
        errors.append("not_ready decision must retain at least one critical blocker")
    if blockers and decision != "not_ready":
        errors.append("critical blockers require not_ready decision")
    if decision == "ready":
        non_ready = sorted(
            str(row.get("domain"))
            for row in valid_domains
            if row.get("state") not in READY_STATES
        )
        if non_ready:
            errors.append(f"ready decision has non-ready domains: {non_ready}")
        evidence = data.get("release_decision_evidence")
        if not isinstance(evidence, dict):
            errors.append("ready decision requires release_decision_evidence")
        else:
            missing = sorted(
                field for field in READY_EVIDENCE_FIELDS if not evidence.get(field)
            )
            if missing:
                errors.append(f"ready decision evidence is incomplete: {missing}")
            if evidence.get("approved") is not True:
                errors.append("ready decision evidence requires explicit approval")

    exceptions = data.get("release_risk_exceptions", [])
    if not isinstance(exceptions, list):
        errors.append("release_risk_exceptions must be an array")
    else:
        for index, exception in enumerate(exceptions):
            if not isinstance(exception, dict):
                errors.append(f"release risk exception {index} must be an object")
                continue
            required = {"id", "domain", "risk", "disposition", "approved_by"}
            missing = sorted(field for field in required if not exception.get(field))
            if missing:
                errors.append(f"release risk exception {index} is incomplete: {missing}")
            if exception.get("domain") not in EXPECTED:
                errors.append(f"release risk exception {index} has unknown domain")
            if exception.get("disposition") not in {"accepted", "rejected"}:
                errors.append(f"release risk exception {index} has invalid disposition")
    if decision == "ready" and isinstance(exceptions, list):
        evidence = data.get("release_decision_evidence")
        recorded = (
            evidence.get("release_risk_exceptions", [])
            if isinstance(evidence, dict)
            else []
        )
        expected_ids = sorted(
            str(row.get("id"))
            for row in exceptions
            if isinstance(row, dict) and row.get("id")
        )
        if (
            not isinstance(recorded, list)
            or sorted(str(item) for item in recorded) != expected_ids
        ):
            errors.append("ready decision must enumerate every release risk exception")

    if check_documents:
        for path in (
            "docs/maturity/1.0-gate.md",
            "docs/maturity/gap-register.md",
            "docs/maturity/release-decision.md",
        ):
            if not (ROOT / path).is_file():
                errors.append(f"missing {path}")
    return errors


def validate_status_snapshot(
    data: Any,
    *,
    dossier_decision: Any,
    readme: str,
    project_status: str,
    cargo_lock_exists: bool,
) -> list[str]:
    """Validate exact-revision status claims and their human-readable surfaces."""
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["project status snapshot must be a JSON object"]
    if data.get("schema_version") != STATUS_SCHEMA:
        errors.append("unsupported project status snapshot schema_version")

    status_date = data.get("status_date")
    if not isinstance(status_date, str):
        errors.append("project status snapshot requires an ISO status_date")
    else:
        try:
            date.fromisoformat(status_date)
        except ValueError:
            errors.append("project status snapshot requires an ISO status_date")

    observed = data.get("observed_main_revision")
    admitted = data.get("last_fully_admitted_revision")
    if not isinstance(observed, str) or SHA_PATTERN.fullmatch(observed) is None:
        errors.append("observed_main_revision must be a lowercase 40-character SHA")
    if not isinstance(admitted, str) or SHA_PATTERN.fullmatch(admitted) is None:
        errors.append(
            "last_fully_admitted_revision must be a lowercase 40-character SHA"
        )
    if not isinstance(data.get("observed_default_branch"), str) or not data.get(
        "observed_default_branch"
    ):
        errors.append("observed_default_branch must be non-empty")
    if not isinstance(data.get("evidence_ceiling"), str) or not data.get(
        "evidence_ceiling"
    ):
        errors.append("evidence_ceiling must be non-empty")
    if data.get("maturity_decision") not in DECISIONS:
        errors.append("snapshot maturity_decision must be not_ready or ready")
    if data.get("maturity_decision") != dossier_decision:
        errors.append("snapshot maturity_decision differs from maturity dossier")

    counts = data.get("source_bound_counts")
    required_counts = {
        "tracks",
        "active_tracks",
        "archived_tracks",
        "total_tasks",
        "completed_tasks",
        "open_evidence_tasks",
        "requirements_checked",
    }
    if not isinstance(counts, dict):
        errors.append("source_bound_counts must be an object")
    else:
        for field in sorted(required_counts):
            if not _non_negative_integer(counts.get(field)):
                errors.append(f"source_bound_counts.{field} must be non-negative")
        if all(_non_negative_integer(counts.get(field)) for field in required_counts):
            if counts["active_tracks"] + counts["archived_tracks"] != counts["tracks"]:
                errors.append("active and archived track counts do not reconcile")
            if (
                counts["completed_tasks"] + counts["open_evidence_tasks"]
                != counts["total_tasks"]
            ):
                errors.append("completed and open task counts do not reconcile")

    observations = data.get("exact_head_observations")
    if not isinstance(observations, list) or not observations:
        errors.append("exact_head_observations must contain at least one observation")
        observations = []
    for index, observation in enumerate(observations):
        prefix = f"exact_head_observations[{index}]"
        if not isinstance(observation, dict):
            errors.append(f"{prefix} must be an object")
            continue
        if observation.get("kind") != "github_actions_run":
            errors.append(f"{prefix}.kind must be github_actions_run")
        if not _non_negative_integer(observation.get("run_id")) or not observation.get(
            "run_id"
        ):
            errors.append(f"{prefix}.run_id must be a positive integer")
        if observation.get("head_sha") != observed:
            errors.append(f"{prefix}.head_sha differs from observed_main_revision")
        if observation.get("status") != "completed":
            errors.append(f"{prefix}.status must be completed")
        if observation.get("conclusion") not in OBSERVATION_CONCLUSIONS:
            errors.append(f"{prefix}.conclusion is unsupported")
        if observation.get("full_admission_matrix") is not False:
            errors.append(f"{prefix} must not claim a full admission matrix")
        if not isinstance(observation.get("claim_scope"), str) or not observation.get(
            "claim_scope"
        ):
            errors.append(f"{prefix}.claim_scope must be non-empty")
        if not isinstance(observation.get("claim_boundary"), str) or not observation.get(
            "claim_boundary"
        ):
            errors.append(f"{prefix}.claim_boundary must be non-empty")
        for field in ("started_at", "completed_at"):
            if not _valid_iso_datetime(observation.get(field)):
                errors.append(f"{prefix}.{field} must be an ISO timestamp")

    historical = data.get("historical_admission")
    if not isinstance(historical, dict):
        errors.append("historical_admission must be an object")
    else:
        if historical.get("revision") != admitted:
            errors.append(
                "historical_admission.revision differs from last fully admitted revision"
            )
        for field in ("checks_passed", "failures", "pending"):
            if not _non_negative_integer(historical.get(field)):
                errors.append(f"historical_admission.{field} must be non-negative")
        if historical.get("exact_revision_only") is not True:
            errors.append("historical admission must remain exact-revision only")
        if observed != admitted and historical.get("transfers_to_observed_head") is not False:
            errors.append("historical admission must not transfer to a later observed head")
        if not isinstance(historical.get("claim_boundary"), str) or not historical.get(
            "claim_boundary"
        ):
            errors.append("historical_admission.claim_boundary must be non-empty")

    remote_policy = data.get("remote_state_policy")
    if not isinstance(remote_policy, dict):
        errors.append("remote_state_policy must be an object")
    else:
        if remote_policy.get("static_remote_state_claims_allowed") is not False:
            errors.append("static remote-state claims must remain disabled")
        prohibited = remote_policy.get("dynamic_assertions_prohibited")
        if not isinstance(prohibited, list) or not REQUIRED_DYNAMIC_ASSERTIONS.issubset(
            {str(value) for value in prohibited}
        ):
            errors.append("remote-state policy omits required prohibited assertions")
        if not isinstance(remote_policy.get("reason"), str) or not remote_policy.get(
            "reason"
        ):
            errors.append("remote_state_policy.reason must be non-empty")

    limitations = data.get("limitations")
    if not isinstance(limitations, list) or not limitations or not all(
        isinstance(value, str) and value for value in limitations
    ):
        errors.append("project status snapshot requires non-empty limitations")
    elif not any("No complete exact-head" in value for value in limitations):
        errors.append("snapshot limitations must deny complete exact-head admission")

    snapshot_reference = "verification/receipts/project-status-snapshot.json"
    for name, document in (("README", readme), ("PROJECT_STATUS", project_status)):
        if snapshot_reference not in document:
            errors.append(f"{name} does not reference the project status snapshot")
        for revision_name, revision in (
            ("observed main", observed),
            ("last fully admitted", admitted),
        ):
            if isinstance(revision, str) and revision not in document:
                errors.append(f"{name} omits the {revision_name} revision")

    if "No complete exact-head admission matrix is claimed." not in project_status:
        errors.append("PROJECT_STATUS omits the exact-head admission denial")
    for phrase, message in STALE_STATUS_PHRASES.items():
        if phrase in readme or phrase in project_status:
            errors.append(message)
    if cargo_lock_exists and re.search(
        r"(?:no|without|absent).{0,40}(?:committed\s+)?`?Cargo\.lock`?",
        readme,
        flags=re.IGNORECASE,
    ):
        errors.append("README contradicts the committed Cargo.lock")
    return errors


def main() -> int:
    data = json.loads(
        (ROOT / "conductor/maturity-dossier.json").read_text(encoding="utf-8")
    )
    errors = validate(data)
    snapshot: Any
    try:
        snapshot = json.loads(STATUS_SNAPSHOT.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        snapshot = {}
        errors.append(f"project status snapshot is unreadable: {type(error).__name__}")
    else:
        errors.extend(
            validate_status_snapshot(
                snapshot,
                dossier_decision=data.get("decision") if isinstance(data, dict) else None,
                readme=README.read_text(encoding="utf-8"),
                project_status=PROJECT_STATUS.read_text(encoding="utf-8"),
                cargo_lock_exists=(ROOT / "Cargo.lock").is_file(),
            )
        )

    domains = data.get("domains", []) if isinstance(data, dict) else []
    blockers = sorted(
        str(row.get("domain"))
        for row in domains
        if isinstance(row, dict) and row.get("critical_blocker") is True
    )
    observations = (
        snapshot.get("exact_head_observations", [])
        if isinstance(snapshot, dict)
        else []
    )
    receipt = {
        "schema_version": "org.searchright.maturity-dossier-receipt.v1",
        "status": "failed" if errors else "passed",
        "decision": data.get("decision") if isinstance(data, dict) else None,
        "domains": len(domains),
        "critical_blockers": blockers,
        "release_risk_exceptions": len(data.get("release_risk_exceptions", []))
        if isinstance(data, dict)
        and isinstance(data.get("release_risk_exceptions", []), list)
        else None,
        "status_snapshot": str(STATUS_SNAPSHOT.relative_to(ROOT)),
        "observed_main_revision": snapshot.get("observed_main_revision")
        if isinstance(snapshot, dict)
        else None,
        "last_fully_admitted_revision": snapshot.get(
            "last_fully_admitted_revision"
        )
        if isinstance(snapshot, dict)
        else None,
        "exact_head_observations": len(observations)
        if isinstance(observations, list)
        else None,
        "errors": errors,
        "limitations": [
            "Static dossier and claim-surface consistency only; this validator does not query GitHub or generate compiler, live, human or external evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
