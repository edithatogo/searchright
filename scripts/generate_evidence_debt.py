#!/usr/bin/env python3
"""Generate a deterministic evidence-debt register from canonical ledgers."""
from __future__ import annotations

import argparse
import importlib.util
import json
from collections import Counter
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "verification" / "evidence-debt.json"
TRACKS = ROOT / "conductor" / "tracks"
MATURITY = ROOT / "conductor" / "maturity-dossier.json"
PACKAGES = ROOT / "release" / "public-packages.json"
PROVIDERS = ROOT / "integration" / "provider-policies" / "index.json"


def load_gate_catalog() -> dict[str, Any]:
    spec = importlib.util.spec_from_file_location(
        "searchright_gate_catalog", ROOT / "scripts" / "check_gate_catalog.py"
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load gate catalogue generator")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.render()


def render() -> dict[str, Any]:
    assertions: list[dict[str, Any]] = []
    track_states: Counter[str] = Counter()
    for path in sorted(TRACKS.glob("*/traceability.json")):
        value = json.loads(path.read_text(encoding="utf-8"))
        track_states[value.get("implementation_state", "unknown")] += 1
        for row in value.get("assertions", []):
            copy = dict(row)
            copy["track_id"] = value.get("track_id")
            assertions.append(copy)
    gate_catalog = load_gate_catalog()
    registered_commands = {gate["command"] for gate in gate_catalog["gates"]}
    assertion_commands = {
        command
        for row in assertions
        for command in row.get("deterministic_tests", [])
    }
    maturity = json.loads(MATURITY.read_text(encoding="utf-8"))
    packages = json.loads(PACKAGES.read_text(encoding="utf-8"))
    providers = json.loads(PROVIDERS.read_text(encoding="utf-8"))
    states = Counter(row.get("state", "unknown") for row in assertions)
    confidence = Counter(row.get("mapping_confidence", "unknown") for row in assertions)
    open_gates = sum(len(row.get("open_gates", [])) for row in assertions)
    without_symbols = [row["assertion_id"] for row in assertions if not row.get("implementation_symbols")]
    track_level_only = [
        row["assertion_id"] for row in assertions if row.get("mapping_confidence") == "track_level_only"
    ]
    priority_queue = [
        {
            "priority": 1,
            "debt": "assertion_specific_proof",
            "reason": f"{len(track_level_only)} assertions retain track-level-only mappings.",
            "closure_evidence": ["implementation symbol", "assertion-specific deterministic test", "current receipt"],
        },
        {
            "priority": 2,
            "debt": "provider_runtime_and_policy",
            "reason": "Provider fixtures and conservative policies exist, but live canaries and policy review are not evidenced.",
            "closure_evidence": ["authorised live canary", "redacted receipt", "policy review evidence"],
        },
        {
            "priority": 3,
            "debt": "methodological_validation",
            "reason": "Search translation, recall, deduplication, linkage and screening require independent gold-standard evaluation.",
            "closure_evidence": ["sealed benchmark receipt", "information-specialist review", "calibration report"],
        },
        {
            "priority": 4,
            "debt": "downstream_migration",
            "reason": "Sourceright, CiteWeft and estate migrations are prepared but not dual-run or cut over.",
            "closure_evidence": ["producer/consumer receipts", "dual-run parity", "rollback rehearsal"],
        },
        {
            "priority": 5,
            "debt": "operations_and_adoption",
            "reason": "Authenticated hosting, restore drills, pilots, generated SDK adoption and release acceptance remain unevidenced.",
            "closure_evidence": ["tenant-isolation receipt", "restore receipt", "pilot exit", "downstream SDK receipt", "release acceptance"],
        },
    ]
    return {
        "schema_version": "org.searchright.evidence-debt.v1",
        "evidence_ceiling": "source_verified",
        "tracks": {
            "total": sum(track_states.values()),
            "by_state": dict(sorted(track_states.items())),
        },
        "assertions": {
            "total": len(assertions),
            "by_state": dict(sorted(states.items())),
            "by_mapping_confidence": dict(sorted(confidence.items())),
            "track_level_only": len(track_level_only),
            "without_symbol_mapping": len(without_symbols),
            "open_gate_entries": open_gates,
        },
        "static_gates": {
            "catalogued": len(gate_catalog["gates"]),
            "harness": sum(1 for gate in gate_catalog["gates"] if gate["harness_gate"]),
            "unregistered_assertion_commands": sorted(assertion_commands - registered_commands),
        },
        "maturity": {
            "decision": maturity.get("decision"),
            "critical_blockers": sorted(
                row.get("domain")
                for row in maturity.get("domains", [])
                if row.get("critical_blocker") is True
            ),
        },
        "publication": {
            "candidate_packages": len(packages.get("packages", [])),
            "publish_ready": sum(1 for row in packages.get("packages", []) if row.get("publish_ready") is True),
        },
        "provider_policy": {
            "providers": len(providers.get("providers", [])),
            "reviewed_with_evidence": sum(
                1
                for row in providers.get("providers", [])
                if row.get("policy_review_status") == "reviewed_with_evidence"
            ),
        },
        "priority_queue": priority_queue,
        "claim_boundary": "The register exposes unresolved proof and implementation work. Counts are not a quality score and cannot be used to promote claims without the named evidence."
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected_value = render()
    expected = json.dumps(expected_value, indent=2, sort_keys=True) + "\n"
    stale = not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected
    if args.write:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(expected, encoding="utf-8")
        stale = False
    errors: list[str] = []
    if expected_value["static_gates"]["unregistered_assertion_commands"]:
        errors.append("traceability includes commands absent from the gate catalogue")
    if args.check and stale:
        errors.append("verification/evidence-debt.json is stale; run generate_evidence_debt.py --write")
    receipt = {
        "schema_version": "org.searchright.evidence-debt-receipt.v1",
        "status": "failed" if errors else "passed",
        "stale": stale,
        "assertions": expected_value["assertions"]["total"],
        "track_level_only": expected_value["assertions"]["track_level_only"],
        "critical_blockers": len(expected_value["maturity"]["critical_blockers"]),
        "errors": errors,
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
