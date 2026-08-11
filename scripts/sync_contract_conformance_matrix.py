#!/usr/bin/env python3
"""Generate the static contract conformance matrix from canonical catalogues."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts" / "schema-catalog.json"
SURFACE = ROOT / "contracts" / "compatibility" / "schema-surface-0.1.0-alpha.1.json"
SDK_MANIFEST = ROOT / "sdk" / "manifest.json"
MATRIX = ROOT / "contracts" / "compatibility" / "contract-conformance-matrix.json"
RUST_SCHEMA_SOURCE = ROOT / "crates" / "evidence-search-contracts" / "src" / "schema.rs"


def rust_owned_ids() -> set[str]:
    """Read the explicit Rust-owned root registry without treating every DTO as generated."""
    source = RUST_SCHEMA_SOURCE.read_text(encoding="utf-8")
    return set(re.findall(r'entry::<[^>]+>\(\s*"([^"]+)"', source))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render() -> dict:
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    surface = json.loads(SURFACE.read_text(encoding="utf-8"))
    sdk = json.loads(SDK_MANIFEST.read_text(encoding="utf-8"))
    baselines = {entry["id"]: entry for entry in surface["schemas"]}
    rust_roots = rust_owned_ids()

    contracts = []
    for entry in sorted(catalog["entries"], key=lambda item: item["id"]):
        schema_path = ROOT / entry["schema"]
        example_path = ROOT / entry["example"]
        baseline = baselines.get(entry["id"])
        digest = sha256(schema_path) if schema_path.is_file() else None
        contracts.append(
            {
                "id": entry["id"],
                "owner_track": entry["owner_track"],
                "stability": entry["stability"],
                "schema": {
                    "path": entry["schema"],
                    "present": schema_path.is_file(),
                    "catalogued_id": entry["schema_id"],
                    "canonical_digest_matches": bool(
                        baseline and digest == baseline.get("sha256")
                    ),
                },
                "example": {
                    "path": entry["example"],
                    "present": example_path.is_file(),
                    "static_validation": entry.get("evidence_level")
                    == "static_verified",
                },
                "rust": {
                    "declared_type": entry["rust_type"],
                    "compiler_conformance": (
                        "root_field_parity" if entry["id"] in rust_roots else "not_evidenced"
                    ),
                    "round_trip_conformance": "not_evidenced",
                },
                "downstream_consumer_conformance": "not_evidenced",
            }
        )

    targets = {
        target["language"]: {
            "package": target["package"],
            "status": target["status"],
        }
        for target in sorted(sdk["targets"], key=lambda item: item["language"])
    }
    return {
        "schema_version": "org.searchright.contract-conformance-matrix.v1",
        "surface_version": surface["surface_version"],
        "contracts": contracts,
        "bindings": targets,
        "binding_ownership": {
            "track": "35",
            "requirement": "SR-086",
            "policy": sdk["generation_policy"],
        },
        "rust_schema_parity": {
            "registered_roots": len(rust_roots),
            "exact_parity": False,
            "loss_report": "evidence_search_contracts::rust_schema_parity_report"
        },
        "claim_boundary": (
            "This matrix proves catalogue presence, declared static validation, and "
            "exact canonical digest equality only. It does not prove Rust compiler, "
            "round-trip, generated binding, install, or downstream conformance."
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()

    rendered = render()
    expected = json.dumps(rendered, indent=2, ensure_ascii=False) + "\n"
    stale = not MATRIX.is_file() or MATRIX.read_text(encoding="utf-8") != expected
    errors = []
    for contract in rendered["contracts"]:
        if not contract["schema"]["present"]:
            errors.append(f"{contract['id']}: schema missing")
        if not contract["example"]["present"]:
            errors.append(f"{contract['id']}: example missing")
        if not contract["schema"]["canonical_digest_matches"]:
            errors.append(f"{contract['id']}: canonical digest mismatch")
        if not contract["example"]["static_validation"]:
            errors.append(f"{contract['id']}: static validation is not declared")
    if args.write:
        MATRIX.write_text(expected, encoding="utf-8")
        stale = False
    failed = bool(errors or (args.check and stale))
    receipt = {
        "schema_version": "org.searchright.contract-conformance-matrix-receipt.v1",
        "mode": "write" if args.write else "check" if args.check else "inspect",
        "status": "failed" if failed else "passed",
        "stale": stale,
        "contracts": len(rendered["contracts"]),
        "errors": errors,
        "limitations": [
            "Static matrix only; compiler, round-trip, binding, install and downstream evidence remain separate."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
