#!/usr/bin/env python3
"""Validate prepared, non-mutating companion repository change packets and estate registers."""
from __future__ import annotations

import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
INDEX = ROOT / "migration/companion-repositories/index.json"
ESTATE_REPOSITORIES = ROOT / "migration/estate/repositories.json"
ESTATE_REPLACEMENTS = ROOT / "migration/estate/replacement-decisions.json"
HEX40 = re.compile(r"^[a-f0-9]{40}$")


def validate_estate_repositories(errors: list[str]) -> None:
    if not ESTATE_REPOSITORIES.is_file():
        errors.append(f"missing estate repositories: {ESTATE_REPOSITORIES.name}")
        return
    try:
        data = json.loads(ESTATE_REPOSITORIES.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"unparseable estate repositories: {exc}")
        return
    if data.get("schema_version") != "org.searchright.estate-repositories.v1":
        errors.append("estate repositories: invalid schema version")
    repos = data.get("repositories")
    if not isinstance(repos, list) or not repos:
        errors.append("estate repositories: non-empty repositories required")
        return
    seen: set[str] = set()
    for entry in repos:
        name = entry.get("repository")
        if not isinstance(name, str) or not name.strip():
            errors.append("estate repositories: repository name must be non-empty string")
            continue
        if name in seen:
            errors.append(f"estate repositories: duplicate repository {name}")
        seen.add(name)
        for field in ("role", "decision", "priority", "evidence"):
            val = entry.get(field)
            if not isinstance(val, str) or not val.strip():
                errors.append(f"estate repositories:{name}: missing or empty {field}")
        priority = entry.get("priority")
        if priority not in {"must", "should", "could"}:
            errors.append(f"estate repositories:{name}: invalid priority {priority}")


def validate_estate_decisions(errors: list[str]) -> None:
    if not ESTATE_REPLACEMENTS.is_file():
        errors.append(f"missing estate replacement decisions: {ESTATE_REPLACEMENTS.name}")
        return
    try:
        data = json.loads(ESTATE_REPLACEMENTS.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"unparseable estate replacement decisions: {exc}")
        return
    if data.get("schema_version") != "org.searchright.estate-replacement-decisions.v1":
        errors.append("estate replacement decisions: invalid schema version")
    decisions = data.get("decisions")
    if not isinstance(decisions, list) or not decisions:
        errors.append("estate replacement decisions: non-empty decisions required")
        return
    for entry in decisions:
        repo = entry.get("repository")
        if not isinstance(repo, str) or not repo.strip():
            errors.append("estate replacement decisions: repository must be non-empty string")
        for field in ("current_form", "target", "status", "deletion_gate"):
            val = entry.get(field)
            if not isinstance(val, str) or not val.strip():
                errors.append(f"estate replacement decisions:{repo}: missing or empty {field}")
        obs = entry.get("observed_path")
        if obs is not None and (not isinstance(obs, str) or not obs.strip()):
            errors.append(f"estate replacement decisions:{repo}: invalid observed_path")


def main() -> int:
    errors: list[str] = []
    if not INDEX.is_file():
        errors.append("missing companion index")
        index = {}
    else:
        try:
            index = json.loads(INDEX.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"unparseable companion index: {exc}")
            index = {}

    if index.get("schema_version") != "org.searchright.companion-change-packet-index.v1":
        errors.append("companion index: schema version")
    if index.get("automatic_apply") is not False:
        errors.append("companion index: automatic apply prohibited")
    if not index.get("claim_boundary"):
        errors.append("companion index: claim boundary missing")

    checked = 0
    changes = 0
    repos: set[str] = set()

    for item in index.get("packets", []):
        path = ROOT / item["path"]
        if not path.is_file():
            errors.append(f"missing packet {item['path']}")
            continue
        try:
            d = json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"unparseable packet {item['path']}: {exc}")
            continue
        checked += 1
        repo = d.get("repository")
        rev = d.get("revision")
        if repo in repos:
            errors.append(f"duplicate companion repository {repo}")
        repos.add(repo)
        if d.get("schema_version") != "org.searchright.companion-change-packet.v1":
            errors.append(f"{repo}: schema version")
        if not isinstance(rev, str) or not HEX40.fullmatch(rev):
            errors.append(f"{repo}: exact revision required")
        if d.get("remote_mutation_permitted") is not False:
            errors.append(f"{repo}: remote mutation must be false")
        if not d.get("claim_boundary") or not d.get("completion_condition"):
            errors.append(f"{repo}: claim/completion boundary missing")
        entries = d.get("changes", [])
        changes += len(entries)
        ids = []
        for change in entries:
            ids.append(change.get("id"))
            if change.get("automatic_apply") is not False:
                errors.append(f"{repo}:{change.get('id')}: automatic apply prohibited")
            for key in ("action", "owner", "target_paths", "required_evidence"):
                if not change.get(key):
                    errors.append(f"{repo}:{change.get('id')}: missing {key}")
        if len(ids) != len(set(ids)):
            errors.append(f"{repo}: duplicate change IDs")

    expected = {
        "edithatogo/citeweft",
        "edithatogo/sourceright",
        "edithatogo/repository-standards",
        "edithatogo/standards_check",
        "edithatogo/academic-research-skills",
        "edithatogo/UOGTO",
        "edithatogo/voiage",
        "edithatogo/scholarly-publishing-agents",
        "edithatogo/api-standards",
        "edithatogo/PRISMA.jl",
        "edithatogo/synergy-dataset",
    }
    if expected - repos:
        errors.append(f"missing required companion packets: {sorted(expected - repos)}")

    validate_estate_repositories(errors)
    validate_estate_decisions(errors)

    receipt = {
        "changes_checked": changes,
        "errors": errors,
        "limitations": ["No companion repository was modified or tested by this check."],
        "packets_checked": checked,
        "remote_mutations": 0,
        "schema_version": "org.searchright.companion-change-packet-receipt.v1",
        "status": "failed" if errors else "passed",
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
