#!/usr/bin/env python3
"""Verify an explicitly supplied, pinned StandardFlow EQUATOR directory offline."""
import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REF = ROOT / "integration/candidates/equator-directory-20261123.json"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
PATH = re.compile(r"^reporting-guidelines/([^/]+)/index\.qmd$")


def git_blob_sha(data: bytes) -> str:
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


def validate(reference: dict, content: bytes) -> None:
    if len(content) > 5_000_000 or git_blob_sha(content) != reference["producer_blob_sha"]:
        raise ValueError("catalogue bytes do not match the pinned StandardFlow Git blob")
    catalog = json.loads(content)
    if (reference["status"] != "proposed_not_active" or
        reference["auto_promote"] is not False or
        reference["default_network"] is not False or
        reference["default_external_writes"] is not False or
        reference["rights_status"] != "not_assessed_for_individual_guidelines"):
        raise ValueError("unsafe consumer admission or rights status")
    for key, expected in (
        ("schema_version", 1),
        ("source_repository", reference["upstream_repository"]),
        ("source_revision", reference["upstream_revision"]),
        ("source_directory", "reporting-guidelines"),
        ("rights_status", "not_assessed_for_individual_guidelines"),
        ("published_count", reference["expected_published_count"]),
    ):
        if catalog.get(key) != expected:
            raise ValueError(f"catalogue {key} mismatch")
    rows = catalog.get("guidelines")
    if not isinstance(rows, list) or len(rows) != catalog["published_count"]:
        raise ValueError("catalogue row count mismatch")
    seen = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != {
            "id", "page_url", "source_path", "source_blob_sha", "metadata_status"
        }:
            raise ValueError("unexpected row or guideline content")
        m = PATH.fullmatch(row["source_path"]) if isinstance(row["source_path"], str) else None
        if not m or m[1] == "under-development":
            raise ValueError("not a published guideline page")
        slug = m[1]
        if (row["id"] != "equator:" + slug or row["id"] in seen or
            row["page_url"] != "https://equator-network.org/reporting-guidelines/" + slug + "/" or
            row["metadata_status"] != "not_extracted" or
            not isinstance(row["source_blob_sha"], str) or
            not HEX40.fullmatch(row["source_blob_sha"])):
            raise ValueError("invalid source identity, provenance or status")
        seen.add(row["id"])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalogue", type=Path, required=True,
                        help="Local StandardFlow discovery.json (no network access)")
    args = parser.parse_args()
    try:
        ref = json.loads(REF.read_text(encoding="utf-8"))
        validate(ref, args.catalogue.read_bytes())
    except (OSError, ValueError, TypeError, KeyError, json.JSONDecodeError) as exc:
        print(f"equator-directory: rejected: {exc}")
        return 1
    print("equator-directory: pinned source bytes and discovery contract pass; candidate remains inactive")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
