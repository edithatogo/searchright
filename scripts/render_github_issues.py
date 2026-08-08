#!/usr/bin/env python3
"""Render the Conductor roadmap into deterministic GitHub issue bodies.

The hierarchy is deliberately four levels deep:
roadmap epic -> track -> phase -> task. Conductor remains canonical.
"""
from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
COVERAGE = ROOT / "conductor/roadmap-coverage.json"
REQUIREMENTS = ROOT / "conductor/requirements.md"
OUT = ROOT / "conductor/github/issues"
HIERARCHY = ROOT / "conductor/github/issue-hierarchy.json"
LABELS = ROOT / "conductor/github/labels.json"
PHASE_RE = re.compile(r"^## Phase (\d+): (.+)$", re.MULTILINE)
TASK_RE = re.compile(r"^- \[([ xX])\] (.+)$")
REQ_RE = re.compile(r"^\|\s*(SR-\d{3})\s*\|\s*(Must|Should|Could|Won[’']t(?: now)?)\s*\|", re.MULTILINE)
PRIORITY_RANK = {"Must": 0, "Should": 1, "Could": 2, "Won't": 3}


@dataclass(frozen=True)
class Task:
    number: int
    completed: bool
    title: str
    markdown: str


def phase_sections(plan: str) -> list[tuple[int, str, str]]:
    matches = list(PHASE_RE.finditer(plan))
    result: list[tuple[int, str, str]] = []
    for index, match in enumerate(matches):
        start = match.end()
        end = matches[index + 1].start() if index + 1 < len(matches) else len(plan)
        result.append((int(match.group(1)), match.group(2).strip(), plan[start:end].strip()))
    return result


def phase_tasks(section: str) -> list[Task]:
    tasks: list[Task] = []
    current: list[str] = []
    completed = False
    title = ""
    for line in section.splitlines():
        match = TASK_RE.match(line)
        if match:
            if current:
                tasks.append(Task(len(tasks) + 1, completed, title, "\n".join(current).strip()))
            completed = match.group(1).lower() == "x"
            title = match.group(2).strip()
            current = [line]
        elif current:
            current.append(line)
    if current:
        tasks.append(Task(len(tasks) + 1, completed, title, "\n".join(current).strip()))
    return tasks


def marker(key: str) -> str:
    return f"<!-- searchright-issue-key: {key} -->"


def title_fragment(value: str, maximum: int = 170) -> str:
    value = re.sub(r"`([^`]+)`", r"\1", value)
    value = re.sub(r"\s+", " ", value).strip().rstrip(".")
    return value if len(value) <= maximum else value[: maximum - 1].rstrip() + "…"


def requirement_priorities() -> dict[str, str]:
    result: dict[str, str] = {}
    for requirement, priority in REQ_RE.findall(REQUIREMENTS.read_text(encoding="utf-8")):
        result[requirement] = "Won't" if priority.startswith("Won") else priority
    return result


def track_priority(entry: dict[str, Any], priorities: dict[str, str]) -> str:
    values = [priorities.get(req, "Should") for req in entry.get("requirements", [])]
    return min(values or ["Should"], key=lambda item: PRIORITY_RANK[item])


def project_status(kind: str, *, completed: bool = False, phase: int | None = None, track_status: str = "") -> str:
    if kind == "epic":
        return "In progress"
    if kind == "track":
        if track_status == "source_implemented":
            return "Source complete"
        if track_status in {"external_evidence_required", "submission_prepared", "release_prepared"}:
            return "Evidence blocked"
        return "In progress"
    if kind == "phase":
        if completed:
            return "Source complete"
        if phase == 3:
            return "Evidence blocked"
        if phase == 4:
            return "Review"
        return "In progress"
    if completed:
        return "Source complete"
    if phase == 3:
        return "Evidence blocked"
    if phase == 4:
        return "Review"
    return "Backlog"


def project_fields(
    *,
    kind: str,
    key: str,
    horizon: str,
    evidence: str,
    implementation: str,
    priority: str,
    status: str,
    track_id: str | None,
    phase: int | None,
    task: int | None,
    conductor_path: str,
    external_gate: str,
) -> dict[str, str | int]:
    fields: dict[str, str | int] = {
        "Delivery status": status,
        "Work kind": kind.capitalize(),
        "Horizon": horizon.capitalize(),
        "Evidence level": evidence.replace("_", " ").title(),
        "Implementation state": implementation.replace("_", " ").title(),
        "MoSCoW": priority,
        "External gate": external_gate,
        "Conductor key": key,
        "Conductor path": conductor_path,
    }
    if track_id is not None:
        fields["Track ID"] = track_id
    if phase is not None:
        fields["Phase"] = phase
    if task is not None:
        fields["Task"] = task
    return fields


