#!/usr/bin/env python3
"""Compile and smoke-test the generated contract-only bindings without network use."""
from __future__ import annotations

import json
import py_compile
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts/schema-catalog.json"


def main() -> int:
    contract_count = len(json.loads(CATALOG.read_text(encoding="utf-8"))["entries"])
    commands = [
        [sys.executable, "scripts/generate_contract_bindings.py", "--check"],
        [
            "node",
            "--experimental-strip-types",
            "--input-type=module",
            "--eval",
            (
                "import { CONTRACT_IDS } from './sdk/typescript/src/index.ts';"
                f"if (CONTRACT_IDS.length !== {contract_count}) process.exit(2);"
            ),
        ],
    ]
    errors: list[str] = []
    python_path = ROOT / "sdk/python/searchright_contracts/__init__.py"
    try:
        py_compile.compile(str(python_path), doraise=True)
        sys.path.insert(0, str(ROOT / "sdk/python"))
        import searchright_contracts  # type: ignore[import-not-found]

        if len(searchright_contracts.CONTRACT_IDS) != contract_count:
            errors.append("Python binding contract count differs from the catalogue")
    except (OSError, py_compile.PyCompileError, ImportError) as exc:
        errors.append(f"Python binding compilation failed: {type(exc).__name__}")

    results = []
    for command in commands:
        process = subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        results.append({"command": command, "returncode": process.returncode})
        if process.returncode != 0:
            errors.append(f"binding command failed: {' '.join(command[:3])}")

    receipt = {
        "schema_version": "org.searchright.contract-binding-check.v1",
        "status": "failed" if errors else "passed",
        "contracts": contract_count,
        "languages": ["python", "typescript"],
        "checks": results,
        "errors": errors,
        "limitations": [
            "Contract-only syntax and import evidence; package installation, client behaviour, publication and downstream conformance remain Track 35 gates."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
