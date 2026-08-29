#!/usr/bin/env python3
"""Record exact-command receipts for Track 10 official `rmcp` client tests.

The script never treats a passed JSON-RPC smoke transcript as official-client
evidence: it runs the two typed SDK tests independently and labels their scope
and remaining limitations explicitly.  Receipt writes require ``--receipt-dir``
so local inspection cannot silently alter durable verification evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEST_TARGET = "live_client_conformance"
TESTS = {
    "2026-07-28": "official_rmcp_current_client_consumes_structured_results_and_governed_errors",
    "2025-11-25": "official_rmcp_previous_era_client_consumes_structured_results_and_governed_errors",
}
SOURCE_PATHS = (
    "Cargo.lock",
    "Cargo.toml",
    "contracts/interface-catalog.json",
    "contracts/mcp/tool-catalog.json",
    "contracts/json-schema/press-review.v1.schema.json",
    "crates/searchright/src/engine.rs",
    "crates/searchright-store/src/lib.rs",
    "crates/searchright-mcp/src/effect_policy.rs",
    "crates/searchright-mcp/src/lib.rs",
    "crates/searchright-mcp/tests/advanced_mcp.rs",
    "crates/searchright-mcp/tests/live_client_conformance.rs",
    "scripts/record_mcp_live_client_conformance.py",
)
TRANSCRIPT_SPEC = {
    "catalogue": "tools/list through official rmcp typed client",
    "success": "all tools advertised for each protocol era and both generate_prisma branches return schema-valid structuredContent",
    "governed_error": "semantically invalid validate_plan returns isError without structuredContent",
}
REPLACEABLE_RECEIPTS = {
    f"verification/receipts/mcp-official-client-{version}.json" for version in TESTS
}


def source_revision() -> str:
    return subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def worktree_status() -> str:
    return subprocess.run(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def require_clean_worktree() -> str:
    status = worktree_status()
    source_status = "\n".join(
        line
        for line in status.splitlines()
        if line[3:].split(" -> ")[-1] not in REPLACEABLE_RECEIPTS
    )
    if source_status:
        raise SystemExit("refusing to record MCP conformance receipts from a dirty worktree")
    return sha256_bytes(source_status.encode("utf-8"))


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_path(path: Path) -> str | None:
    return sha256_bytes(path.read_bytes()) if path.is_file() else None


def binary_path() -> Path | None:
    candidates = (
        ROOT / "target" / "debug" / "searchright-mcp.exe",
        ROOT / "target" / "debug" / "searchright-mcp",
    )
    return next((candidate for candidate in candidates if candidate.is_file()), None)


def bindings(protocol_version: str, observed: dict[str, object]) -> dict[str, object]:
    binary = binary_path()
    tools = observed["advertised_tools"]
    transcript = {"protocol_version": protocol_version, **TRANSCRIPT_SPEC, "tools": tools}
    return {
        "source_revision": source_revision(),
        "source_sha256": {path: sha256_path(ROOT / path) for path in SOURCE_PATHS},
        "interface_catalog_sha256": sha256_path(ROOT / "contracts/interface-catalog.json"),
        "binary_path": str(binary.relative_to(ROOT)) if binary else None,
        "binary_sha256": sha256_path(binary) if binary else None,
        "official_client_source_sha256": sha256_path(ROOT / "crates/searchright-mcp/tests/live_client_conformance.rs"),
        "transcript_spec_sha256": sha256_bytes(
            json.dumps(transcript, sort_keys=True, separators=(",", ":")).encode("utf-8")
        ),
        "advertised_tool_count": len(tools),
        "advertised_tools": tools,
    }


def run_test(test_name: str) -> tuple[list[str], subprocess.CompletedProcess[str]]:
    command = [
        "cargo",
        "test",
        "-p",
        "searchright-mcp",
        "--test",
        TEST_TARGET,
        "--locked",
        test_name,
        "--",
        "--exact",
        "--nocapture",
    ]
    with tempfile.TemporaryDirectory(prefix="searchright-mcp-receipt-") as store_root:
        environment = os.environ.copy()
        environment["SEARCHRIGHT_MCP_STORE_ROOT"] = store_root
        return command, subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env=environment,
        )


def observed_conformance(stdout: str, protocol_version: str) -> dict[str, object]:
    prefix = "SEARCHRIGHT_MCP_CONFORMANCE "
    observations = [
        json.loads(line.removeprefix(prefix))
        for line in stdout.splitlines()
        if line.startswith(prefix)
    ]
    if len(observations) != 1 or observations[0].get("protocol_version") != protocol_version:
        raise ValueError(f"missing unique conformance observation for {protocol_version}")
    observation = observations[0]
    tools = observation.get("advertised_tools")
    successes = observation.get("success_cases_validated")
    if not isinstance(tools, list) or not all(isinstance(tool, str) for tool in tools):
        raise ValueError("conformance observation has invalid advertised_tools")
    if tools != sorted(set(tools)):
        raise ValueError("conformance observation tools are not unique and sorted")
    if not isinstance(successes, int) or successes < len(tools):
        raise ValueError("conformance observation has invalid success count")
    return observation


def receipt(
    protocol_version: str,
    test_name: str,
    worktree_status_sha256: str,
) -> dict[str, object]:
    command, completed = run_test(test_name)
    status = "passed" if completed.returncode == 0 else "failed"
    observed = observed_conformance(completed.stdout, protocol_version) if status == "passed" else {
        "advertised_tools": [],
        "success_cases_validated": 0,
    }
    return {
        "schema_version": "org.searchright.mcp-official-client-receipt.v1",
        "status": status,
        "evidence_level": "compiler_verified" if status == "passed" else "source_verified",
        "track": "10",
        "roadmap_item": "LP-001",
        "protocol_version": protocol_version,
        "client_implementation": "official rmcp 3.1.2 typed client over child-process stdio",
        "test_target": TEST_TARGET,
        "test_name": test_name,
        "command": command,
        "exit_code": completed.returncode,
        "bindings": bindings(protocol_version, observed),
        "tracked_tree_clean": True,
        "tracked_status_sha256": worktree_status_sha256,
        "untracked_files_included_in_clean_check": True,
        "replaceable_receipt_outputs_excluded_from_clean_check": sorted(REPLACEABLE_RECEIPTS),
        "assertions": [
            "protocol negotiation selects the requested supported era",
            "the typed client observes every tool advertised for its protocol era and each output schema",
            "every advertised successful tool path exposes structuredContent matching outputSchema",
            "both generate_prisma union branches match the advertised outputSchema",
            "a malformed request remains a governed tool error without structuredContent",
        ],
        "limitations": [
            "This is local child-process stdio evidence, not an authenticated remote MCP deployment receipt.",
            "This does not establish third-party client interoperability beyond the official rmcp SDK version pinned by the workspace.",
        ],
        "advertised_tools_validated": len(observed["advertised_tools"]),
        "success_cases_validated": observed["success_cases_validated"],
        "stdout_sha256": sha256_bytes(completed.stdout.encode("utf-8")),
        "stderr_sha256": sha256_bytes(completed.stderr.encode("utf-8")),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--receipt-dir",
        type=Path,
        help="write per-era receipts here; omit for JSON-only dry run",
    )
    args = parser.parse_args()
    worktree_status_sha256 = require_clean_worktree()
    receipts = {
        version: receipt(version, test, worktree_status_sha256)
        for version, test in TESTS.items()
    }
    document = json.dumps(receipts, indent=2, sort_keys=True) + "\n"
    print(document, end="")
    if args.receipt_dir:
        output_dir = args.receipt_dir if args.receipt_dir.is_absolute() else ROOT / args.receipt_dir
        output_dir.mkdir(parents=True, exist_ok=True)
        for version, value in receipts.items():
            (output_dir / f"mcp-official-client-{version}.json").write_text(
                json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
    return 0 if all(item["status"] == "passed" for item in receipts.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
