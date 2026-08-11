#!/usr/bin/env python3
"""Validate the evidence-scaled maturity dossier and release decision."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
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


def validate(data: Any, *, check_documents: bool = True) -> list[str]:
    """Return deterministic validation errors without promoting evidence."""
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
                field for field in READY_EVIDENCE_FIELDS
                if not evidence.get(field)
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
        recorded = evidence.get("release_risk_exceptions", []) if isinstance(evidence, dict) else []
        expected_ids = sorted(
            str(row.get("id")) for row in exceptions
            if isinstance(row, dict) and row.get("id")
        )
        if not isinstance(recorded, list) or sorted(str(item) for item in recorded) != expected_ids:
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


def main() -> int:
    data = json.loads(
        (ROOT / "conductor/maturity-dossier.json").read_text(encoding="utf-8")
    )
    errors = validate(data)
    domains = data.get("domains", []) if isinstance(data, dict) else []
    blockers = sorted(
        str(row.get("domain"))
        for row in domains
        if isinstance(row, dict) and row.get("critical_blocker") is True
    )
    receipt = {
        "schema_version": "org.searchright.maturity-dossier-receipt.v1",
        "status": "failed" if errors else "passed",
        "decision": data.get("decision") if isinstance(data, dict) else None,
        "domains": len(domains),
        "critical_blockers": blockers,
        "release_risk_exceptions": len(data.get("release_risk_exceptions", []))
        if isinstance(data, dict) and isinstance(data.get("release_risk_exceptions", []), list)
        else None,
        "errors": errors,
        "limitations": [
            "Static dossier consistency only; it cannot generate missing compiler, live, human or external evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
