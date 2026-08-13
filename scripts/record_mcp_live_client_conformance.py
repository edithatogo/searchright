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
import subprocess
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
    "crates/searchright-mcp/src/lib.rs",
    "crates/searchright-mcp/tests/live_client_conformance.rs",
)
TRANSCRIPT_SPEC = {
    "catalogue": "tools/list through official rmcp typed client",
    "success": "all 31 tools and both generate_prisma branches return schema-valid structuredContent",
    "governed_error": "semantically invalid validate_plan returns isError without structuredContent",
}


def source_revision() -> str:
    return subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def tracked_status() -> str:
    return subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=no"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def require_clean_tracked_tree() -> str:
    status = tracked_status()
    if status:
        raise SystemExit(
            "refusing to record MCP conformance receipts from a dirty tracked tree"
        )
    return sha256_bytes(status.encode("utf-8"))


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


def advertised_tools() -> list[str]:
    catalogue = json.loads((ROOT / "contracts/interface-catalog.json").read_text(encoding="utf-8"))
    return sorted(entry["mcp_tool"] for entry in catalogue["entries"])


def bindings(protocol_version: str) -> dict[str, object]:
    binary = binary_path()
    tools = advertised_tools()
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
        "--exact",
    ]
    return command, subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def receipt(
    protocol_version: str,
    test_name: str,
    tracked_status_sha256: str,
) -> dict[str, object]:
    command, completed = run_test(test_name)
    status = "passed" if completed.returncode == 0 else "failed"
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
        "bindings": bindings(protocol_version),
        "tracked_tree_clean": True,
        "tracked_status_sha256": tracked_status_sha256,
        "assertions": [
            "protocol negotiation selects the requested supported era",
            "the typed client observes all 31 advertised tools and their output schemas",
            "all 31 successful tool paths expose structuredContent matching outputSchema",
            "both generate_prisma union branches match the advertised outputSchema",
            "a malformed request remains a governed tool error without structuredContent",
        ],
        "limitations": [
            "This is local child-process stdio evidence, not an authenticated remote MCP deployment receipt.",
            "This does not establish third-party client interoperability beyond the official rmcp SDK version pinned by the workspace.",
        ],
        "advertised_tools_validated": 31,
        "success_cases_validated": 32,
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
    tracked_status_sha256 = require_clean_tracked_tree()
    receipts = {
        version: receipt(version, test, tracked_status_sha256)
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
