from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


COVERAGE = load_module("check_roadmap_coverage", ROOT / "scripts" / "check_roadmap_coverage.py")
SYNC = load_module("sync_track_evidence", ROOT / "scripts" / "sync_track_evidence.py")


class TrackArchivalTests(unittest.TestCase):
    def archived_entry(self) -> dict:
        return {
            "track_id": "31",
            "slug": "github-control-plane",
            "title": "GitHub control plane",
            "horizon": "mature",
            "status": "source_implemented",
            "implementation_state": "source_implemented",
            "evidence_level": "live_proven",
            "outcome": "Converged projection",
            "lifecycle": "archived",
            "archived_on": "2026-08-12",
            "closeout_completed": True,
            "review_completed": True,
            "higher_evidence_completed": True,
            "completed_higher_evidence_gates": ["live receipt"],
            "blockers": [],
        }

    def test_archived_track_requires_closed_evidence(self) -> None:
        entry = self.archived_entry()
        self.assertEqual(
            COVERAGE.validate_archived_track(
                "31",
                entry,
                "- [x] done\n",
                {"assertions": [{"open_gates": [], "evidence_receipts": ["CONTEXT.md"]}]},
            ),
            [],
        )
        entry["blockers"] = ["still open"]
        violations = COVERAGE.validate_archived_track(
            "31",
            entry,
            "- [ ] pending\n",
            {"assertions": [{"open_gates": ["gate"], "evidence_receipts": ["CONTEXT.md"]}]},
        )
        self.assertEqual(len(violations), 3)

    def test_archived_track_rejects_false_completion_shapes(self) -> None:
        entry = self.archived_entry()
        entry.update(
            archived_on="9999-99-99",
            review_completed=False,
            higher_evidence_completed=False,
            completed_higher_evidence_gates=[],
            evidence_level="source_verified",
        )
        violations = COVERAGE.validate_archived_track(
            "31", entry, "- [x] done\n", {"assertions": [{}]}
        )
        self.assertGreaterEqual(len(violations), 7)

    def test_registry_separates_archived_tracks_without_losing_link(self) -> None:
        active = dict(self.archived_entry(), track_id="30", slug="maturity", lifecycle="active")
        rendered = SYNC.render_tracks([active, self.archived_entry()])
        active_table, archived_table = rendered.split("## Archived tracks", maxsplit=1)
        self.assertIn("tracks/30-maturity/spec.md", active_table)
        self.assertNotIn("tracks/31-github-control-plane/spec.md", active_table)
        self.assertIn("tracks/31-github-control-plane/spec.md", archived_table)

    def test_phase_task_identity_is_immutable_across_gate_completion(self) -> None:
        entry = self.archived_entry()
        entry["phase_3_task_count"] = 3
        entry["blockers"] = []
        entry["completed_higher_evidence_gates"] = ["one", "two"]
        self.assertEqual(SYNC.task_counts(entry)[3], 3)

    def test_review_history_preserves_both_record_shapes(self) -> None:
        entry = dict(self.archived_entry(), deliverables=[], checks=[], review_fixes=[
            "Historical reviewed fix (`abcdef0`).",
            {"commit": "1234567", "summary": "Structured reviewed fix."},
        ])
        rendered = SYNC.render_plan(entry)
        self.assertEqual(rendered.count("  - [x] Review fix: Historical reviewed fix (`abcdef0`)."), 1)
        self.assertEqual(rendered.count("  - Review fix `1234567`: Structured reviewed fix."), 1)
        self.assertEqual(SYNC.task_counts(entry)[4], 4)

    def test_owner_decision_label_does_not_change_other_tracks(self) -> None:
        entry = dict(self.archived_entry(), deliverables=[], checks=[])
        default = SYNC.render_plan(entry)
        entry["closeout_gate_label"] = "Close only after the owner's recorded panel decision."
        self.assertIn("- [x] Close only after the owner's recorded panel decision.", SYNC.render_plan(entry))
        self.assertIn("- [x] Close the track only when all applicable", default)


if __name__ == "__main__":
    unittest.main()
