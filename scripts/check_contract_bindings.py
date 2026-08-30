#!/usr/bin/env python3
"""Compile and smoke-test the generated contract-only bindings without network use."""
from __future__ import annotations

import json
import py_compile
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts/schema-catalog.json"


def main() -> int:
    contract_count = len(json.loads(CATALOG.read_text(encoding="utf-8"))["entries"])
    commands = [
        [sys.executable, "tests/test_contract_generation.py"],
        [sys.executable, "scripts/generate_contract_bindings.py", "--check"],
        ["node", "requirements/bindings/node_modules/typescript/bin/tsc",
         "--project", "tests/fixtures/bindings/tsconfig.json"],
        ["node", "requirements/bindings/node_modules/pyright/index.js",
         "--project", "tests/fixtures/bindings/pyrightconfig.json"],
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

    python_source = python_path.read_text(encoding="utf-8")
    python_names = [
        left or right
        for left, right in re.findall(
            r"^(\w+) = TypedDict\(|^(\w+): TypeAlias =", python_source, re.MULTILINE
        )
    ]
    duplicate_python = sorted(
        name for name, count in Counter(python_names).items() if count > 1
    )
    if duplicate_python:
        errors.append(f"duplicate Python declarations: {', '.join(duplicate_python)}")

    typescript_path = ROOT / "sdk/typescript/src/index.ts"
    typescript_names = re.findall(
        r"^export type (\w+) =", typescript_path.read_text(encoding="utf-8"), re.MULTILINE
    )
    duplicate_typescript = sorted(
        name for name, count in Counter(typescript_names).items() if count > 1
    )
    if duplicate_typescript:
        errors.append(
            f"duplicate TypeScript declarations: {', '.join(duplicate_typescript)}"
        )

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

    negative_command = [
        "node", "requirements/bindings/node_modules/pyright/index.js",
        "--project", "tests/fixtures/bindings/pyrightconfig.json", "--outputjson",
        "tests/fixtures/bindings/invalid_map.py",
    ]
    negative = subprocess.run(negative_command, cwd=ROOT, check=False,
                              capture_output=True, text=True)
    try:
        diagnostics = json.loads(negative.stdout).get("generalDiagnostics", [])
    except json.JSONDecodeError:
        diagnostics = []
    rejected_as_expected = (
        negative.returncode == 1 and len(diagnostics) == 1
        and diagnostics[0].get("rule") == "reportAssignmentType"
        and diagnostics[0].get("range", {}).get("start", {}).get("line") == 2
        and Path(diagnostics[0].get("file", "")).name == "invalid_map.py"
    )
    results.append({"command": negative_command, "returncode": negative.returncode,
                    "expected_rejection": rejected_as_expected})
    if not rejected_as_expected:
        errors.append("Python dictionary fixture did not reject the invalid value type")

    receipt = {
        "schema_version": "org.searchright.contract-binding-check.v1",
        "status": "failed" if errors else "passed",
        "contracts": contract_count,
        "languages": ["python", "typescript"],
        "checks": results,
        "errors": errors,
        "limitations": [
            "Contract-only static typing, assignment fixtures, syntax and import evidence; JSON Schema validation, package installation, client behaviour, publication and downstream conformance remain separate gates."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
