#!/usr/bin/env python3
"""Fixture-first CHHHS publication intelligence demonstration.

SearchRight remains the provider-execution boundary. This module handles only
institutional attribution, conservative record linkage, transparent thematic
classification, incremental state and monthly rendering.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import html
import json
import os
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Any, Iterable

ROOT = Path(__file__).resolve().parent
DEFAULT_CONFIG = ROOT / "config.json"
DEFAULT_FIXTURE = ROOT / "fixtures" / "records.json"
SCHEMA_VERSION = "org.searchright.chhhs-research-demo.v1"


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def dump_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def normalise_space(value: str | None) -> str:
    return " ".join((value or "").split())


def normalise_key(value: str | None) -> str:
    text = normalise_space(value).casefold()
    return re.sub(r"[^a-z0-9]+", " ", text).strip()


def normalise_doi(value: str | None) -> str | None:
    if not value:
        return None
    doi = value.strip().casefold()
    doi = re.sub(r"^https?://(?:dx\.)?doi\.org/", "", doi)
    doi = re.sub(r"^doi:\s*", "", doi)
    return doi or None


def title_fingerprint(title: str) -> str:
    return hashlib.sha256(normalise_key(title).encode()).hexdigest()[:24]


def record_key(record: dict[str, Any]) -> str:
    doi = normalise_doi(record.get("doi"))
    return f"doi:{doi}" if doi else f"title:{title_fingerprint(record.get('title', ''))}"


def parse_year(value: str | None) -> int | None:
    if not value:
        return None
    match = re.match(r"^(\d{4})", value)
    return int(match.group(1)) if match else None


@dataclass(frozen=True)
class Attribution:
    status: str
    score: float
    evidence: tuple[str, ...]


def _contains_phrase(text: str, phrase: str) -> bool:
    return bool(re.search(rf"\b{re.escape(normalise_key(phrase))}\b", normalise_key(text)))


def attribute(record: dict[str, Any], config: dict[str, Any]) -> Attribution:
    institution = config["institution"]
    affiliations = [normalise_space(item) for item in record.get("affiliations", [])]
    affiliation_text = " | ".join(affiliations)
    identifiers = {normalise_key(item) for item in record.get("institution_ids", [])}
    evidence: list[str] = []

    approved_ids = {normalise_key(item) for item in institution.get("identifiers", []) if item}
    identifier_hits = sorted(approved_ids & identifiers)
    if identifier_hits:
        evidence.extend(f"institution_id:{item}" for item in identifier_hits)
        return Attribution("confirmed", 1.0, tuple(evidence))

    strong_aliases = institution.get("aliases", []) + institution.get("facilities", [])
    strong_hits = sorted(
        alias for alias in strong_aliases if _contains_phrase(affiliation_text, alias)
    )
    if strong_hits:
        evidence.extend(f"affiliation_alias:{item}" for item in strong_hits)
        return Attribution("confirmed", 0.95, tuple(evidence))

    abbreviation_hits = sorted(
        alias
        for alias in institution.get("abbreviations", [])
        if _contains_phrase(affiliation_text, alias)
    )
    if abbreviation_hits and "queensland" in normalise_key(affiliation_text):
        evidence.extend(f"qualified_abbreviation:{item}" for item in abbreviation_hits)
        return Attribution("probable", 0.78, tuple(evidence))

    combined = " ".join(
        [record.get("title", ""), record.get("abstract", ""), record.get("acknowledgements", "")]
    )
    mention_hits = sorted(alias for alias in strong_aliases if _contains_phrase(combined, alias))
    if mention_hits:
        evidence.extend(f"non_affiliation_mention:{item}" for item in mention_hits)
        return Attribution("review_required", 0.45, tuple(evidence))

    if "cairns" in normalise_key(combined + " " + affiliation_text):
        return Attribution("insufficient_evidence", 0.10, ("geographic_cairns_only",))
    return Attribution("insufficient_evidence", 0.0, ())


def classify(record: dict[str, Any], config: dict[str, Any]) -> dict[str, Any]:
    text = normalise_key(f"{record.get('title', '')} {record.get('abstract', '')}")
    themes: list[dict[str, Any]] = []
    for theme in config["taxonomy"]["themes"]:
        hits = sorted(term for term in theme["terms"] if normalise_key(term) in text)
        if hits:
            themes.append({"id": theme["id"], "label": theme["label"], "matched_terms": hits})

    study_type = "other"
    study_hits: list[str] = []
    for candidate in config["taxonomy"]["study_types"]:
        hits = sorted(term for term in candidate["terms"] if normalise_key(term) in text)
        if hits:
            study_type = candidate["id"]
            study_hits = hits
            break
    return {
        "taxonomy_version": config["taxonomy"]["version"],
        "themes": themes,
        "study_type": study_type,
        "study_type_terms": study_hits,
    }


def normalise_record(raw: dict[str, Any], retrieved_at: str) -> dict[str, Any]:
    source = normalise_space(raw.get("source") or "unknown").casefold()
    return {
        "title": normalise_space(raw.get("title")),
        "abstract": normalise_space(raw.get("abstract")),
        "doi": normalise_doi(raw.get("doi")),
        "published": normalise_space(raw.get("published")) or None,
        "authors": [normalise_space(item) for item in raw.get("authors", []) if normalise_space(item)],
        "affiliations": [
            normalise_space(item) for item in raw.get("affiliations", []) if normalise_space(item)
        ],
        "institution_ids": [normalise_space(item) for item in raw.get("institution_ids", [])],
        "acknowledgements": normalise_space(raw.get("acknowledgements")),
        "url": normalise_space(raw.get("url")) or None,
        "sources": [source],
        "source_record_ids": {
            source: normalise_space(raw.get("source_record_id")) or record_key(raw)
        },
        "first_seen": retrieved_at,
        "last_seen": retrieved_at,
    }


def merge_records(records: Iterable[dict[str, Any]]) -> list[dict[str, Any]]:
    merged: dict[str, dict[str, Any]] = {}
    for record in records:
        key = record_key(record)
        if key not in merged:
            merged[key] = dict(record)
            continue
        current = merged[key]
        current["sources"] = sorted(set(current.get("sources", [])) | set(record.get("sources", [])))
        current.setdefault("source_record_ids", {}).update(record.get("source_record_ids", {}))
        current["affiliations"] = sorted(
            set(current.get("affiliations", [])) | set(record.get("affiliations", []))
        )
        current["institution_ids"] = sorted(
            set(current.get("institution_ids", [])) | set(record.get("institution_ids", []))
        )
        if len(record.get("abstract", "")) > len(current.get("abstract", "")):
            current["abstract"] = record["abstract"]
        current["first_seen"] = min(current["first_seen"], record["first_seen"])
        current["last_seen"] = max(current["last_seen"], record["last_seen"])
    return [merged[key] for key in sorted(merged)]


def read_adapter_output(output: str) -> list[dict[str, Any]]:
    text = output.strip()
    if not text:
        return []
    try:
        value = json.loads(text)
    except json.JSONDecodeError:
        return [json.loads(line) for line in text.splitlines() if line.strip()]
    if isinstance(value, list):
        return value
    if isinstance(value, dict) and isinstance(value.get("records"), list):
        return value["records"]
    raise ValueError("SearchRight adapter output must be a record array, JSON Lines, or {records:[...]}")


def execute_searchright(command: str, config: dict[str, Any], state: dict[str, Any]) -> list[dict[str, Any]]:
    argv = shlex.split(command)
    if not argv:
        raise ValueError("CHHHS_SEARCHRIGHT_COMMAND is empty")
    request = {
        "schema_version": SCHEMA_VERSION,
        "operation": "institutional_publication_search",
        "institution": config["institution"],
        "providers": config["providers"],
        "watermark": state.get("watermark"),
        "network_authority": "delegated_to_searchright",
    }
    completed = subprocess.run(
        argv,
        input=json.dumps(request),
        text=True,
        capture_output=True,
        check=False,
        timeout=300,
        shell=False,
    )
    if completed.returncode:
        raise RuntimeError(
            f"SearchRight adapter failed with exit {completed.returncode}: "
            f"{completed.stderr.strip()[:1000]}"
        )
    return read_adapter_output(completed.stdout)


def update_state(
    raw_records: list[dict[str, Any]],
    config: dict[str, Any],
    state_path: Path,
    retrieved_at: str,
) -> dict[str, Any]:
    existing = load_json(state_path) if state_path.exists() else {"records": [], "runs": []}
    incoming = [normalise_record(item, retrieved_at) for item in raw_records]
    all_records = merge_records([*existing.get("records", []), *incoming])
    for record in all_records:
        decision = attribute(record, config)
        record["attribution"] = {
            "status": decision.status,
            "score": decision.score,
            "evidence": list(decision.evidence),
            "rules_version": config["institution"]["version"],
        }
        record["classification"] = classify(record, config)
    state = {
        "schema_version": SCHEMA_VERSION,
        "watermark": retrieved_at,
        "records": all_records,
        "runs": [
            *existing.get("runs", []),
            {
                "retrieved_at": retrieved_at,
                "input_records": len(raw_records),
                "unique_records": len(all_records),
                "providers": config["providers"],
            },
        ],
    }
    dump_json(state_path, state)
    return state


def month_matches(record: dict[str, Any], month: str) -> bool:
    published = record.get("published") or ""
    return published.startswith(month)


def render_report(state: dict[str, Any], month: str, output_dir: Path) -> dict[str, Path]:
    if not re.fullmatch(r"\d{4}-\d{2}", month):
        raise ValueError("month must use YYYY-MM")
    output_dir.mkdir(parents=True, exist_ok=True)
    records = [record for record in state.get("records", []) if month_matches(record, month)]
    accepted = [
        record
        for record in records
        if record["attribution"]["status"] in {"confirmed", "probable", "review_required"}
    ]
    accepted.sort(key=lambda item: (item.get("published") or "", item.get("title") or ""))

    report = {
        "schema_version": SCHEMA_VERSION,
        "month": month,
        "generated_from_watermark": state.get("watermark"),
        "candidate_records": len(records),
        "included_candidates": len(accepted),
        "counts_by_attribution": {
            status: sum(1 for item in records if item["attribution"]["status"] == status)
            for status in ["confirmed", "probable", "review_required", "insufficient_evidence"]
        },
        "records": accepted,
        "claim_boundary": (
            "High-recall candidate monitor only; not an exhaustive or authoritative CHHHS register."
        ),
    }
    json_path = output_dir / f"chhhs-research-{month}.json"
    csv_path = output_dir / f"chhhs-research-{month}.csv"
    html_path = output_dir / f"chhhs-research-{month}.html"
    dump_json(json_path, report)

    with csv_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["title", "published", "doi", "attribution", "score", "sources", "themes"],
        )
        writer.writeheader()
        for record in accepted:
            writer.writerow(
                {
                    "title": record["title"],
                    "published": record.get("published") or "",
                    "doi": record.get("doi") or "",
                    "attribution": record["attribution"]["status"],
                    "score": record["attribution"]["score"],
                    "sources": "; ".join(record["sources"]),
                    "themes": "; ".join(
                        theme["label"] for theme in record["classification"]["themes"]
                    ),
                }
            )

    rows = []
    for record in accepted:
        themes = ", ".join(theme["label"] for theme in record["classification"]["themes"]) or "Other"
        rows.append(
            "<tr>"
            f"<td>{html.escape(record['title'])}</td>"
            f"<td>{html.escape(record.get('published') or '')}</td>"
            f"<td>{html.escape(record['attribution']['status'])}</td>"
            f"<td>{html.escape(', '.join(record['sources']))}</td>"
            f"<td>{html.escape(themes)}</td>"
            "</tr>"
        )
    html_path.write_text(
        "<!doctype html><html lang='en'><meta charset='utf-8'>"
        f"<title>CHHHS research monitor {html.escape(month)}</title>"
        "<style>body{font-family:system-ui;max-width:1100px;margin:2rem auto;padding:0 1rem}"
        "table{border-collapse:collapse;width:100%}th,td{border:1px solid #bbb;padding:.5rem;vertical-align:top}"
        "th{text-align:left}aside{background:#f3f3f3;padding:1rem;margin:1rem 0}</style>"
        f"<h1>CHHHS research intelligence: {html.escape(month)}</h1>"
        f"<p>{len(accepted)} included candidates from {len(records)} dated records.</p>"
        f"<aside>{html.escape(report['claim_boundary'])}</aside>"
        "<table><thead><tr><th>Title</th><th>Date</th><th>Attribution</th>"
        "<th>Sources</th><th>Themes</th></tr></thead><tbody>"
        + "".join(rows)
        + "</tbody></table></html>\n",
        encoding="utf-8",
    )
    return {"json": json_path, "csv": csv_path, "html": html_path}


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    subparsers = parser.add_subparsers(dest="command", required=True)

    update = subparsers.add_parser("update")
    update.add_argument("--state", type=Path, default=ROOT / "state.json")
    update.add_argument("--fixture", action="store_true")
    update.add_argument("--retrieved-at", default=None)
    update.add_argument("--adapter-command", default=None)

    report = subparsers.add_parser("report")
    report.add_argument("--state", type=Path, default=ROOT / "state.json")
    report.add_argument("--month", required=True)
    report.add_argument("--output-dir", type=Path, default=ROOT / "output")

    run = subparsers.add_parser("run")
    run.add_argument("--state", type=Path, default=ROOT / "state.json")
    run.add_argument("--fixture", action="store_true")
    run.add_argument("--retrieved-at", default="2026-08-31T00:00:00+00:00")
    run.add_argument("--adapter-command", default=None)
    run.add_argument("--month", default=date.today().strftime("%Y-%m"))
    run.add_argument("--output-dir", type=Path, default=ROOT / "output")
    return parser


def perform_update(args: argparse.Namespace, config: dict[str, Any]) -> dict[str, Any]:
    retrieved_at = args.retrieved_at or utc_now()
    if args.fixture:
        raw_records = load_json(DEFAULT_FIXTURE)
    else:
        command = args.adapter_command or os.environ.get("CHHHS_SEARCHRIGHT_COMMAND")
        if not command:
            raise RuntimeError(
                "Live update requires --adapter-command or CHHHS_SEARCHRIGHT_COMMAND; "
                "use --fixture for the rights-clear demonstration."
            )
        previous = load_json(args.state) if args.state.exists() else {}
        raw_records = execute_searchright(command, config, previous)
    return update_state(raw_records, config, args.state, retrieved_at)


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    config = load_json(args.config)
    if args.command == "update":
        state = perform_update(args, config)
        print(json.dumps({"status": "updated", "records": len(state["records"])}))
        return 0
    if args.command == "report":
        paths = render_report(load_json(args.state), args.month, args.output_dir)
        print(json.dumps({key: str(value) for key, value in paths.items()}, sort_keys=True))
        return 0
    if args.command == "run":
        state = perform_update(args, config)
        paths = render_report(state, args.month, args.output_dir)
        print(json.dumps({key: str(value) for key, value in paths.items()}, sort_keys=True))
        return 0
    raise AssertionError(args.command)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(2) from exc
