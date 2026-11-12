#!/usr/bin/env python3
"""Offline referential integrity for draft Searchright research artifacts.

This does not replace upstream CSL JSON Schema validation or certify any study.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


def validate_links(references: Any, protocol: dict[str, Any], publication: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if not isinstance(references, list):
        return ["canonical CSL bibliography must be an array"]
    ids = [record.get("id") for record in references if isinstance(record, dict)]
    if len(ids) != len(references) or any(not isinstance(i, str) or not i for i in ids):
        errors.append("canonical references require nonempty string CSL IDs")
    valid = [i for i in ids if isinstance(i, str)]
    if len(valid) != len(set(valid)):
        errors.append("duplicate CSL identifiers")
    for ref in protocol.get("references", []):
        if ref not in valid:
            errors.append(f"unresolved CSL reference: {ref}")
    if publication.get("protocol_id") != protocol.get("protocol_id"):
        errors.append("publication protocol identity mismatch")
    if publication.get("protocol_version") != protocol.get("version"):
        errors.append("publication protocol version mismatch")
    if publication.get("canonical_references") != "research/references.csl.json":
        errors.append("publication must bind the canonical CSL bibliography")
    registration = protocol.get("registration", {})
    if registration.get("status") == "registered" and not registration.get("identifier"):
        errors.append("registered protocol requires an actual registration identifier")
    if publication.get("status") != "draft" and not publication.get("results_evidence"):
        errors.append("publication outside draft requires result evidence")
    return errors


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    try:
        refs = json.loads((root / "research/references.csl.json").read_text(encoding="utf-8"))
        protocol = json.loads((root / "research/protocols/R7.protocol.json").read_text(encoding="utf-8"))
        publication = json.loads((root / "research/publications/R7.publication.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        print(json.dumps({"status": "failed", "errors": [str(exc)]}))
        return 1
    errors = validate_links(refs, protocol, publication)
    print(json.dumps({"status": "failed" if errors else "passed", "errors": errors}, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
