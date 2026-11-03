#!/usr/bin/env python3
"""Validate benchmark governance, anti-leakage controls, and report contracts."""
from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "benchmarks/methodology/manifest.json"
SEALED = ROOT / "benchmarks/methodology/fixtures/sealed/manifest.json"
REPORT_SCHEMA = ROOT / "contracts/json-schema/benchmark-report.v1.schema.json"
FORBIDDEN_KEYS = {"label", "labels", "gold", "gold_clusters", "gold_studies", "expected_labels", "outcomes"}
SCAN_ROOTS = [ROOT / "crates", ROOT / "skills", ROOT / "agents", ROOT / "prompts"]


def contains_forbidden_key(value: object) -> bool:
    """Check if any dictionary key matches forbidden label identifiers."""
    if isinstance(value, dict):
        return any(key in FORBIDDEN_KEYS or contains_forbidden_key(child) for key, child in value.items())
    if isinstance(value, list):
        return any(contains_forbidden_key(child) for child in value)
    return False


def check_benchmark_governance() -> dict[str, Any]:
    """Execute all benchmark governance and anti-leakage verification checks."""
    errors: list[str] = []
    if not MANIFEST.is_file():
        errors.append("missing methodology benchmark manifest")
        manifest: dict[str, Any] = {}
    else:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    if not SEALED.is_file():
        errors.append("missing sealed benchmark manifest")
        sealed: dict[str, Any] = {}
    else:
        sealed = json.loads(SEALED.read_text(encoding="utf-8"))

    if manifest.get("schema_version") != "org.searchright.methodology-benchmark-suite.v1":
        errors.append("unexpected methodology benchmark suite schema version")
    if manifest.get("status") != "synthetic_fixture_ready_external_validation_pending":
        errors.append("benchmark status must not claim external validation")

    tasks = manifest.get("tasks", [])
    task_ids = [t.get("id") for t in tasks if isinstance(t, dict)]
    if len(task_ids) != len(set(task_ids)) or len(task_ids) < 6:
        errors.append("methodology tasks must be unique and cover at least 6 core task families")

    for task in tasks:
        fixture_rel = task.get("fixture")
        if not fixture_rel or not (ROOT / fixture_rel).is_file():
            errors.append(f"task {task.get('id')} references missing fixture {fixture_rel!r}")
        if not task.get("metrics") or not isinstance(task.get("metrics"), list):
            errors.append(f"task {task.get('id')} lacks a valid metrics list")
        if not str(task.get("claim_boundary", "")).strip():
            errors.append(f"task {task.get('id')} lacks an explicit claim boundary")

    partitions = manifest.get("partitions", {})
    if partitions.get("sealed_test", {}).get("labels_visible") is not False:
        errors.append("sealed_test partition must declare labels_visible=false")

    leakage = manifest.get("leakage_controls", {})
    for control in (
        "sealed_labels_committed",
        "sealed_labels_available_to_agents",
        "benchmark_test_ids_may_enter_training_prompts",
    ):
        if leakage.get(control) is not False:
            errors.append(f"leakage control {control} must be false")
    for requirement in ("configuration_digest_required", "environment_receipt_required"):
        if leakage.get(requirement) is not True:
            errors.append(f"leakage control {requirement} must be true")

    if sealed.get("schema_version") != "org.searchright.sealed-benchmark-manifest.v1":
        errors.append("unexpected sealed benchmark manifest schema version")
    if sealed.get("partition") != "sealed_test":
        errors.append("sealed manifest partition must be sealed_test")
    if sealed.get("labels_present") is not False or sealed.get("label_digest") is not None:
        errors.append("sealed manifest must not present labels or label digest in repository")
    if contains_forbidden_key(sealed):
        errors.append("sealed manifest contains forbidden label keys")

    case_ids = sealed.get("case_ids", [])
    if not case_ids or len(case_ids) != len(set(case_ids)):
        errors.append("sealed case IDs must be non-empty and unique")

    token_pattern = re.compile("|".join(re.escape(val) for val in case_ids)) if case_ids else None
    leaked_paths: list[str] = []
    if token_pattern:
        for scan_root in SCAN_ROOTS:
            if not scan_root.exists():
                continue
            for path in scan_root.rglob("*"):
                if path.is_file() and path.suffix.lower() in {".rs", ".md", ".json", ".yaml", ".yml", ".txt"}:
                    try:
                        content = path.read_text(encoding="utf-8")
                    except UnicodeDecodeError:
                        continue
                    if token_pattern.search(content):
                        leaked_paths.append(path.relative_to(ROOT).as_posix())

    if leaked_paths:
        errors.append(f"sealed case identifiers leaked into codebase: {leaked_paths}")

    if not REPORT_SCHEMA.is_file():
        errors.append("missing benchmark report JSON schema")
    else:
        schema_obj = json.loads(REPORT_SCHEMA.read_text(encoding="utf-8"))
        req_props = schema_obj.get("required", [])
        for expected in (
            "schema_version",
            "benchmark_id",
            "corpus_id",
            "corpus_version",
            "rights_basis",
            "implementation_version",
            "configuration_digest",
            "leakage_controls",
            "metrics",
            "environment",
            "generated_at",
            "claim_boundary",
        ):
            if expected not in req_props:
                errors.append(f"benchmark report schema missing required property {expected}")

    return {
        "schema_version": "org.searchright.benchmark-governance-receipt.v1",
        "status": "failed" if errors else "passed",
        "tasks_checked": len(tasks),
        "sealed_cases_checked": len(case_ids),
        "scan_roots_checked": len(SCAN_ROOTS),
        "errors": errors,
        "limitations": [
            "Governance checks verify static contracts, anti-leakage policy and sealed split isolation.",
            "No external benchmark execution, model comparison, or performance claim is validated.",
        ],
    }


def main() -> int:
    receipt = check_benchmark_governance()
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
