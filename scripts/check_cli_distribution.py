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
    if not binary.is_file() and binary.suffix.lower() != ".exe":
        executable = Path(f"{binary}.exe")
        if executable.is_file():
            binary = executable
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

        apply_result = invoke(binary, "init", "--target", "snapshot.json", "--apply", cwd=temporary)
        try:
            apply_document = json.loads(apply_result.stdout)
        except json.JSONDecodeError:
            apply_document = None
        created = temporary / "snapshot.json"
        if (
            apply_result.returncode != 0
            or not isinstance(apply_document, dict)
            or apply_document.get("mode") != "apply"
            or apply_document.get("changed") is not True
            or not created.is_file()
        ):
            errors.append("explicit apply did not create the expected configuration")
        original_bytes = created.read_bytes() if created.is_file() else b""
        refusal = invoke(binary, "init", "--target", "snapshot.json", "--apply", cwd=temporary)
        try:
            refusal_document = json.loads(refusal.stderr)
        except json.JSONDecodeError:
            refusal_document = None
        if (
            refusal.returncode != 3
            or refusal.stdout
            or not isinstance(refusal_document, dict)
            or refusal_document.get("code") != "cli.filesystem"
            or refusal_document.get("stage") != "init"
            or refusal_document.get("category") != "filesystem"
            or not created.is_file()
            or created.read_bytes() != original_bytes
        ):
            errors.append("second apply did not fail closed without changing the target")

    error_result = invoke(binary, "invalid-command")
    expected_error = json.loads((SNAPSHOTS / "usage-error.json").read_text(encoding="utf-8"))
    try:
        observed_error = json.loads(error_result.stderr)
    except json.JSONDecodeError:
        observed_error = None
    if error_result.returncode != 2 or observed_error != expected_error:
        errors.append("usage error did not match the checked-in JSON snapshot")

    completion_markers = {
        "bash": "_searchright()",
        "elvish": "edit:completion:arg-completer[searchright]",
        "fish": "__fish_searchright_global_optspecs",
        "powershell": "Register-ArgumentCompleter -Native -CommandName 'searchright'",
        "zsh": "#compdef searchright",
    }
    for shell, marker in completion_markers.items():
        completions = invoke(binary, "completions", shell)
        if completions.returncode != 0 or marker not in completions.stdout:
            errors.append(f"{shell} completion generation failed")

    secret = "TRACK09_SENTINEL_SECRET"
    envelope = ROOT / "contracts" / "examples" / "execution-envelope.yaml"
    query_endpoint = f"https://eutils.ncbi.nlm.nih.gov/path?api_key={secret}"
    for prefix in (("run", "authorise-endpoint"), ("authorise-endpoint",)):
        authority = invoke(binary, *prefix, str(envelope), query_endpoint)
        if authority.returncode != 0 or secret in authority.stdout or secret in authority.stderr:
            errors.append(f"{' '.join(prefix)} leaked a query credential")
        else:
            authority_result = json.loads(authority.stdout)
            if authority_result.get("endpoint") != "https://eutils.ncbi.nlm.nih.gov":
                errors.append(f"{' '.join(prefix)} did not emit the sanitized endpoint origin")
    credential_endpoint = f"https://user:{secret}@eutils.ncbi.nlm.nih.gov/path"
    authority = invoke(binary, "run", "authorise-endpoint", str(envelope), credential_endpoint)
    if authority.returncode != 3 or secret in authority.stdout or secret in authority.stderr:
        errors.append("credential-bearing endpoint did not fail without reflection")

    manpage = invoke(binary, "manpage")
    if manpage.returncode != 0 or ".TH searchright 1" not in manpage.stdout:
        errors.append("manual-page generation failed")

    receipt = {
        "schema_version": "org.searchright.cli-distribution-check.v1",
        "status": "failed" if errors else "passed",
        "checks": [
            "help_snapshot",
            "dry_run_json",
            "no_clobber_apply_refusal",
            "usage_error_json",
            "all_completions",
            "endpoint_secret_non_reflection",
            "manpage",
        ],
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
