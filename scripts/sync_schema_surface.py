#!/usr/bin/env python3
"""Freeze the current contract/interface surface for explicit compatibility review."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts" / "schema-catalog.json"
BASELINE = ROOT / "contracts" / "compatibility" / "schema-surface-0.1.0-alpha.1.json"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render() -> dict:
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    schemas = []
    for entry in sorted(catalog["entries"], key=lambda item: item["id"]):
        schema_path = ROOT / entry["schema"]
        schemas.append(
            {
                "id": entry["id"],
                "schema_id": entry["schema_id"],
                "path": entry["schema"],
                "sha256": sha256(schema_path),
                "stability": entry["stability"],
                "owner_track": entry["owner_track"],
            }
        )
    interfaces = []
    for relative in [
        "contracts/wit/search-provider.wit",
        "contracts/openapi/searchright-http.openapi.yaml",
        "server.json",
    ]:
        path = ROOT / relative
        interfaces.append({"path": relative, "sha256": sha256(path)})
    return {
        "schema_version": "org.searchright.contract-surface-baseline.v1",
        "surface_version": "0.1.0-alpha.1",
        "source_epoch": "2026-08-08",
        "automatic_updates": False,
        "schemas": schemas,
        "interfaces": interfaces,
        "change_policy": {
            "alpha": "Every hash, identifier, removal or addition requires an explicit baseline update and compatibility note.",
            "stable": "Breaking changes require a major version, migration fixture and downstream consumer evidence.",
        },
        "claim_boundary": "Hash equality detects exact surface drift; semantic compatibility still requires schema, API and consumer-specific analysis.",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.write and args.check:
        parser.error("choose --write or --check")
    expected = json.dumps(render(), indent=2, ensure_ascii=False) + "\n"
    stale = not BASELINE.is_file() or BASELINE.read_text(encoding="utf-8") != expected
    if args.write:
        BASELINE.write_text(expected, encoding="utf-8")
        stale = False
    receipt = {
        "schema_version": "org.searchright.contract-surface-sync-receipt.v1",
        "status": "failed" if (args.check and stale) else "passed",
        "mode": "write" if args.write else "check" if args.check else "inspect",
        "stale": stale,
        "schemas": len(render()["schemas"]),
        "interfaces": len(render()["interfaces"]),
        "limitations": ["Exact-byte baseline only; semantic and downstream compatibility require separate evidence."],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if args.check and stale else 0


if __name__ == "__main__":
    raise SystemExit(main())
