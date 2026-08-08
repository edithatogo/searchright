#!/usr/bin/env python3
"""Check the generated epic -> track -> phase -> task GitHub hierarchy."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HIERARCHY = ROOT / "conductor/github/issue-hierarchy.json"
COVERAGE = ROOT / "conductor/roadmap-coverage.json"


def top_level_task_counts(plan: str) -> dict[int, int]:
    phase_re = re.compile(r"^## Phase (\d+): .+$", re.MULTILINE)
    task_re = re.compile(r"^- \[[ xX]\] ", re.MULTILINE)
    matches = list(phase_re.finditer(plan))
    result: dict[int, int] = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(plan)
        result[int(match.group(1))] = len(task_re.findall(plan[match.end() : end]))
    return result


def main() -> int:
    errors: list[str] = []
    data = json.loads(HIERARCHY.read_text(encoding="utf-8"))
    tracks = json.loads(COVERAGE.read_text(encoding="utf-8"))["tracks"]
    nodes = data.get("nodes", [])
    by_key = {node.get("key"): node for node in nodes}
    expected_tracks = {f"track-{entry['track_id']}" for entry in tracks}
    expected_phases = {f"track-{entry['track_id']}-phase-{number}" for entry in tracks for number in range(1, 5)}
    expected_tasks: set[str] = set()
    for entry in tracks:
        track_id = entry["track_id"]
        plan = (ROOT / f"conductor/tracks/{track_id}-{entry['slug']}/plan.md").read_text(encoding="utf-8")
        counts = top_level_task_counts(plan)
        if sorted(counts) != [1, 2, 3, 4]:
            errors.append(f"track {track_id} does not expose phases 1-4")
        for phase, count in counts.items():
            if count == 0:
                errors.append(f"track {track_id} phase {phase} has no task")
            expected_tasks.update(f"track-{track_id}-phase-{phase}-task-{number:02d}" for number in range(1, count + 1))
    epics = [node for node in nodes if node.get("kind") == "epic"]
    actual_tracks = {node["key"] for node in nodes if node.get("kind") == "track"}
    actual_phases = {node["key"] for node in nodes if node.get("kind") == "phase"}
    actual_tasks = {node["key"] for node in nodes if node.get("kind") == "task"}
    expected_count = 1 + len(expected_tracks) + len(expected_phases) + len(expected_tasks)
    if len(nodes) != expected_count:
        errors.append(f"expected {expected_count} nodes, found {len(nodes)}")
    if len(epics) != 1 or epics[0].get("key") != "roadmap-epic" or epics[0].get("parent_key") is not None:
        errors.append("invalid roadmap epic")
    if actual_tracks != expected_tracks:
        errors.append("track issue keys differ from roadmap")
    if actual_phases != expected_phases:
        errors.append("phase issue keys differ from four phases per track")
    if actual_tasks != expected_tasks:
        errors.append("task issue keys differ from top-level Conductor tasks")
    allowed_project_fields = {
        "Delivery status",
        "Work kind",
        "Horizon",
        "Evidence level",
        "MoSCoW",
        "External gate",
        "Conductor key",
        "Conductor path",
        "Track ID",
        "Phase",
        "Task",
    }
    for key, node in by_key.items():
        path = ROOT / node.get("body_path", "")
        if not path.is_file():
            errors.append(f"missing body for {key}")
            continue
        body = path.read_text(encoding="utf-8")
        if f"<!-- searchright-issue-key: {key} -->" not in body:
            errors.append(f"missing stable marker for {key}")
        if node.get("status") != "prepared_not_synced":
            errors.append(f"{key} overclaims remote status")
        if node.get("desired_state") not in {"open", "closed"}:
            errors.append(f"{key} has invalid desired state")
        fields = node.get("project_fields")
        if not isinstance(fields, dict) or not fields:
            errors.append(f"{key} lacks project fields")
        elif set(fields) - allowed_project_fields:
            errors.append(f"{key} has unknown project fields: {sorted(set(fields)-allowed_project_fields)}")
        kind = node.get("kind")
        if kind == "track" and node.get("parent_key") != "roadmap-epic":
            errors.append(f"{key} has wrong epic parent")
        elif kind == "phase":
            match = re.fullmatch(r"(track-\d{2})-phase-([1-4])", key)
            if not match or node.get("parent_key") != match.group(1) or match.group(1) not in by_key:
                errors.append(f"{key} has wrong track parent")
        elif kind == "task":
            match = re.fullmatch(r"(track-\d{2}-phase-[1-4])-task-(\d{2})", key)
            if not match or node.get("parent_key") != match.group(1) or match.group(1) not in by_key:
                errors.append(f"{key} has wrong phase parent")
            if node.get("desired_state") == "closed" and "state:source-complete" not in node.get("labels", []):
                errors.append(f"{key} closes without source-complete label")
    if data.get("apply_permitted") is not False:
        errors.append("local hierarchy must not authorise remote apply")
    if data.get("state_sync_policy") != "task_issues_only":
        errors.append("only task issues may have their remote open/closed state synchronised")
    if data.get("project_manifest") != "conductor/github/project.json":
        errors.append("hierarchy must point to the canonical project manifest")
    for entry in tracks:
        track_id = entry["track_id"]
        directory = ROOT / f"conductor/tracks/{track_id}-{entry['slug']}"
        metadata = json.loads((directory / "metadata.json").read_text(encoding="utf-8"))
        plan = (directory / "plan.md").read_text(encoding="utf-8")
        github = metadata.get("github", {})
        counts = top_level_task_counts(plan)
        expected_track_tasks = [
            f"track-{track_id}-phase-{phase}-task-{number:02d}"
            for phase, count in sorted(counts.items())
            for number in range(1, count + 1)
        ]
        if (
            github.get("track_issue_key") != f"track-{track_id}"
            or github.get("phase_issue_keys") != [f"track-{track_id}-phase-{number}" for number in range(1, 5)]
            or github.get("task_issue_keys") != expected_track_tasks
        ):
            errors.append(f"track {track_id} metadata issue keys differ")
        for number in range(1, 5):
            if f"<!-- github-subissue-key: track-{track_id}-phase-{number} -->" not in plan:
                errors.append(f"track {track_id} plan lacks phase {number} marker")
    receipt = {
        "schema_version": "org.searchright.github-issue-hierarchy-receipt.v2",
        "status": "failed" if errors else "passed",
        "nodes_checked": len(nodes),
        "track_issues": len(actual_tracks),
        "phase_subissues": len(actual_phases),
        "task_subissues": len(actual_tasks),
        "maximum_depth": 4,
        "errors": errors,
        "limitations": ["No remote GitHub repository, issue number, subissue relationship or Project item is claimed."],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
