#!/usr/bin/env python3
"""Synchronise Conductor metadata, evidence and plans from roadmap coverage.

The machine-readable roadmap is the only dependency/status source. Every track
maps to one GitHub issue key, every plan phase maps to a deterministic subissue,
and every top-level phase task maps to a nested task subissue; remote identities
remain external evidence.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COVERAGE = ROOT / "conductor" / "roadmap-coverage.json"
TRACKS = ROOT / "conductor" / "tracks"
PHASES = (
    (1, "Source implementation"),
    (2, "Source-level verification"),
    (3, "Higher-evidence gates"),
    (4, "Review and closeout"),
)


def task_counts(entry: dict) -> dict[int, int]:
    return {1: 1, 2: 2, 3: max(1, len(entry.get("blockers", []))), 4: 4}


def issue_keys(track_id: str, entry: dict) -> dict[str, object]:
    tasks = [
        f"track-{track_id}-phase-{phase}-task-{number:02d}"
        for phase, count in task_counts(entry).items()
        for number in range(1, count + 1)
    ]
    return {
        "epic_issue_key": "roadmap-epic",
        "track_issue_key": f"track-{track_id}",
        "phase_issue_keys": [f"track-{track_id}-phase-{number}" for number, _ in PHASES],
        "task_issue_keys": tasks,
        "remote_issue_number": None,
        "remote_phase_issue_numbers": [],
        "remote_task_issue_numbers": [],
    }


def render_metadata(entry: dict) -> str:
    track_id = entry["track_id"]
    value = {
        "schema_version": "conductor.track-metadata.v3",
        "track_id": track_id,
        "slug": entry["slug"],
        "title": entry["title"],
        "status": entry["status"],
        "implementation_state": entry["implementation_state"],
        "created": "2026-08-05" if int(track_id) <= 11 else ("2026-08-06" if int(track_id) <= 30 else "2026-08-08"),
        "updated": entry.get("updated", "2026-08-09"),
        "dependencies": entry.get("dependencies", []),
        "horizon": entry["horizon"],
        "evidence_level": entry["evidence_level"],
        "external_approval_required": bool(entry.get("external_approval_required", False)),
        "evidence_path": f"conductor/tracks/{track_id}-{entry['slug']}/evidence.json",
        "traceability_path": f"conductor/tracks/{track_id}-{entry['slug']}/traceability.json",
        "github": issue_keys(track_id, entry),
    }
    return json.dumps(value, indent=2) + "\n"


def render_evidence(entry: dict) -> str:
    track_id = entry["track_id"]
    keys = issue_keys(track_id, entry)
    value = {
        "schema_version": "org.searchright.track-evidence.v2",
        "track_id": track_id,
        "title": entry["title"],
        "status": entry["status"],
        "implementation_state": entry["implementation_state"],
        "evidence_level": entry["evidence_level"],
        "source_verified_on": entry.get("source_verified_on", "2026-08-06"),
        "source_evidence": entry["deliverables"],
        "traceability": f"conductor/tracks/{track_id}-{entry['slug']}/traceability.json",
        "static_checks": entry["checks"],
        "requirements": entry["requirements"],
        "blockers": entry["blockers"],
        "claim_boundary": entry["claim_boundary"],
        "github_issue_keys": {
            "epic": keys["epic_issue_key"],
            "track": keys["track_issue_key"],
            "phases": keys["phase_issue_keys"],
            "tasks": keys["task_issue_keys"],
        },
        "remote_github_evidence": entry.get("remote_github_evidence", []),
        "runtime_evidence": entry.get("runtime_evidence", []),
        "external_evidence": entry.get("external_evidence", []),
    }
    return json.dumps(value, indent=2) + "\n"


def render_plan(entry: dict) -> str:
    track_id = entry["track_id"]
    lines = [
        f"# Plan: {track_id} {entry['title']}",
        "",
        f"Current status: **{entry['status']}**. Implementation state: **{entry['implementation_state']}**. Evidence level: **{entry['evidence_level']}**.",
        "",
        f"GitHub issue key: `track-{track_id}`. Each numbered phase maps to the same-numbered native subissue.",
        "",
        "## Phase 1: Source implementation",
        "",
        f"<!-- github-subissue-key: track-{track_id}-phase-1 -->",
        "",
        (
            "- [x] Implement and document every acceptance assertion with symbol- and test-level mappings."
            if entry["implementation_state"] == "source_implemented"
            else "- [ ] Complete every acceptance assertion; existing paths are scaffolding or partial implementation only."
        ),
    ]
    for path in entry["deliverables"]:
        lines.append(f"  - [x] Present source path: `{path}`")
    lines.append(f"  - [x] Assertion ledger: `conductor/tracks/{track_id}-{entry['slug']}/traceability.json`")
    lines.extend(
        [
            "",
            "## Phase 2: Source-level verification",
            "",
            f"<!-- github-subissue-key: track-{track_id}-phase-2 -->",
            "",
            "- [x] Run deterministic, network-free contract and policy checks.",
        ]
    )
    for check in entry["checks"]:
        lines.append(f"  - [x] `{check}`")
    lines.extend(
        [
            "- [x] Record machine-readable evidence without promoting compiler, live or external claims.",
            "",
            "## Phase 3: Higher-evidence gates",
            "",
            f"<!-- github-subissue-key: track-{track_id}-phase-3 -->",
            "",
        ]
    )
    for blocker in entry["blockers"]:
        lines.append(f"- [ ] {blocker}")
    if not entry["blockers"]:
        lines.append("- [ ] Promote evidence only when a newer reproducible receipt justifies it.")
    lines.extend(
        [
            "",
            "## Phase 4: Review and closeout",
            "",
            f"<!-- github-subissue-key: track-{track_id}-phase-4 -->",
            "",
            "- [x] Reconcile source paths, requirements, interface effects and claim boundaries.",
            "- [x] Record unresolved blockers in `evidence.json` and the roadmap coverage ledger.",
            (
                "- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute."
                if entry.get("review_completed", False)
                else "- [ ] Run compiler-backed Conductor review and append review fixes after Cargo gates execute."
            ),
            "- [ ] Close the track only when all applicable live, downstream, human and external gates are evidenced.",
            "",
        ]
    )
    return "\n".join(lines)


def render_tracks(entries: list[dict]) -> str:
    lines = [
        "# Tracks",
        "",
        "Track status, implementation completeness and evidence level are separate.",
        "`scaffolded` and `partially_implemented` prevent path presence from being",
        "misrepresented as completed behaviour; `traceability.json` owns assertion-level claims.",
        "",
        "Each track maps to `track-NN`; each phase maps to `track-NN-phase-M`; and",
        "each top-level plan task maps to `track-NN-phase-M-task-TT`. The generated",
        "native issue hierarchy and Project projection remain prepared-not-synced until",
        "an explicit, approval-gated apply receipt exists.",
        "",
        "| ID | Track | Horizon | Status | Evidence | Outcome |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for entry in entries:
        path = f"tracks/{entry['track_id']}-{entry['slug']}/spec.md"
        lines.append(
            f"| {entry['track_id']} | [{entry['title']}]({path}) | {entry['horizon']} | "
            f"{entry['status']} | {entry['implementation_state']} | {entry['evidence_level']} | {entry['outcome']} |"
        )
    lines.extend(
        [
            "",
            "## Evidence ladder",
            "",
            "Contracted → source-verified → compiler-verified → fixture-proven →",
            "opt-in live proven → externally validated → publicly accepted.",
            "",
            "The canonical machine-readable mapping is `roadmap-coverage.json`; each track",
            "contains `evidence.json`, `spec.md` and an evidence-aware `plan.md`.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    coverage = json.loads(COVERAGE.read_text(encoding="utf-8"))
    outputs: dict[Path, str] = {}
    for entry in coverage["tracks"]:
        directory = TRACKS / f"{entry['track_id']}-{entry['slug']}"
        outputs[directory / "metadata.json"] = render_metadata(entry)
        outputs[directory / "evidence.json"] = render_evidence(entry)
        outputs[directory / "plan.md"] = render_plan(entry)
    outputs[ROOT / "conductor" / "tracks.md"] = render_tracks(coverage["tracks"])

    stale: list[str] = []
    for path, content in outputs.items():
        if args.check:
            if not path.is_file() or path.read_text(encoding="utf-8") != content:
                stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
    if stale:
        print(json.dumps({"status": "failed", "stale": stale}, indent=2))
        return 1
    print(
        json.dumps(
            {"status": "passed", "files": len(outputs), "mode": "check" if args.check else "write"},
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