def epic_body(entries: list[dict[str, Any]]) -> str:
    lines = [
        marker("roadmap-epic"),
        "# Searchright roadmap epic",
        "",
        "This issue is generated from `conductor/roadmap-coverage.json`. Conductor remains canonical; remote state cannot promote repository evidence.",
        "",
        "## Tracks",
        "",
    ]
    for entry in entries:
        lines.append(f"- [ ] `{entry['track_id']}` — {entry['title']} (`track-{entry['track_id']}`)")
    lines += [
        "",
        "## Synchronisation contract",
        "",
        "- Dry-run is the default.",
        "- Apply requires explicit CLI and environment opt-ins plus GitHub write permission.",
        "- Track issues are native subissues of this epic.",
        "- Phase issues are native subissues of their track.",
        "- Individual top-level Conductor tasks are native subissues of their phase.",
        "- Stable markers preserve idempotency and a portable hierarchy fallback.",
        "- GitHub Project fields are a projection; they never replace Conductor evidence.",
        "",
    ]
    return "\n".join(lines)


def track_body(entry: dict[str, Any]) -> str:
    track_id = entry["track_id"]
    lines = [
        marker(f"track-{track_id}"),
        f"# Track {track_id}: {entry['title']}",
        "",
        entry["outcome"],
        "",
        "## Source of truth",
        "",
        f"- Spec: `conductor/tracks/{track_id}-{entry['slug']}/spec.md`",
        f"- Plan: `conductor/tracks/{track_id}-{entry['slug']}/plan.md`",
        f"- Evidence: `conductor/tracks/{track_id}-{entry['slug']}/evidence.json`",
        "",
        "## Contract",
        "",
        f"- Horizon: `{entry['horizon']}`",
        f"- Status: `{entry['status']}`",
        f"- Implementation: `{entry['implementation_state']}`",
        f"- Evidence: `{entry['evidence_level']}`",
        f"- Dependencies: `{', '.join(entry.get('dependencies', [])) or 'none'}`",
        f"- Requirements: `{', '.join(entry.get('requirements', [])) or 'none'}`",
        f"- External approval required: `{str(bool(entry.get('external_approval_required'))).lower()}`",
        "",
        "## Phase subissues",
        "",
    ]
    for number, title in (
        (1, "Source implementation"),
        (2, "Source-level verification"),
        (3, "Higher-evidence gates"),
        (4, "Review and closeout"),
    ):
        lines.append(f"- [ ] Phase {number}: {title} (`track-{track_id}-phase-{number}`)")
    lines += [
        "",
        "## Claim boundary",
        "",
        entry["claim_boundary"],
        "",
        "> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime or external receipts remain authoritative.",
        "",
    ]
    return "\n".join(lines)


def phase_body(entry: dict[str, Any], number: int, title: str, tasks: list[Task]) -> str:
    track_id = entry["track_id"]
    key = f"track-{track_id}-phase-{number}"
    lines = [
        marker(key),
        f"# Track {track_id} / Phase {number}: {title}",
        "",
        f"Parent track key: `track-{track_id}`",
        f"Conductor plan: `conductor/tracks/{track_id}-{entry['slug']}/plan.md`",
        "",
        "## Task subissues",
        "",
    ]
    for task in tasks:
        mark = "x" if task.completed else " "
        lines.append(
            f"- [{mark}] T{task.number:02d}: {task.title} (`{key}-task-{task.number:02d}`)"
        )
    lines += [
        "",
        "## Evidence rule",
        "",
        "Remote completion is a planning signal only. Evidence is promoted only through the track evidence record and a reproducible receipt at the claimed level.",
        "",
    ]
    return "\n".join(lines)


def task_body(entry: dict[str, Any], phase: int, phase_title: str, task: Task) -> str:
    track_id = entry["track_id"]
    key = f"track-{track_id}-phase-{phase}-task-{task.number:02d}"
    completion = "source task complete" if task.completed else "open evidence or implementation task"
    return "\n".join(
        [
            marker(key),
            f"# Track {track_id} / Phase {phase} / Task {task.number:02d}",
            "",
            f"Parent phase key: `track-{track_id}-phase-{phase}`",
            f"Conductor plan: `conductor/tracks/{track_id}-{entry['slug']}/plan.md`",
            f"Canonical task state: **{completion}**.",
            "",
            "## Canonical task",
            "",
            task.markdown,
            "",
            "## Completion and evidence contract",
            "",
            "- This issue mirrors one top-level checklist item in the Conductor plan.",
            "- Nested checklist entries remain acceptance details inside this issue.",
            "- Closing a source-complete task does not promote the parent track's evidence level.",
            "- Reopening or closing is synchronised only from the canonical Conductor checklist.",
            "- Higher-evidence, downstream, human and registry gates require their own receipts.",
            "",
        ]
    )


