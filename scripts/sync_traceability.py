#!/usr/bin/env python3
"""Generate assertion-level traceability from Conductor specifications.

Path presence is not treated as proof of behaviour. Each scope statement becomes
an acceptance assertion with an implementation state, source mapping, test
mapping, evidence target and permitted claim. Track-level mappings are explicitly
marked as such until a symbol/test-specific override is supplied.
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COVERAGE = ROOT / "conductor" / "roadmap-coverage.json"
TRACKS = ROOT / "conductor" / "tracks"
OVERRIDES = ROOT / "conductor" / "traceability-overrides.json"

STATE_CLAIMS = {
    "contracted": "Requirement is contracted; no implementation claim is permitted.",
    "scaffolded": "Scaffolding exists; behaviour is not claimed.",
    "partially_implemented": "Some source paths implement the assertion; completeness is not claimed.",
    "source_implemented": "Source implementation is mapped, but compiler and runtime evidence remain separate.",
    "external_evidence_required": "Repository preparation exists; completion depends on external evidence.",
}


def scope_assertions(spec: str) -> list[str]:
    match = re.search(r"^## Scope\s*$\n(?P<body>.*?)(?=^##\s|\Z)", spec, re.MULTILINE | re.DOTALL)
    if not match:
        return []
    values: list[str] = []
    current: list[str] = []
    for raw in match.group("body").splitlines():
        line = raw.rstrip()
        if line.startswith("- "):
            if current:
                values.append(" ".join(current).strip())
            current = [line[2:].strip()]
        elif current and line.startswith("  "):
            current.append(line.strip())
        elif current and not line.strip():
            values.append(" ".join(current).strip())
            current = []
    if current:
        values.append(" ".join(current).strip())
    return [value for value in values if value]


def render(entry: dict, override: dict) -> str:
    track_id = entry["track_id"]
    directory = TRACKS / f"{track_id}-{entry['slug']}"
    assertions = scope_assertions((directory / "spec.md").read_text(encoding="utf-8"))
    if not assertions:
        assertions = [entry["outcome"]]
    default_state = entry["implementation_state"]
    assertion_overrides = override.get("assertions", {})
    result = []
    for number, statement in enumerate(assertions, start=1):
        assertion_id = f"T{track_id}-A{number:03d}"
        custom = assertion_overrides.get(assertion_id, {})
        state = custom.get("state", default_state)
        result.append(
            {
                "assertion_id": assertion_id,
                "statement": statement,
                "state": state,
                "mapping_confidence": custom.get("mapping_confidence", "track_level_only"),
                "implementation_paths": custom.get("implementation_paths", entry.get("deliverables", [])),
                "implementation_symbols": custom.get("implementation_symbols", []),
                "deterministic_tests": custom.get("deterministic_tests", entry.get("checks", [])),
                "evidence_receipts": custom.get(
                    "evidence_receipts",
                    [f"conductor/tracks/{track_id}-{entry['slug']}/evidence.json"],
                ),
                "open_gates": custom.get("open_gates", entry.get("blockers", [])),
                "permitted_claim": custom.get("permitted_claim", STATE_CLAIMS[state]),
            }
        )
    payload = {
        "schema_version": "org.searchright.track-traceability.v1",
        "track_id": track_id,
        "title": entry["title"],
        "implementation_state": entry["implementation_state"],
        "evidence_level": entry["evidence_level"],
        "source_of_assertions": f"conductor/tracks/{track_id}-{entry['slug']}/spec.md#scope",
        "claim_boundary": entry["claim_boundary"],
        "assertions": result,
    }
    return json.dumps(payload, indent=2, ensure_ascii=False) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    coverage = json.loads(COVERAGE.read_text(encoding="utf-8"))
    overrides = json.loads(OVERRIDES.read_text(encoding="utf-8")) if OVERRIDES.is_file() else {"tracks": {}}
    outputs: dict[Path, str] = {}
    for entry in coverage["tracks"]:
        directory = TRACKS / f"{entry['track_id']}-{entry['slug']}"
        outputs[directory / "traceability.json"] = render(
            entry, overrides.get("tracks", {}).get(entry["track_id"], {})
        )
    stale: list[str] = []
    for path, content in outputs.items():
        if args.check:
            if not path.is_file() or path.read_text(encoding="utf-8") != content:
                stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.write_text(content, encoding="utf-8")
    print(
        json.dumps(
            {
                "schema_version": "org.searchright.traceability-sync-receipt.v1",
                "status": "failed" if stale else "passed",
                "mode": "check" if args.check else "write",
                "tracks": len(outputs),
                "stale": stale,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 1 if stale else 0


if __name__ == "__main__":
    raise SystemExit(main())
