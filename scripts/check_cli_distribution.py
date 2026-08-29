#!/usr/bin/env python3
"""Exercise a built or installed Searchright CLI using stable snapshots."""

from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SNAPSHOTS = ROOT / "crates" / "searchright-cli" / "tests" / "snapshots"


def invoke(binary: Path, *arguments: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(binary), *arguments],
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )


def normalise(value: str) -> str:
    return value.replace("\r\n", "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    arguments = parser.parse_args()
    binary = arguments.binary.resolve()
    errors: list[str] = []

    help_result = invoke(binary, "--help")
    expected_help = (SNAPSHOTS / "help.txt").read_text(encoding="utf-8")
    if help_result.returncode != 0 or normalise(help_result.stdout) != expected_help:
        errors.append("help output did not match the checked-in snapshot")

    with tempfile.TemporaryDirectory(prefix="searchright-cli-") as directory:
        temporary = Path(directory)
        init_result = invoke(binary, "init", "--target", "snapshot.json", cwd=temporary)
        expected_init = (SNAPSHOTS / "init.json").read_text(encoding="utf-8")
        if init_result.returncode != 0 or normalise(init_result.stdout) != expected_init:
            errors.append("dry-run JSON did not match the checked-in snapshot")
        if (temporary / "snapshot.json").exists():
            errors.append("dry-run init wrote its target")

    error_result = invoke(binary, "invalid-command")
    expected_error = json.loads((SNAPSHOTS / "usage-error.json").read_text(encoding="utf-8"))
    try:
        observed_error = json.loads(error_result.stderr)
    except json.JSONDecodeError:
        observed_error = None
    if error_result.returncode != 2 or observed_error != expected_error:
        errors.append("usage error did not match the checked-in JSON snapshot")

    completions = invoke(binary, "completions", "bash")
    if completions.returncode != 0 or "_searchright" not in completions.stdout:
        errors.append("Bash completion generation failed")

    manpage = invoke(binary, "manpage")
    if manpage.returncode != 0 or ".TH searchright 1" not in manpage.stdout:
        errors.append("manual-page generation failed")

    receipt = {
        "schema_version": "org.searchright.cli-distribution-check.v1",
        "status": "failed" if errors else "passed",
        "checks": ["help_snapshot", "dry_run_json", "usage_error_json", "completions", "manpage"],
        "errors": errors,
        "limitations": [
            "The receipt applies only to the supplied binary and operating system.",
            "Installation provenance and hosted cross-platform execution remain separate evidence.",
        ],
    }
    print(json.dumps(receipt, indent=2))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