def label_manifest(entries: list[dict[str, Any]]) -> dict[str, Any]:
    labels = [
        {"name": "conductor", "color": "1d76db", "description": "Generated from Conductor source-of-truth artefacts"},
        {"name": "kind:epic", "color": "5319e7", "description": "Roadmap epic"},
        {"name": "kind:track", "color": "0052cc", "description": "Conductor track"},
        {"name": "kind:phase", "color": "bfd4f2", "description": "Numbered Conductor plan phase"},
        {"name": "kind:task", "color": "d4e5ff", "description": "Top-level Conductor plan task"},
        {"name": "state:source-complete", "color": "0e8a16", "description": "Canonical source task is complete; higher evidence may remain open"},
        {"name": "state:evidence-open", "color": "d93f0b", "description": "Canonical task remains open or requires higher evidence"},
        {"name": "evidence:source-verified", "color": "0e8a16", "description": "Source-level evidence only"},
        {"name": "evidence:external-required", "color": "d93f0b", "description": "Needs live, downstream, human or external evidence"},
    ]
    horizon_colours = {
        "foundation": "c5def5",
        "mvp": "bfe5bf",
        "alpha": "fbca04",
        "beta": "f9d0c4",
        "mature": "d4c5f9",
    }
    for horizon in sorted({entry["horizon"] for entry in entries}):
        labels.append({"name": f"horizon:{horizon}", "color": horizon_colours.get(horizon, "ededed"), "description": f"{horizon.capitalize()} horizon"})
    for phase in range(1, 5):
        labels.append({"name": f"phase:{phase}", "color": "bfdadc", "description": f"Conductor phase {phase}"})
    for entry in entries:
        labels.append({"name": f"track:{entry['track_id']}", "color": "dbeafe", "description": f"Conductor track {entry['track_id']}"})
    return {"schema_version": "org.searchright.github-labels.v2", "labels": labels}


