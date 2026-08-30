#!/usr/bin/env python3
"""Record and verify compiled Rust/canonical JSON Schema differences."""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts" / "schema-catalog.json"
REPORT = ROOT / "contracts" / "compatibility" / "rust-schema-parity.json"


def canonicalise(value: Any) -> Any:
    """Remove schema annotations, never property names or literal instance data.

    Dialect and reference-base keywords remain significant. Unknown keyword data
    is preserved conservatively rather than assumed to contain nested schemas.
    """
    if not isinstance(value, dict):
        return value
    schema_maps = {"$defs", "definitions", "properties", "patternProperties", "dependentSchemas"}
    schema_values = {"additionalProperties", "unevaluatedProperties", "propertyNames",
                     "contains", "if", "then", "else", "not", "additionalItems",
                     "unevaluatedItems", "contentSchema"}
    schema_arrays = {"allOf", "anyOf", "oneOf", "prefixItems"}
    result = {}
    for key, item in sorted(value.items()):
        if key in {"title", "description"}:
            continue
        if key in schema_maps and isinstance(item, dict):
            result[key] = {name: canonicalise(schema) for name, schema in sorted(item.items())}
        elif key == "dependencies" and isinstance(item, dict):
            result[key] = {name: canonicalise(schema) if isinstance(schema, dict) else schema
                           for name, schema in sorted(item.items())}
        elif key in schema_values or (key == "items" and not isinstance(item, list)):
            result[key] = canonicalise(item)
        elif (key in schema_arrays or key == "items") and isinstance(item, list):
            result[key] = [canonicalise(schema) for schema in item]
        else:
            result[key] = item
    return result


def semantic_digest(schema: Any) -> str:
    encoded = json.dumps(canonicalise(schema), sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(encoded.encode("utf-8")).hexdigest()


def difference_paths(left: Any, right: Any, path: str = "") -> list[str]:
    """Return JSON-pointer-like paths whose validation shapes differ."""
    if type(left) is not type(right):
        return [path or "/"]
    if isinstance(left, dict):
        differences = [
            f"{path}/{key}"
            for key in sorted(set(left) ^ set(right))
        ]
        for key in sorted(set(left) & set(right)):
            differences.extend(difference_paths(left[key], right[key], f"{path}/{key}"))
        return differences
    if isinstance(left, list):
        if len(left) != len(right):
            return [path or "/"]
        differences: list[str] = []
        for index, (left_item, right_item) in enumerate(zip(left, right, strict=True)):
            differences.extend(
                difference_paths(left_item, right_item, f"{path}/{index}")
            )
        return differences
    return [] if left == right else [path or "/"]


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--strict", action="store_true")
    args = parser.parse_args()
    process = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "-p",
            "evidence-search-contracts",
            "--example",
            "export_schemas",
        ],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    errors: list[str] = []
    if process.returncode != 0:
        errors.append("compiled Rust schema export failed")
        generated: dict[str, Any] = {}
    else:
        generated = json.loads(process.stdout)

    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    paths = {entry["id"]: entry["schema"] for entry in catalog["entries"]}
    mismatches: dict[str, list[str]] = {}
    compared_schemas: dict[str, dict[str, str]] = {}
    for contract_id, generated_schema in generated.items():
        schema_path = paths.get(contract_id)
        if schema_path is None:
            errors.append(f"{contract_id}: missing canonical catalogue entry")
            continue
        canonical = json.loads((ROOT / schema_path).read_text(encoding="utf-8"))
        generated_semantics = canonicalise(generated_schema)
        canonical_semantics = canonicalise(canonical)
        compared_schemas[contract_id] = {
            "canonical_sha256": semantic_digest(canonical),
            "generated_sha256": semantic_digest(generated_schema),
        }
        if generated_semantics != canonical_semantics:
            mismatches[contract_id] = difference_paths(
                generated_semantics, canonical_semantics
            )

    parity_report = {
        "schema_version": "org.searchright.rust-schema-parity-report.v1",
        "authority": {
            "canonical_validation": "contracts/json-schema/*.schema.json",
            "rust_generated_role": "compiled_drift_diagnostic",
            "binding_generation": "canonical_json_schema",
        },
        "registered_roots": len(generated),
        "compared_schemas": compared_schemas,
        "exact_semantic_parity": not mismatches,
        "contracts": {
            contract_id: {
                "difference_count": len(paths),
                "difference_paths": paths,
            }
            for contract_id, paths in sorted(mismatches.items())
        },
        "resolution": (
            "Generated Schemars documents are compiler-backed drift diagnostics, not "
            "substitutes for the canonical JSON Schemas. Every observed validation-shape "
            "difference is recorded and exact semantic parity remains fail-closed."
        ),
        "claim_boundary": (
            "The report proves that compiled generated roots were compared and all observed "
            "differences were recorded. It does not prove equivalence of JSON Schema and Rust "
            "cross-field validation or downstream consumer compatibility."
        ),
    }
    expected = json.dumps(parity_report, indent=2, sort_keys=True) + "\n"
    stale = not REPORT.is_file() or REPORT.read_text(encoding="utf-8") != expected
    if args.write and not errors:
        REPORT.write_text(expected, encoding="utf-8")
        stale = False
    failed = bool(errors or (args.strict and mismatches) or (args.check and stale))
    receipt = {
        "schema_version": "org.searchright.rust-schema-parity-receipt.v1",
        "status": "failed" if failed else "passed_with_recorded_differences",
        "mode": "write" if args.write else "check" if args.check else "strict",
        "registered_roots": len(generated),
        "exact_semantic_matches": len(generated) - len(mismatches),
        "contracts_with_recorded_differences": len(mismatches),
        "report_sha256": hashlib.sha256(expected.encode("utf-8")).hexdigest(),
        "stale": stale,
        "errors": errors,
        "claim_boundary": (
            "Equality ignores annotation-only JSON Schema keys but retains validation "
            "keywords. Runtime cross-field validation remains separate."
        ),
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
