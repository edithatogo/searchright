#!/usr/bin/env python3
"""Validate methodology benchmark structure and prevent sealed-label leakage."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "benchmarks/methodology/manifest.json"
SEALED = ROOT / "benchmarks/methodology/fixtures/sealed/manifest.json"
FORBIDDEN_KEYS = {"label", "labels", "gold", "gold_clusters", "gold_studies", "expected_labels", "outcomes"}
SCAN_ROOTS = [ROOT / "crates", ROOT / "skills", ROOT / "agents", ROOT / "prompts"]


def contains_forbidden_key(value: object) -> bool:
    if isinstance(value, dict):
        return any(key in FORBIDDEN_KEYS or contains_forbidden_key(child) for key, child in value.items())
    if isinstance(value, list):
        return any(contains_forbidden_key(child) for child in value)
    return False


def main() -> int:
    errors: list[str] = []
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    sealed = json.loads(SEALED.read_text(encoding="utf-8"))
    if manifest.get("schema_version") != "org.searchright.methodology-benchmark-suite.v1":
        errors.append("unexpected methodology benchmark schema version")
    if manifest.get("status") != "synthetic_fixture_ready_external_validation_pending":
        errors.append("benchmark status must not imply external validation")
    tasks = manifest.get("tasks", [])
    ids = [item.get("id") for item in tasks if isinstance(item, dict)]
    if len(ids) != len(set(ids)) or len(ids) < 6:
        errors.append("methodology tasks must be unique and cover the declared task families")
    for item in tasks:
        fixture = item.get("fixture")
        if not fixture or not (ROOT / fixture).is_file():
            errors.append(f"missing benchmark fixture {fixture!r}")
        if not item.get("metrics") or not item.get("claim_boundary"):
            errors.append(f"benchmark task {item.get('id')} lacks metrics or a claim boundary")
    partitions = manifest.get("partitions", {})
    if partitions.get("sealed_test", {}).get("labels_visible") is not False:
        errors.append("sealed-test labels must be invisible")
    leakage = manifest.get("leakage_controls", {})
    for key in (
        "sealed_labels_committed",
        "sealed_labels_available_to_agents",
        "benchmark_test_ids_may_enter_training_prompts",
    ):
        if leakage.get(key) is not False:
            errors.append(f"leakage control {key} must be false")
    if sealed.get("labels_present") is not False or sealed.get("label_digest") is not None:
        errors.append("sealed manifest must contain neither labels nor a label digest before evaluation")
    if contains_forbidden_key(sealed):
        errors.append("sealed manifest contains a label-like key")
    sealed_ids = sealed.get("case_ids", [])
    if not sealed_ids or len(sealed_ids) != len(set(sealed_ids)):
        errors.append("sealed case identifiers must be non-empty and unique")
    token_pattern = re.compile("|".join(re.escape(value) for value in sealed_ids))
    leaked_paths: list[str] = []
    for scan_root in SCAN_ROOTS:
        if not scan_root.exists():
            continue
        for path in scan_root.rglob("*"):
            if path.is_file() and path.suffix.lower() in {".rs", ".md", ".json", ".yaml", ".yml", ".txt"}:
                try:
                    text = path.read_text(encoding="utf-8")
                except UnicodeDecodeError:
                    continue
                if token_pattern.search(text):
                    leaked_paths.append(path.relative_to(ROOT).as_posix())
    if leaked_paths:
        errors.append(f"sealed case identifiers leaked into implementation or prompt paths: {leaked_paths}")
    upstreams = manifest.get("external_upstreams", [])
    synergy = next((item for item in upstreams if item.get("name") == "SYNERGY"), None)
    if not synergy or synergy.get("canonical_repository") != "asreview/synergy-dataset":
        errors.append("SYNERGY must identify the canonical upstream repository")
    if synergy and synergy.get("local_fork_role") != "mirror_or_patch_carrier_only":
        errors.append("the personal SYNERGY fork must not be treated as canonical")
    receipt = {
        "schema_version": "org.searchright.methodology-benchmark-receipt.v1",
        "status": "failed" if errors else "passed",
        "tasks_checked": len(tasks),
        "sealed_cases_checked": len(sealed_ids),
        "external_results": 0,
        "errors": errors,
        "limitations": [
            "Only rights-clear synthetic fixtures and leakage policy were checked.",
            "No external benchmark, hidden-label evaluation, methodological comparison or performance claim was executed.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
