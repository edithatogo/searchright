#!/usr/bin/env python3
"""Validate assertion-level implementation/evidence traceability."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COVERAGE = ROOT / "conductor" / "roadmap-coverage.json"
TRACKS = ROOT / "conductor" / "tracks"
STATES = {
    "contracted",
    "scaffolded",
    "partially_implemented",
    "source_implemented",
    "external_evidence_required",
}


def main() -> int:
    errors: list[str] = []
    coverage = json.loads(COVERAGE.read_text(encoding="utf-8"))
    assertions = 0
    individually_mapped = 0
    for entry in coverage["tracks"]:
        track_id = entry["track_id"]
        path = TRACKS / f"{track_id}-{entry['slug']}" / "traceability.json"
        if not path.is_file():
            errors.append(f"missing {path.relative_to(ROOT)}")
            continue
        value = json.loads(path.read_text(encoding="utf-8"))
        if value.get("track_id") != track_id:
            errors.append(f"track identity mismatch in {path.relative_to(ROOT)}")
        if value.get("implementation_state") != entry.get("implementation_state"):
            errors.append(f"implementation state mismatch for track {track_id}")
        rows = value.get("assertions")
        if not isinstance(rows, list) or not rows:
            errors.append(f"track {track_id} has no assertions")
            continue
        seen: set[str] = set()
        states: set[str] = set()
        for row in rows:
            assertions += 1
            aid = row.get("assertion_id")
            if not isinstance(aid, str) or not re.fullmatch(fr"T{track_id}-A\d{{3}}", aid):
                errors.append(f"track {track_id} has invalid assertion id {aid!r}")
                continue
            if aid in seen:
                errors.append(f"track {track_id} duplicates {aid}")
            seen.add(aid)
            state = row.get("state")
            states.add(state)
            if state not in STATES:
                errors.append(f"{aid} has invalid state {state!r}")
            if not str(row.get("statement", "")).strip():
                errors.append(f"{aid} has no statement")
            for key in ("implementation_paths", "implementation_symbols", "deterministic_tests", "evidence_receipts", "open_gates"):
                if not isinstance(row.get(key), list):
                    errors.append(f"{aid}.{key} must be an array")
            for relative in row.get("implementation_paths", []):
                if not (ROOT / relative).exists():
                    errors.append(f"{aid} maps missing implementation path {relative}")
            if not str(row.get("permitted_claim", "")).strip():
                errors.append(f"{aid} has no permitted claim")
            if row.get("mapping_confidence") != "track_level_only":
                individually_mapped += 1
            if state == "source_implemented":
                if not row.get("implementation_symbols"):
                    errors.append(f"{aid} claims source implementation without symbol mapping")
                if not row.get("deterministic_tests"):
                    errors.append(f"{aid} claims source implementation without deterministic test mapping")
        track_state = entry.get("implementation_state")
        if track_state == "source_implemented" and states != {"source_implemented"}:
            errors.append(f"track {track_id} is source_implemented but not all assertions are")
        if track_state == "scaffolded" and "source_implemented" in states:
            errors.append(f"track {track_id} contains a source-implemented assertion but remains scaffolded")
        if track_state == "partially_implemented" and states == {"source_implemented"}:
            errors.append(f"track {track_id} is partial even though every assertion is source implemented")
    receipt = {
        "schema_version": "org.searchright.traceability-receipt.v1",
        "status": "failed" if errors else "passed",
        "tracks": len(coverage["tracks"]),
        "assertions": assertions,
        "individually_mapped_assertions": individually_mapped,
        "errors": errors,
        "limitations": [
            "Track-level mappings are intentionally not accepted as proof of complete behaviour.",
            "Compiler, runtime, live-provider and external validation remain separate evidence levels.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
