#!/usr/bin/env python3
"""Read-only status for Searchright's evidence-aware Conductor v3 dialect."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


def read(root: Path, relative: str) -> str:
    path = (root / relative).resolve()
    if not path.is_relative_to(root) or Path(relative).is_absolute():
        raise ValueError(f"unsafe local path: {relative}")
    text = path.read_text(encoding="utf-8")
    if not text.strip():
        raise ValueError(f"empty file: {relative}")
    return text


def audit(root: Path) -> dict:
    root = root.resolve()
    errors: list[str] = []
    tracks: list[dict] = []
    result = {
        "schema_version": "org.searchright.conductor-status.v1",
        "root": str(root), "tracks": tracks, "errors": errors,
        "isolation": {"state": "unconfigured", "ownership": "not_inferred"},
        "limitations": [
            "Native v3 registry and in-place lifecycle; not the generic skill's metadata dialect.",
            "Integrity is not compiler, hosted, human, adoption, release or publication evidence.",
            "No network calls, GitHub mutations, task completion or evidence promotion.",
        ],
    }
    try:
        index = read(root, "conductor/index.md")
        links = re.findall(r"^- \[([^]]+)\]\(([^)]+)\)", index, re.M)
        by_label = dict(links)
        if len(by_label) != len(links):
            errors.append("duplicate project index labels")
        for label in ("Product Definition", "Product Guidelines", "Tech Stack", "Workflow", "Tracks Registry"):
            if label not in by_label:
                raise ValueError(f"missing project index link: {label}")
            read(root, "conductor/" + by_label[label])
        if (root / "conductor" / by_label["Tracks Registry"]).resolve() != root / "conductor/tracks.md":
            raise ValueError("unsupported noncanonical registry target")
        workflow = read(root, "conductor/" + by_label["Workflow"])
        if re.search(r"isolation\s+mode\s*:\s*worktree", workflow, re.I):
            result["isolation"]["state"] = "inconsistent"
            errors.append("worktree isolation configured: exact lease validation is required separately")
        coverage = json.loads(read(root, "conductor/roadmap-coverage.json"))["tracks"]
        entries = {entry["track_id"]: entry for entry in coverage}
        if len(entries) != len(coverage):
            errors.append("duplicate roadmap track IDs")
        registry = read(root, "conductor/tracks.md")
        archived = False
        rows: dict[str, tuple[str, list[str], bool]] = {}
        for line in registry.splitlines():
            if line.startswith("## "):
                archived = line == "## Archived tracks"
            if not re.match(r"^\| \d{2} \|", line):
                continue
            cells = [cell.strip() for cell in line.strip("|").split("|")]
            ident = cells[0]
            match = re.fullmatch(r"\[[^]]+\]\(([^)]+)\)", cells[1])
            if not match or len(cells) != (4 if archived else 7):
                raise ValueError(f"invalid registry row: {ident}")
            if ident in rows:
                errors.append(f"duplicate registry track: {ident}")
            rows[ident] = (match[1], cells, archived)
        if set(rows) != set(entries):
            errors.append("registry/roadmap track IDs differ")
        expected_dirs = {f"{e['track_id']}-{e['slug']}" for e in coverage}
        actual_dirs = {p.name for p in (root / "conductor/tracks").iterdir() if p.is_dir()}
        if expected_dirs != actual_dirs:
            errors.append("track directories/roadmap differ")
        for ident, entry in entries.items():
            try:
                directory = f"conductor/tracks/{ident}-{entry['slug']}"
                target, cells, registry_archived = rows[ident]
                read(root, "conductor/" + target)
                if (root / "conductor" / target).resolve() != root / directory / "spec.md":
                    raise ValueError("registry target differs from canonical specification")
                metadata = json.loads(read(root, directory + "/metadata.json"))
                evidence = json.loads(read(root, directory + "/evidence.json"))
                lifecycle = entry.get("lifecycle", "active")
                if registry_archived != (lifecycle == "archived"):
                    errors.append(f"track {ident}: registry archive disagreement")
                if cells[-1 if registry_archived else 5] != entry["evidence_level"]:
                    errors.append(f"track {ident}: registry evidence disagreement")
                if not registry_archived and (cells[3], cells[4]) != (entry["status"], entry["implementation_state"]):
                    errors.append(f"track {ident}: registry status disagreement")
                for record in (metadata, evidence):
                    for key in ("track_id", "status", "implementation_state", "evidence_level"):
                        if record.get(key) != entry.get(key):
                            errors.append(f"track {ident}: {key} disagreement")
                    if record.get("lifecycle", "active") != lifecycle:
                        errors.append(f"track {ident}: lifecycle disagreement")
                plan = re.sub(r"```.*?```", "", read(root, directory + "/plan.md"), flags=re.S)
                tasks = re.findall(r"^- \[([ x~])\] (.+)$", plan, re.M)
                if not tasks:
                    errors.append(f"track {ident}: no top-level tasks")
                done = sum(state == "x" for state, _ in tasks)
                if lifecycle == "archived" and (done != len(tasks) or entry.get("blockers")):
                    errors.append(f"track {ident}: archived track retains open work")
                ledger = "not_opted_in"
                if metadata.get("evidence_schema") is not None:
                    ledger = "requires_schema_specific_validation"
                    errors.append(f"track {ident}: opted-in ledger requires schema-specific validation")
                elif (root / directory / "evidence.jsonl").exists():
                    ledger = "present_without_optin"
                    errors.append(f"track {ident}: ledger exists without schema opt-in")
                tracks.append({
                    "track_id": ident, "status": entry["status"], "lifecycle": lifecycle,
                    "evidence_level": entry["evidence_level"], "completed_tasks": done,
                    "total_tasks": len(tasks), "next_task": next((text for state, text in tasks if state != "x"), None),
                    "blockers": entry.get("blockers", []), "gates": metadata.get("gates", []),
                    "ledger": ledger,
                })
            except (OSError, ValueError, KeyError, TypeError) as exc:
                errors.append(f"track {ident}: {exc}")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        errors.append(str(exc))
    result["counts"] = {
        "tracks": len(tracks), "archived": sum(t["lifecycle"] == "archived" for t in tracks),
        "active": sum(t["lifecycle"] == "active" for t in tracks),
        "completed_tasks": sum(t["completed_tasks"] for t in tracks),
        "total_tasks": sum(t["total_tasks"] for t in tracks),
    }
    result["status"] = "failed" if errors else "passed"
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    result = audit(args.root)
    # These existing repository gates only read source when invoked this way.
    result["native_checks"] = []
    for command in (["scripts/check_roadmap_coverage.py"], ["scripts/sync_track_evidence.py", "--check"]):
        try:
            run = subprocess.run([sys.executable, "-B", *command], cwd=args.root, capture_output=True, text=True, check=False)
            exit_code = run.returncode
        except OSError:
            exit_code = 127
        result["native_checks"].append({"command": command, "exit_code": exit_code})
        if exit_code:
            result["errors"].append(f"native check failed: {' '.join(command)}; run directly for diagnostics")
    result["status"] = "failed" if result["errors"] else "passed"
    print(json.dumps(result, indent=2, sort_keys=True))
    return int(bool(result["errors"]))


if __name__ == "__main__":
    raise SystemExit(main())
