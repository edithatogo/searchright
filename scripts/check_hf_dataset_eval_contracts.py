#!/usr/bin/env python3
"""Validate external dataset evaluation contracts, split definitions, and anti-leakage isolation."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "benchmarks/methodology/manifest.json"
REPORT_SCHEMA = ROOT / "contracts/json-schema/benchmark-report.v1.schema.json"


def check_hf_dataset_eval_contracts() -> dict[str, Any]:
    """Execute evaluation contract and partition isolation verification."""
    errors: list[str] = []
    contracts_checked = 0
    partitions_checked = 0

    if not MANIFEST.is_file():
        errors.append("missing methodology benchmark manifest")
    else:
        contracts_checked += 1
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

        partitions = manifest.get("partitions", {})
        expected_partitions = {"train", "validation", "sealed_test"}
        for part in expected_partitions:
            if part not in partitions:
                errors.append(f"methodology manifest missing partition {part}")
            else:
                partitions_checked += 1

        train = partitions.get("train", {})
        if train.get("labels_visible") is not True:
            errors.append("train partition must declare labels_visible=true")

        val = partitions.get("validation", {})
        if val.get("labels_visible") is not True:
            errors.append("validation partition must declare labels_visible=true")

        sealed = partitions.get("sealed_test", {})
        if sealed.get("labels_visible") is not False:
            errors.append("sealed_test partition must declare labels_visible=false")
        if not sealed.get("manifest"):
            errors.append("sealed_test partition must reference an external manifest path")

        controls = manifest.get("leakage_controls", {})
        for required_control in (
            "sealed_labels_committed",
            "sealed_labels_available_to_agents",
            "benchmark_test_ids_may_enter_training_prompts",
        ):
            if controls.get(required_control) is not False:
                errors.append(f"leakage control {required_control} must be false")

        for required_invariant in (
            "final_metric_computation_requires_external_label_mount",
            "configuration_digest_required",
            "environment_receipt_required",
        ):
            if controls.get(required_invariant) is not True:
                errors.append(f"leakage invariant {required_invariant} must be true")

    if not REPORT_SCHEMA.is_file():
        errors.append("missing benchmark report schema")
    else:
        contracts_checked += 1
        schema = json.loads(REPORT_SCHEMA.read_text(encoding="utf-8"))
        req = schema.get("required", [])
        for field in ("configuration_digest", "leakage_controls", "metrics", "claim_boundary"):
            if field not in req:
                errors.append(f"benchmark report schema must require {field}")

    return {
        "schema_version": "org.searchright.hf-dataset-eval-contracts-receipt.v1",
        "status": "failed" if errors else "passed",
        "contracts_checked": contracts_checked,
        "partitions_checked": partitions_checked,
        "errors": errors,
        "claim_boundary": (
            "External dataset evaluation contract checks verify split integrity and isolation; "
            "external execution remains pending."
        ),
    }


def main() -> int:
    receipt = check_hf_dataset_eval_contracts()
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
