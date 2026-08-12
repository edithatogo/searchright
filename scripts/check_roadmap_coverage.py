#!/usr/bin/env python3
"""Validate roadmap-to-track coverage and evidence without overstating runtime proof."""

from __future__ import annotations

import json
import re
import sys
from collections.abc import Mapping
from datetime import date
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
COVERAGE_PATH = ROOT / "conductor" / "roadmap-coverage.json"
TRACKS_ROOT = ROOT / "conductor" / "tracks"
ALLOWED_STATUSES = {
    "contracted",
    "scaffolded",
    "partially_implemented",
    "source_implemented",
    "source_implemented_unverified",
    "integration_prepared",
    "release_prepared",
    "submission_prepared",
    "external_evidence_required",
}
ALLOWED_EVIDENCE = {
    "contracted",
    "source_verified",
    "compiler_verified",
    "fixture_proven",
    "live_proven",
    "externally_validated",
    "published",
}
FINAL_EVIDENCE = {"compiler_verified", "fixture_proven", "live_proven", "externally_validated", "published"}
ALLOWED_IMPLEMENTATION_STATES = {
    "contracted", "scaffolded", "partially_implemented", "source_implemented", "external_evidence_required"
}


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def error(errors: list[str], message: str) -> None:
    errors.append(message)


def validate_archived_track(
    track_id: str,
    entry: Mapping[str, Any],
    plan: str,
    traceability: Mapping[str, Any],
) -> list[str]:
    """Return lifecycle violations without relocating canonical track state."""
    if entry.get("lifecycle") != "archived":
        return []
    violations: list[str] = []
    try:
        date.fromisoformat(str(entry.get("archived_on", "")))
    except ValueError:
        violations.append(f"track {track_id} archived_on must be an ISO date")
    if entry.get("blockers"):
        violations.append(f"track {track_id} archived lifecycle retains blockers")
    if not entry.get("closeout_completed"):
        violations.append(f"track {track_id} archived lifecycle lacks completed closeout")
    if not entry.get("review_completed"):
        violations.append(f"track {track_id} archived lifecycle lacks completed review")
    if not entry.get("higher_evidence_completed"):
        violations.append(f"track {track_id} archived lifecycle lacks completed higher evidence")
    if not entry.get("completed_higher_evidence_gates"):
        violations.append(f"track {track_id} archived lifecycle lacks completed evidence gates")
    if entry.get("evidence_level") not in FINAL_EVIDENCE:
        violations.append(f"track {track_id} archived lifecycle lacks final evidence")
    if re.search(r"^- \[ \]", plan, flags=re.MULTILINE):
        violations.append(f"track {track_id} archived lifecycle retains unchecked plan tasks")
    assertions = traceability.get("assertions")
    if not isinstance(assertions, list) or not assertions:
        violations.append(f"track {track_id} archived lifecycle lacks assertion traceability")
    else:
        for assertion in assertions:
            if not isinstance(assertion, Mapping):
                violations.append(f"track {track_id} archived lifecycle has invalid assertion")
                continue
            gates = assertion.get("open_gates")
            if not isinstance(gates, list) or gates:
                violations.append(f"track {track_id} archived lifecycle retains open assertion gates")
            receipts = assertion.get("evidence_receipts")
            if not isinstance(receipts, list) or not receipts:
                violations.append(f"track {track_id} archived lifecycle lacks assertion receipts")
            elif any(not isinstance(path, str) or not (ROOT / path).is_file() for path in receipts):
                violations.append(f"track {track_id} archived lifecycle refers to missing assertion receipts")
    return violations


