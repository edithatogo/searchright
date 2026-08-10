#!/usr/bin/env python3
"""Install the exact Rust developer tools declared by Searchright."""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "requirements/rust-tools.json"


def run(args: list[str]) -> None:
    process = subprocess.run(args, cwd=ROOT, text=True, check=False)
    if process.returncode != 0:
        raise SystemExit(process.returncode)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", choices=("core", "all"), default="core")
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    selected = [tool for tool in manifest["tools"] if args.profile in tool["profiles"]]
    for tool in selected:
        command = [
            "cargo", "install", tool["crate"],
            "--version", f"={tool['version']}", "--locked",
        ]
        if args.force:
            command.append("--force")
        run(command)
    print(json.dumps({
        "schema_version": "org.searchright.rust-tools-install-plan.v1",
        "profile": args.profile,
        "tools": [{"crate": tool["crate"], "version": tool["version"]} for tool in selected],
        "status": "installation_commands_completed"
    }, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
