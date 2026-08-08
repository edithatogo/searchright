#!/usr/bin/env python3
"""Run all network-free repository gates and aggregate their receipts."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMANDS = [
    [sys.executable, "scripts/validate_repository.py"],
    [sys.executable, "scripts/check_traceability.py"],
    [sys.executable, "scripts/check_public_package_policy.py"],
    [sys.executable, "scripts/sync_schema_surface.py", "--check"],
    [sys.executable, "scripts/check_native_query_corpus.py"],
    [sys.executable, "scripts/check_provider_contract_baselines.py"],
    [sys.executable, "scripts/review_bundle.py", "self-test"],
    [sys.executable, "scripts/check_vertical_slice.py"],
    [sys.executable, "scripts/reduce_review_events.py", "--self-test"],
    [sys.executable, "scripts/check_methodology_benchmarks.py"],
    [sys.executable, "scripts/check_licence_firewall.py"],
    [sys.executable, "scripts/check_companion_change_packets.py"],
    [sys.executable, "scripts/check_portfolio_project.py"],
    [sys.executable, "scripts/sync_ecosystem_lock.py", "--check"],
    [sys.executable, "scripts/check_citeweft_integration.py"],
    [sys.executable, "scripts/check_integration_passports.py"],
    [sys.executable, "scripts/check_consumer_contracts.py"],
    [sys.executable, "scripts/check_integration_drift.py"],
    [sys.executable, "scripts/render_github_issues.py", "--check"],
    [sys.executable, "scripts/check_github_issue_hierarchy.py"],
    [sys.executable, "scripts/check_github_project.py"],
    [sys.executable, "scripts/sync_github_issues.py", "--repo", "edithatogo/searchright"],
    [sys.executable, "scripts/sync_github_project.py"],
    [sys.executable, "scripts/bootstrap_github.py"],
    [sys.executable, "scripts/check_context_integrity.py"],
    [sys.executable, "scripts/sync_context_lock.py", "--check"],
    [sys.executable, "scripts/check_default_deny.py"],
    [sys.executable, "scripts/check_workflow_hardening.py"],
    [sys.executable, "scripts/check_toolchain_manifest.py"],
    [sys.executable, "scripts/check_cli_mcp_parity.py"],
    [sys.executable, "scripts/check_sourceright_migration.py"],
    [sys.executable, "scripts/check_roadmap_coverage.py"],
    [sys.executable, "scripts/check_release_train.py"],
    [sys.executable, "scripts/check_sdk_examples.py"],
    [sys.executable, "scripts/check_release_rehearsal.py"],
    [sys.executable, "scripts/check_maturity_dossier.py"],
    [sys.executable, "scripts/sync_track_evidence.py", "--check"],
    [sys.executable, "scripts/check_rust_dependency_graph.py"],
    [sys.executable, "scripts/check_rust_source_structure.py"],
    [sys.executable, "scripts/audit_search_code.py", "--self-test"],
    [sys.executable, "scripts/check_secrets.py"],
    [sys.executable, "scripts/generate_source_sbom.py", "--check"],
    [sys.executable, "scripts/generate_source_hash_manifest.py", "--check"],
    [sys.executable, "scripts/check_packaging_reproducibility.py"],
]

def main() -> int:
    results = []
    failed = False
    for index, command in enumerate(COMMANDS, start=1):
        print(f"[static-harness {index}/{len(COMMANDS)}] {' '.join(command)}", file=sys.stderr, flush=True)
        started = time.monotonic()
        try:
            process = subprocess.run(
                command,
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
                timeout=300,
            )
            returncode = process.returncode
            stdout = process.stdout.strip()
            stderr = process.stderr.strip()
            timed_out = False
        except subprocess.TimeoutExpired as exc:
            returncode = 124
            stdout = (exc.stdout or "").strip() if isinstance(exc.stdout, str) else ""
            stderr = (exc.stderr or "").strip() if isinstance(exc.stderr, str) else ""
            stderr = f"{stderr}\ngate exceeded the 300-second network-free budget".strip()
            timed_out = True
        duration_ms = round((time.monotonic() - started) * 1000)
        failed = failed or returncode != 0
        print(
            f"[static-harness {index}/{len(COMMANDS)}] rc={returncode} duration_ms={duration_ms}",
            file=sys.stderr,
            flush=True,
        )
        results.append(
            {
                "command": " ".join(command),
                "returncode": returncode,
                "duration_ms": duration_ms,
                "timed_out": timed_out,
                "stdout": stdout,
                "stderr": stderr,
            }
        )
    receipt = {
        "schema_version": "org.searchright.static-harness-receipt.v1",
        "status": "failed" if failed else "passed",
        "gates": results,
        "limitations": [
            "No Rust compiler, live provider, remote GitHub, registry or external reviewer evidence is represented by this harness.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