def main() -> int:
    errors: list[str] = []
    if not COVERAGE_PATH.is_file():
        print("missing conductor/roadmap-coverage.json", file=sys.stderr)
        return 1

    coverage = load_json(COVERAGE_PATH)
    if not isinstance(coverage, Mapping):
        print("roadmap coverage must be an object", file=sys.stderr)
        return 1
    entries = coverage.get("tracks")
    if not isinstance(entries, list) or not entries:
        print("roadmap coverage must contain tracks", file=sys.stderr)
        return 1

    directories = sorted(path for path in TRACKS_ROOT.glob("[0-9][0-9]-*") if path.is_dir())
    expected_ids = [f"{number:02d}" for number in range(len(directories))]
    actual_ids = [path.name[:2] for path in directories]
    if actual_ids != expected_ids:
        error(errors, f"track IDs are not contiguous: {actual_ids}")

    entries_by_id: dict[str, Mapping[str, Any]] = {}
    horizons: set[str] = set()
    requirements_owned: set[str] = set()
    checked_tasks = 0
    unchecked_tasks = 0

    for index, value in enumerate(entries):
        if not isinstance(value, Mapping):
            error(errors, f"coverage entry {index} is not an object")
            continue
        track_id = value.get("track_id")
        if not isinstance(track_id, str) or not re.fullmatch(r"\d{2}", track_id):
            error(errors, f"coverage entry {index} has invalid track_id")
            continue
        if track_id in entries_by_id:
            error(errors, f"duplicate coverage track {track_id}")
        entries_by_id[track_id] = value
        horizon = value.get("horizon")
        if isinstance(horizon, str) and horizon:
            horizons.add(horizon)
        else:
            error(errors, f"track {track_id} has no horizon")
        for requirement in value.get("requirements", []):
            if isinstance(requirement, str):
                requirements_owned.add(requirement)

    if set(entries_by_id) != set(actual_ids):
        error(
            errors,
            f"coverage/track parity mismatch: missing={sorted(set(actual_ids)-set(entries_by_id))}, "
            f"extra={sorted(set(entries_by_id)-set(actual_ids))}",
        )

    for directory in directories:
        track_id = directory.name[:2]
        entry = entries_by_id.get(track_id)
        if entry is None:
            continue
        metadata_path = directory / "metadata.json"
        evidence_path = directory / "evidence.json"
        plan_path = directory / "plan.md"
        for path in (metadata_path, evidence_path, plan_path, directory / "spec.md"):
            if not path.is_file():
                error(errors, f"missing {path.relative_to(ROOT)}")
        if not all(path.is_file() for path in (metadata_path, evidence_path, plan_path)):
            continue

        metadata = load_json(metadata_path)
        evidence = load_json(evidence_path)
        status = entry.get("status")
        evidence_level = entry.get("evidence_level")
        implementation_state = entry.get("implementation_state")
        lifecycle = entry.get("lifecycle", "active")
        if lifecycle not in {"active", "archived"}:
            error(errors, f"track {track_id} has invalid lifecycle {lifecycle!r}")
        if status not in ALLOWED_STATUSES:
            error(errors, f"track {track_id} has invalid status {status!r}")
        if evidence_level not in ALLOWED_EVIDENCE:
            error(errors, f"track {track_id} has invalid evidence level {evidence_level!r}")
        if implementation_state not in ALLOWED_IMPLEMENTATION_STATES:
            error(errors, f"track {track_id} has invalid implementation state {implementation_state!r}")
        if metadata.get("track_id") != track_id or evidence.get("track_id") != track_id:
            error(errors, f"track {track_id} identity mismatch")
        if metadata.get("status") != status or evidence.get("status") != status:
            error(errors, f"track {track_id} status differs across coverage/metadata/evidence")
        if metadata.get("implementation_state") != implementation_state or evidence.get("implementation_state") != implementation_state:
            error(errors, f"track {track_id} implementation state differs across coverage/metadata/evidence")
        expected_trace = f"conductor/tracks/{track_id}-{entry['slug']}/traceability.json"
        if metadata.get("traceability_path") != expected_trace or evidence.get("traceability") != expected_trace:
            error(errors, f"track {track_id} traceability path differs across metadata/evidence")
        if not (ROOT / expected_trace).is_file():
            error(errors, f"track {track_id} has no assertion traceability ledger")
        if metadata.get("evidence_level") != evidence_level or evidence.get("evidence_level") != evidence_level:
            error(errors, f"track {track_id} evidence differs across coverage/metadata/evidence")
        if metadata.get("slug") != directory.name[3:]:
            error(errors, f"track {track_id} slug mismatch")
        if metadata.get("lifecycle", "active") != lifecycle or evidence.get("lifecycle", "active") != lifecycle:
            error(errors, f"track {track_id} lifecycle differs across coverage/metadata/evidence")

        deliverables = entry.get("deliverables")
        if not isinstance(deliverables, list) or not deliverables:
            error(errors, f"track {track_id} has no deliverables")
        else:
            for relative_path in deliverables:
                if not isinstance(relative_path, str) or not (ROOT / relative_path).exists():
                    error(errors, f"track {track_id} missing deliverable {relative_path!r}")

        checks = entry.get("checks")
        if not isinstance(checks, list) or not checks:
            error(errors, f"track {track_id} has no static/source checks")

        blockers = entry.get("blockers")
        if not isinstance(blockers, list):
            error(errors, f"track {track_id} blockers must be a list")
            blockers = []
        phase_three_count = entry.get("phase_3_task_count")
        if phase_three_count is not None:
            completed_gates = entry.get("completed_higher_evidence_gates", [])
            if (
                not isinstance(phase_three_count, int)
                or phase_three_count < 1
                or phase_three_count < len(blockers)
                or not isinstance(completed_gates, list)
                or phase_three_count < len(completed_gates)
            ):
                error(errors, f"track {track_id} has invalid immutable phase_3_task_count")
        if status == "external_evidence_required" and not blockers:
            error(errors, f"track {track_id} requires external evidence but has no blockers")
        if not blockers and evidence_level not in FINAL_EVIDENCE and status not in {"source_implemented", "source_implemented_unverified", "partially_implemented", "scaffolded", "contracted"}:
            error(errors, f"track {track_id} has no blocker but is not at a final evidence level")

        plan = plan_path.read_text(encoding="utf-8")
        traceability_path = ROOT / expected_trace
        traceability = load_json(traceability_path)
        if isinstance(traceability, Mapping):
            errors.extend(validate_archived_track(track_id, entry, plan, traceability))
        elif entry.get("lifecycle") == "archived":
            error(errors, f"track {track_id} archived lifecycle has invalid traceability")
        checked = len(re.findall(r"^- \[x\]", plan, flags=re.MULTILINE | re.IGNORECASE))
        unchecked = len(re.findall(r"^- \[ \]", plan, flags=re.MULTILINE))
        checked_tasks += checked
        unchecked_tasks += unchecked
        if checked == 0:
            error(errors, f"track {track_id} has no evidenced completed tasks")
        if blockers and unchecked == 0:
            error(errors, f"track {track_id} has blockers but no open task")
        if "## Phase 4: Review and closeout" not in plan:
            error(errors, f"track {track_id} lacks review and closeout")

        source_evidence = evidence.get("source_evidence")
        if not isinstance(source_evidence, list) or sorted(source_evidence) != sorted(deliverables or []):
            error(errors, f"track {track_id} evidence paths differ from coverage deliverables")
        if evidence.get("blockers") != blockers:
            error(errors, f"track {track_id} blockers differ from coverage")

    required_horizons = {"foundation", "mvp", "alpha", "beta", "mature"}
    if not required_horizons.issubset(horizons):
        error(errors, f"roadmap horizons incomplete: {sorted(horizons)}")

    requirement_ids = set(re.findall(r"\|\s*(SR-\d{3})\s*\|", (ROOT / "conductor" / "requirements.md").read_text(encoding="utf-8")))
    missing_requirements = requirement_ids - requirements_owned
    unknown_requirements = requirements_owned - requirement_ids
    if missing_requirements:
        error(errors, f"requirements without a track owner in coverage: {sorted(missing_requirements)}")
    if unknown_requirements:
        error(errors, f"coverage refers to unknown requirements: {sorted(unknown_requirements)}")

    receipt = {
        "schema_version": "org.searchright.roadmap-coverage-receipt.v1",
        "status": "failed" if errors else "passed",
        "tracks_checked": len(directories),
        "horizons": sorted(horizons),
        "requirements_checked": len(requirement_ids),
        "checked_tasks": checked_tasks,
        "open_evidence_tasks": unchecked_tasks,
        "errors": errors,
        "limitations": [
            "Source/evidence validation only; compiler, live-provider and external acceptance gates remain separate.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
