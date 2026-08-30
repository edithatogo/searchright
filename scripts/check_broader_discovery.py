#!/usr/bin/env python3
"""Validate Track 20 source-method and citation fixtures without network access."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts/fixtures/discovery-source-methods.json"
OPEN_CITATIONS = ROOT / "contracts/fixtures/opencitations-forward.json"
DOC = ROOT / "docs/supplementary-discovery.md"

REQUIRED_SOURCES = {
    "clinicaltrials-gov",
    "who-ictrp",
    "anzctr",
    "osf",
    "zenodo",
    "figshare",
    "dataverse",
    "institutional-repositories",
    "conference-search",
    "thesis-search",
    "policy-search",
    "organisational-websites",
    "opencitations",
    "backward-reference-checking",
    "contact-log",
    "handsearch-log",
}


def main() -> int:
    errors: list[str] = []
    methods = json.loads(CATALOG.read_text(encoding="utf-8"))
    citations = json.loads(OPEN_CITATIONS.read_text(encoding="utf-8"))
    documentation = DOC.read_text(encoding="utf-8")

    source_ids = [item.get("source_id") for item in methods]
    if set(source_ids) != REQUIRED_SOURCES:
        errors.append("source catalogue must contain every required source exactly")
    if len(source_ids) != len(set(source_ids)):
        errors.append("source identifiers must be unique")
    for item in methods:
        source_id = item.get("source_id", "<missing>")
        if not str(item.get("procedure", "")).strip():
            errors.append(f"{source_id}: procedure is blank")
        limitations = item.get("limitations")
        if not isinstance(limitations, list) or not limitations:
            errors.append(f"{source_id}: limitations are required")
        if item.get("access_mode") == "fixture_adapter" and not item.get(
            "live_opt_in_required"
        ):
            errors.append(f"{source_id}: fixture adapter must require live opt-in")
    citation_methods = {
        item.get("method")
        for item in methods
        if item.get("source_kind") == "citation_index"
    }
    if citation_methods != {"backward_citation", "forward_citation"}:
        errors.append("citation methods must cover bounded backward and forward chaining")

    citing = {
        identifier
        for row in citations
        for identifier in str(row.get("citing", "")).split()
        if identifier
    }
    if len(citing) != 3:
        errors.append("OpenCitations fixture must contain three unique citing records")

    for phrase in (
        "does not claim",
        "Do not scrape",
        "human release",
        "simulation instrument",
    ):
        if phrase not in documentation:
            errors.append(f"documentation is missing required boundary: {phrase}")

    result = {
        "schema_version": "org.searchright.broader-discovery-check.v1",
        "status": "failed" if errors else "passed",
        "sources_checked": len(methods),
        "citation_fixture_records": len(citing),
        "network_operations": 0,
        "external_writes": 0,
        "errors": errors,
        "claim_boundary": (
            "Deterministic source-method and citation fixtures only; no live source, "
            "exhaustiveness, legal approval, or methodological adequacy claim."
        ),
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
