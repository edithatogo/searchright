#!/usr/bin/env python3
"""Run a rights-clear, network-free contract reference vertical slice."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "verification/vertical-slice"


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def normalise_doi(value: str | None) -> str | None:
    if not value:
        return None
    text = value.strip().lower()
    for prefix in ("https://doi.org/", "http://doi.org/", "http://dx.doi.org/", "doi:"):
        if text.startswith(prefix):
            text = text[len(prefix):]
    return text.rstrip(". ") or None


def normalise_title(value: str) -> str:
    return " ".join("".join(ch.lower() if ch.isalnum() else " " for ch in value).split())


def deduplicate(records: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    unique: list[dict[str, Any]] = []
    clusters: list[dict[str, Any]] = []
    keys: dict[tuple[str, str], str] = {}
    for record in records:
        doi = normalise_doi(record.get("identifiers", {}).get("doi"))
        key = ("doi", doi) if doi else ("title", normalise_title(record["title"]))
        canonical = keys.get(key)
        if canonical is None:
            keys[key] = record["record_id"]
            unique.append(record)
        else:
            clusters.append({
                "canonical_record_id": canonical,
                "duplicate_record_id": record["record_id"],
                "basis": key[0],
                "normalised_value": key[1],
            })
    return unique, clusters


def compute_counts(unique: list[dict[str, Any]], decisions: list[dict[str, Any]]) -> dict[str, int]:
    record_ids = {record["record_id"] for record in unique}
    seen_decisions: set[str] = set()
    for decision in decisions:
        if decision.get("reviewer_kind") != "human":
            raise ValueError(f"final decision {decision.get('decision_id')} is not human")
        if decision.get("subject_id") not in record_ids:
            raise ValueError(f"decision references noncanonical or unknown record {decision.get('subject_id')}")
        if decision.get("decision_id") in seen_decisions:
            raise ValueError(f"duplicate decision_id {decision.get('decision_id')}")
        seen_decisions.add(decision["decision_id"])
    title = [d for d in decisions if d["stage"] == "title_abstract"]
    full = [d for d in decisions if d["stage"] == "full_text"]
    title_excluded = sum(d["decision"] == "exclude" for d in title)
    sought = sum(d["decision"] == "include" for d in title)
    full_excluded = sum(d["decision"] == "exclude" for d in full)
    reports_included = sum(d["decision"] == "include" for d in full)
    return {
        "input_records": len(unique),  # caller corrects to pre-dedup count
        "unique_records": len(unique),
        "duplicate_records": 0,
        "title_abstract_excluded": title_excluded,
        "reports_sought": sought,
        "reports_not_retrieved": 0,
        "full_text_excluded": full_excluded,
        "reports_included": reports_included,
        "studies_included": reports_included,
    }


def load_review_bundle_module():
    spec = importlib.util.spec_from_file_location("searchright_review_bundle", ROOT / "scripts/review_bundle.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load review_bundle.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def run(decisions_override: list[dict[str, Any]] | None = None) -> dict[str, Any]:
    errors: list[str] = []
    stages: list[dict[str, Any]] = []
    plan = yaml.safe_load((ROOT / "contracts/examples/review-plan.yaml").read_text(encoding="utf-8"))
    native = load_json(ROOT / "contracts/examples/native-search-strategy.json")
    records = load_json(FIXTURE / "records.json")
    receipts = load_json(FIXTURE / "receipts.json")
    decisions = decisions_override if decisions_override is not None else load_json(FIXTURE / "decisions.json")
    expected = load_json(FIXTURE / "expected.json")

    if plan.get("schema_version") != "org.searchright.review-plan.v1" or not plan.get("question", {}).get("framework"):
        errors.append("review plan is missing its versioned structured question")
    stages.append({"stage": "planning", "status": "passed" if not errors else "failed"})

    if native.get("raw_text") != "".join(f"{line['text']}\n" for line in native.get("lines", [])):
        errors.append("native strategy lines do not reconstruct exact raw text")
    if native.get("normalisation_state") != "raw_only" or native.get("semantic_strategy") is not None:
        errors.append("native strategy must not overclaim semantic normalisation")
    stages.append({"stage": "native_strategy", "status": "passed" if not errors else "failed"})

    receipt_ids = {item["receipt_id"] for item in receipts}
    for record in records:
        if record.get("schema_version") != "org.searchright.bibliographic-record.v1":
            errors.append(f"unexpected record contract for {record.get('record_id')}")
        if record.get("source_receipt_id") not in receipt_ids:
            errors.append(f"unresolved receipt for {record.get('record_id')}")
    unique, clusters = deduplicate(records)
    stages.append({"stage": "records_and_deduplication", "status": "passed" if not errors else "failed", "clusters": len(clusters)})

    try:
        counts = compute_counts(unique, decisions)
        counts["input_records"] = len(records)
        counts["duplicate_records"] = len(clusters)
    except ValueError as exc:
        errors.append(str(exc))
        counts = {}
    if counts and counts != expected:
        errors.append(f"PRISMA/reference counts differ: expected={expected} observed={counts}")
    if counts:
        arithmetic = {
            "records_screened": counts["input_records"] - counts["duplicate_records"],
            "reports_assessed": counts["reports_sought"] - counts["reports_not_retrieved"],
        }
        if arithmetic["records_screened"] != counts["unique_records"]:
            errors.append("records-screened arithmetic failed")
        if arithmetic["reports_assessed"] != counts["full_text_excluded"] + counts["reports_included"]:
            errors.append("full-text arithmetic failed")
    stages.append({"stage": "human_screening_and_prisma", "status": "passed" if not errors else "failed"})

    bundle_digest = None
    if not errors:
        bundle = load_review_bundle_module()
        with tempfile.TemporaryDirectory(prefix="searchright-vertical-slice-") as temp_value:
            temp = Path(temp_value)
            payloads = {
                "plan.yaml": (ROOT / "contracts/examples/review-plan.yaml").read_bytes(),
                "native.json": json.dumps(native, indent=2, sort_keys=True).encode() + b"\n",
                "records.json": json.dumps(records, indent=2, sort_keys=True).encode() + b"\n",
                "dedup.json": json.dumps(clusters, indent=2, sort_keys=True).encode() + b"\n",
                "decisions.json": json.dumps(decisions, indent=2, sort_keys=True).encode() + b"\n",
                "prisma-reference.json": json.dumps(counts, indent=2, sort_keys=True).encode() + b"\n",
            }
            entries = []
            for name, data in payloads.items():
                (temp / name).write_bytes(data)
                entries.append({"source": name, "path": name, "role": name.rsplit('.', 1)[0]})
            plan_value = {
                "schema_version": "org.searchright.review-bundle-plan.v1",
                "bundle_id": "vertical-slice-reference",
                "review_id": plan["review_id"],
                "source_epoch": "2026-08-08",
                "entries": entries,
            }
            plan_path = temp / "bundle-plan.json"
            plan_path.write_bytes(bundle.canonical_json_bytes(plan_value))
            first = temp / "first.srpack"
            second = temp / "second.srpack"
            bundle.pack(plan_path, first, max_files=50, max_total_bytes=10_000_000, max_file_bytes=2_000_000)
            bundle.pack(plan_path, second, max_files=50, max_total_bytes=10_000_000, max_file_bytes=2_000_000)
            if first.read_bytes() != second.read_bytes():
                errors.append("vertical-slice review bundle is not byte deterministic")
            verification = bundle.verify(first)
            if verification["status"] != "passed":
                errors.extend(verification["errors"])
            bundle_digest = hashlib.sha256(first.read_bytes()).hexdigest()
    stages.append({"stage": "portable_bundle", "status": "passed" if not errors else "failed"})

    return {
        "schema_version": "org.searchright.vertical-slice-reference-receipt.v1",
        "status": "failed" if errors else "passed",
        "review_id": plan.get("review_id"),
        "stages": stages,
        "counts": counts,
        "bundle_sha256": bundle_digest,
        "errors": errors,
        "claim_boundary": "This is a network-free contract reference path. It does not prove Rust compilation, live-provider behaviour, search recall, screening validity or methodological adequacy.",
    }


def main() -> int:
    receipt = run()
    # Negative authority check: an agent final decision must be rejected.
    negative = load_json(FIXTURE / "decisions.json")
    negative[0] = dict(negative[0], reviewer_kind="agent")
    if run(negative)["status"] != "failed":
        receipt["errors"].append("agent final-decision negative test did not fail")
        receipt["status"] = "failed"
    receipt["negative_tests"] = ["nonhuman_final_decision_rejected"]
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0 if receipt["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