def build() -> tuple[dict[str, Any], dict[Path, str]]:
    entries = json.loads(COVERAGE.read_text(encoding="utf-8"))["tracks"]
    priorities = requirement_priorities()
    outputs: dict[Path, str] = {}
    nodes: list[dict[str, Any]] = []
    outputs[OUT / "roadmap-epic.md"] = epic_body(entries)
    nodes.append(
        {
            "key": "roadmap-epic",
            "title": "Searchright roadmap",
            "kind": "epic",
            "parent_key": None,
            "body_path": "conductor/github/issues/roadmap-epic.md",
            "labels": ["kind:epic", "conductor"],
            "status": "prepared_not_synced",
            "desired_state": "open",
            "track_id": None,
            "phase_number": None,
            "task_number": None,
            "project_fields": project_fields(
                kind="epic",
                key="roadmap-epic",
                horizon="mature",
                evidence="source_verified",
                implementation="mixed",
                priority="Must",
                status="In progress",
                track_id=None,
                phase=None,
                task=None,
                conductor_path="conductor/roadmap-coverage.json",
                external_gate="Multiple",
            ),
        }
    )
    for entry in entries:
        track_id = entry["track_id"]
        track_key = f"track-{track_id}"
        track_path = OUT / f"{track_key}.md"
        outputs[track_path] = track_body(entry)
        priority = track_priority(entry, priorities)
        labels = [
            "kind:track",
            "conductor",
            f"track:{track_id}",
            f"horizon:{entry['horizon']}",
            "evidence:external-required" if entry.get("external_approval_required") else "evidence:source-verified",
        ]
        nodes.append(
            {
                "key": track_key,
                "title": f"Track {track_id}: {entry['title']}",
                "kind": "track",
                "parent_key": "roadmap-epic",
                "body_path": track_path.relative_to(ROOT).as_posix(),
                "labels": labels,
                "status": "prepared_not_synced",
                "desired_state": "open",
                "track_id": track_id,
                "phase_number": None,
                "task_number": None,
                "project_fields": project_fields(
                    kind="track",
                    key=track_key,
                    horizon=entry["horizon"],
                    evidence=entry["evidence_level"],
                    implementation=entry["implementation_state"],
                    priority=priority,
                    status=project_status("track", track_status=entry["status"]),
                    track_id=track_id,
                    phase=None,
                    task=None,
                    conductor_path=f"conductor/tracks/{track_id}-{entry['slug']}",
                    external_gate="Yes" if entry.get("external_approval_required") else "No",
                ),
            }
        )
        plan_path = ROOT / f"conductor/tracks/{track_id}-{entry['slug']}/plan.md"
        phases = phase_sections(plan_path.read_text(encoding="utf-8"))
        if [phase[0] for phase in phases] != [1, 2, 3, 4]:
            raise ValueError(f"track {track_id} must contain phases 1-4 exactly")
        for number, phase_title, section in phases:
            tasks = phase_tasks(section)
            if not tasks:
                raise ValueError(f"track {track_id} phase {number} has no top-level task")
            phase_key = f"{track_key}-phase-{number}"
            phase_path = OUT / f"{phase_key}.md"
            outputs[phase_path] = phase_body(entry, number, phase_title, tasks)
            phase_complete = all(task.completed for task in tasks)
            nodes.append(
                {
                    "key": phase_key,
                    "title": f"Track {track_id} / Phase {number}: {phase_title}",
                    "kind": "phase",
                    "parent_key": track_key,
                    "body_path": phase_path.relative_to(ROOT).as_posix(),
                    "labels": ["kind:phase", "conductor", f"track:{track_id}", f"phase:{number}"],
                    "status": "prepared_not_synced",
                    "desired_state": "open",
                    "track_id": track_id,
                    "phase_number": number,
                    "task_number": None,
                    "project_fields": project_fields(
                        kind="phase",
                        key=phase_key,
                        horizon=entry["horizon"],
                        evidence=entry["evidence_level"],
                        implementation=entry["implementation_state"],
                        priority=priority,
                        status=project_status("phase", completed=phase_complete, phase=number),
                        track_id=track_id,
                        phase=number,
                        task=None,
                        conductor_path=f"conductor/tracks/{track_id}-{entry['slug']}/plan.md",
                        external_gate="Yes" if number == 3 or entry.get("external_approval_required") else "No",
                    ),
                }
            )
            for task in tasks:
                task_key = f"{phase_key}-task-{task.number:02d}"
                task_path = OUT / f"{task_key}.md"
                outputs[task_path] = task_body(entry, number, phase_title, task)
                nodes.append(
                    {
                        "key": task_key,
                        "title": f"Track {track_id} / P{number} / T{task.number:02d}: {title_fragment(task.title)}",
                        "kind": "task",
                        "parent_key": phase_key,
                        "body_path": task_path.relative_to(ROOT).as_posix(),
                        "labels": [
                            "kind:task",
                            "conductor",
                            f"track:{track_id}",
                            f"phase:{number}",
                            "state:source-complete" if task.completed else "state:evidence-open",
                        ],
                        "status": "prepared_not_synced",
                        "desired_state": "closed" if task.completed else "open",
                        "track_id": track_id,
                        "phase_number": number,
                        "task_number": task.number,
                        "project_fields": project_fields(
                            kind="task",
                            key=task_key,
                            horizon=entry["horizon"],
                            evidence=entry["evidence_level"],
                            implementation=entry["implementation_state"],
                            priority=priority,
                            status=project_status("task", completed=task.completed, phase=number),
                            track_id=track_id,
                            phase=number,
                            task=task.number,
                            conductor_path=f"conductor/tracks/{track_id}-{entry['slug']}/plan.md",
                            external_gate="Yes" if number == 3 else "No",
                        ),
                    }
                )
    hierarchy = {
        "schema_version": "org.searchright.github-issue-hierarchy.v2",
        "repository": "edithatogo/searchright",
        "epic_key": "roadmap-epic",
        "nodes": nodes,
        "generated_at": "source-epoch:2026-08-08",
        "apply_permitted": False,
        "state_sync_policy": "task_issues_only",
        "project_manifest": "conductor/github/project.json",
    }
    outputs[HIERARCHY] = json.dumps(hierarchy, indent=2, sort_keys=False) + "\n"
    outputs[LABELS] = json.dumps(label_manifest(entries), indent=2, sort_keys=False) + "\n"
    return hierarchy, outputs


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    hierarchy, outputs = build()
    stale: list[str] = []
    for path, content in outputs.items():
        if args.check:
            if not path.is_file() or path.read_text(encoding="utf-8") != content:
                stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
    expected = {path for path in outputs if path.parent == OUT}
    extras = {path for path in OUT.glob("*.md")} - expected
    if args.check and extras:
        stale += [f"extra:{path.relative_to(ROOT).as_posix()}" for path in sorted(extras)]
    elif not args.check:
        for path in extras:
            path.unlink()
    status = "failed" if stale else "passed"
    kinds: dict[str, int] = {}
    for node in hierarchy["nodes"]:
        kinds[node["kind"]] = kinds.get(node["kind"], 0) + 1
    print(
        json.dumps(
            {
                "schema_version": "org.searchright.github-issue-render-receipt.v2",
                "status": status,
                "nodes": len(hierarchy["nodes"]),
                "kinds": kinds,
                "issue_bodies": len(expected),
                "stale": stale,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 1 if stale else 0


if __name__ == "__main__":
    raise SystemExit(main())
