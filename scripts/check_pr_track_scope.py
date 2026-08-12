#!/usr/bin/env python3
"""Fail closed unless a pull request is one track or a justified exception."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

TRACK_LINE = re.compile(r"(?im)^-\s*Conductor track:\s*`?([^`\r\n]+)`?\s*$")
EXCEPTION_TRACKS_LINE = re.compile(
    r"(?im)^-\s*Exception tracks \(only when absolutely inseparable\):\s*`?([^`\r\n]+)`?\s*$"
)
RATIONALE_LINE = re.compile(
    r"(?im)^-\s*Why the work cannot be split \(required for an exception\):\s*`?([^`\r\n]+)`?\s*$"
)
TRACK_PATH = re.compile(r"^conductor/tracks/(\d{2})-")
TRACK_ID = re.compile(r"^(?:0\d|[12]\d|3[0-7])$")
EXCEPTION_LABEL = "scope:multi-track-exception"


def value(pattern: re.Pattern[str], body: str) -> str | None:
    match = pattern.search(body)
    return match.group(1).strip() if match else None


def changed_files(payload: Any) -> list[str]:
    pages = payload if isinstance(payload, list) else [payload]
    if len(pages) == 1 and isinstance(pages[0], list):
        pages = pages[0]
    return [
        str(item.get("filename"))
        for page in pages
        for item in (page if isinstance(page, list) else [])
        if isinstance(item, dict) and item.get("filename")
    ]


def check(event: dict[str, Any], files: list[str]) -> dict[str, Any]:
    pull_request = event.get("pull_request") or {}
    body = str(pull_request.get("body") or "").replace("\\n", "\n")
    labels = {
        str(item.get("name"))
        for item in pull_request.get("labels", [])
        if isinstance(item, dict) and item.get("name")
    }
    declared = value(TRACK_LINE, body)
    exception_text = value(EXCEPTION_TRACKS_LINE, body)
    rationale = value(RATIONALE_LINE, body)
    path_tracks = {
        match.group(1)
        for path in files
        if (match := TRACK_PATH.match(path)) is not None
    }
    errors: list[str] = []
    admitted_tracks: set[str] = set()

    if declared and TRACK_ID.fullmatch(declared):
        admitted_tracks = {declared}
        if EXCEPTION_LABEL in labels:
            errors.append(f"{EXCEPTION_LABEL} is not permitted on a single-track PR")
        if exception_text not in {None, "none"}:
            errors.append("single-track PR must leave exception tracks as `none`")
        if rationale not in {None, "none"}:
            errors.append("single-track PR must leave the exception rationale as `none`")
    elif declared == "MULTI":
        candidates = {
            item.strip()
            for item in (exception_text or "").split(",")
            if item.strip()
        }
        invalid = sorted(item for item in candidates if not TRACK_ID.fullmatch(item))
        if invalid or len(candidates) < 2:
            errors.append("MULTI requires at least two comma-separated Track IDs 00-37")
        else:
            admitted_tracks = candidates
        if EXCEPTION_LABEL not in labels:
            errors.append(f"MULTI requires the {EXCEPTION_LABEL} label")
        if not rationale or rationale == "none" or len(rationale) < 30:
            errors.append("MULTI requires a concrete split-failure rationale of at least 30 characters")
    else:
        errors.append("Conductor track must be exactly one Track ID 00-37 or `MULTI`")

    undeclared = sorted(path_tracks - admitted_tracks)
    if undeclared:
        errors.append(f"changed Conductor track paths are undeclared: {undeclared}")
    if len(path_tracks) > 1 and declared != "MULTI":
        errors.append(f"PR changes multiple Conductor track paths: {sorted(path_tracks)}")

    return {
        "schema_version": "org.searchright.pr-track-scope-check.v1",
        "status": "failed" if errors else "passed",
        "declared_track": declared,
        "admitted_tracks": sorted(admitted_tracks),
        "changed_track_paths": sorted(path_tracks),
        "exception_label_present": EXCEPTION_LABEL in labels,
        "errors": errors,
        "claim_boundary": (
            "This check enforces declared PR scope. It does not establish implementation, "
            "evidence, review or merge success."
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event", type=Path, required=True)
    parser.add_argument("--files-json", type=Path, required=True)
    args = parser.parse_args()
    event = json.loads(args.event.read_text(encoding="utf-8"))
    files = changed_files(json.loads(args.files_json.read_text(encoding="utf-8")))
    receipt = check(event, files)
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
