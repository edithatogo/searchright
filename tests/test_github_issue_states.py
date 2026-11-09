from __future__ import annotations

import contextlib
import io
import json
import os
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import audit_github_control_plane as audit
import check_github_issue_hierarchy as validator
import render_github_issues as renderer
import sync_github_issues as sync


@contextlib.contextmanager
def projection(*, unfinished_phase: int | None = None, overrides: dict | None = None):
    """Exercise the actual renderer against a small, independent Conductor plan."""
    with TemporaryDirectory() as temporary, contextlib.ExitStack() as stack:
        root = Path(temporary)
        coverage = root / "conductor/roadmap-coverage.json"
        hierarchy_path = root / "conductor/github/issue-hierarchy.json"
        labels = root / "conductor/github/labels.json"
        requirements = root / "conductor/requirements.md"
        entry = {
            "track_id": "00", "slug": "example", "title": "Example", "horizon": "mvp",
            "outcome": "A bounded work unit", "status": "source_implemented",
            "implementation_state": "source_implemented", "evidence_level": "fixture_proven",
            "claim_boundary": "Synthetic tests are not external validation.",
            "requirements": [], "lifecycle": "archived", "closeout_completed": True,
            "blockers": [],
        }
        entry.update(overrides or {})
        plan_path = root / "conductor/tracks/00-example/plan.md"
        plan_path.parent.mkdir(parents=True)
        plan_path.write_text("\n".join(
            f"## Phase {number}: Work {number}\n\n"
            f"<!-- github-subissue-key: track-00-phase-{number} -->\n\n"
            f"- [{' ' if number == unfinished_phase else 'x'}] Obligation {number}.\n"
            for number in range(1, 5)
        ))
        coverage.write_text(json.dumps({"tracks": [entry]}))
        requirements.write_text("")
        for module in (renderer, validator, sync, audit):
            stack.enter_context(patch.object(module, "ROOT", root))
        for module in (renderer, validator, sync):
            stack.enter_context(patch.object(module, "HIERARCHY", hierarchy_path))
        for module in (renderer, validator):
            stack.enter_context(patch.object(module, "COVERAGE", coverage))
        stack.enter_context(patch.object(renderer, "REQUIREMENTS", requirements))
        stack.enter_context(patch.object(renderer, "OUT", root / "conductor/github/issues"))
        for module in (renderer, sync):
            stack.enter_context(patch.object(module, "LABELS", labels))
        hierarchy, outputs = renderer.build()
        for path, body in outputs.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(body)
        (plan_path.parent / "metadata.json").write_text(json.dumps({"github": {
            "track_issue_key": "track-00",
            "phase_issue_keys": [n["key"] for n in hierarchy["nodes"] if n["kind"] == "phase"],
            "task_issue_keys": [n["key"] for n in hierarchy["nodes"] if n["kind"] == "task"],
        }}))
        yield hierarchy, root


@contextlib.contextmanager
def remote_session(hierarchy: dict, root: Path, *, initial_state: str = "open"):
    """An in-memory GitHub boundary; unexpected writes fail the test."""
    issues = {
        node["key"]: {
            "number": index + 1, "id": index + 101, "title": node["title"],
            "body": (root / node["body_path"]).read_text(), "labels": node["labels"],
            "state": initial_state, "html_url": f"https://example.test/issues/{index + 1}",
        }
        for index, node in enumerate(hierarchy["nodes"])
    }
    by_number = {issue["number"]: issue for issue in issues.values()}
    writes = []

    def api(args, *, input_text=None):
        path = args[args.index("PATCH") + 1] if "PATCH" in args else args[-1]
        issue = by_number[int(path.rsplit("/", 1)[1])]
        if input_text is not None:
            if "PATCH" not in args:
                raise AssertionError(f"Unexpected write: {args}")
            payload = json.loads(input_text)
            if set(payload) != {"state"}:
                raise AssertionError(f"Unexpected payload: {payload}")
            issue.update(payload)
            writes.append((issue["number"], payload["state"]))
        return dict(issue)

    def invoke(flags, *, state_opt_in=True):
        env = {"SEARCHRIGHT_GITHUB_APPLY": "1"}
        if state_opt_in:
            env["SEARCHRIGHT_GITHUB_TASK_STATE_APPLY"] = "1"
        output = io.StringIO()
        with (
            patch.object(sys, "argv", ["sync", *flags]),
            patch.dict(os.environ, env, clear=True),
            contextlib.redirect_stdout(output),
        ):
            result = sync.main()
        if result != 0:
            raise AssertionError(f"Sync failed: {output.getvalue()}")
        return json.loads(output.getvalue())

    with (
        patch.object(sync, "require_clean_tree"), patch.object(sync, "require_gh"),
        patch.object(sync, "run"), patch.object(sync, "ensure_labels"),
        patch.object(sync, "existing_issues", return_value=issues),
        patch.object(sync, "update_issue_if_needed", side_effect=lambda _r, i, _n: (i, "unchanged", [])),
        patch.object(sync, "attach_child", return_value="existing"),
        patch.object(sync, "run_json", side_effect=api),
    ):
        yield issues, writes, invoke


class ParentProjectionTests(unittest.TestCase):
    def test_completed_archived_track_and_phases_close_but_epic_stays_open(self):
        with projection() as (hierarchy, root):
            nodes = {n["key"]: n for n in hierarchy["nodes"]}
            self.assertEqual(nodes["track-00"]["desired_state"], "closed")
            self.assertEqual(nodes["track-00-phase-4"]["desired_state"], "closed")
            self.assertEqual(nodes["roadmap-epic"]["desired_state"], "open")
            self.assertEqual(nodes["track-00"]["project_fields"]["Delivery status"], "Done")
            self.assertIn("- [x] Phase 4", (root / nodes["track-00"]["body_path"]).read_text())
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(validator.main(), 0)

    def test_unfinished_external_child_keeps_its_phase_and_track_open(self):
        with projection(unfinished_phase=3) as (hierarchy, _root):
            nodes = {n["key"]: n for n in hierarchy["nodes"]}
            self.assertEqual(nodes["track-00-phase-3-task-01"]["desired_state"], "open")
            self.assertEqual(nodes["track-00-phase-3"]["desired_state"], "open")
            self.assertEqual(nodes["track-00"]["desired_state"], "open")
            self.assertEqual(nodes["track-00-phase-2"]["desired_state"], "closed")

    def test_active_blocked_or_unaccepted_track_does_not_close(self):
        for changes in (
            {"lifecycle": "active"}, {"blockers": ["Independent evaluation remains"]},
            {"blockers": None}, {"closeout_completed": False},
            {"implementation_state": "partially_implemented"},
        ):
            with self.subTest(changes=changes), projection(overrides=changes) as (hierarchy, _root):
                track = next(n for n in hierarchy["nodes"] if n["kind"] == "track")
                self.assertEqual(track["desired_state"], "open")

    def test_validator_rejects_parent_closure_that_hides_an_unfinished_child(self):
        with projection(unfinished_phase=3) as (hierarchy, root):
            for node in hierarchy["nodes"]:
                if node["key"] in {"track-00", "track-00-phase-3"}:
                    node["desired_state"] = "closed"
            (root / "conductor/github/issue-hierarchy.json").write_text(json.dumps(hierarchy))
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(validator.main(), 1)
            self.assertIn("desired state differs from canonical completion", output.getvalue())


class IssueStateSyncTests(unittest.TestCase):
    def test_all_issue_preview_exposes_reopening_without_remote_writes(self):
        with projection(unfinished_phase=3) as (hierarchy, root), remote_session(hierarchy, root, initial_state="closed") as (_, writes, invoke):
            plan = invoke(["--sync-issue-state", "--verbose-plan"])
            operation = next(x for x in plan["operations"] if x["key"] == "track-00-phase-3")
            self.assertEqual(operation["desired_state"], "open")
            self.assertTrue(operation["state_requested"])
            self.assertFalse(plan["issue_state_apply"])
            self.assertEqual(writes, [])

    def test_task_only_flag_preserves_parent_states(self):
        with projection() as (hierarchy, root), remote_session(hierarchy, root) as (issues, writes, invoke):
            receipt = invoke(["--apply", "--sync-task-state"])
            self.assertFalse(receipt["issue_state_apply"])
            self.assertTrue(writes)
            for node in hierarchy["nodes"]:
                expected = node["desired_state"] if node["kind"] == "task" else "open"
                self.assertEqual(issues[node["key"]]["state"], expected)

    def test_all_issue_apply_closes_completed_parents_and_preserves_epic(self):
        with projection() as (hierarchy, root), remote_session(hierarchy, root) as (issues, writes, invoke):
            receipt = invoke(["--apply", "--sync-issue-state"])
            self.assertTrue(receipt["issue_state_apply"])
            self.assertIn((issues["track-00"]["number"], "closed"), writes)
            self.assertIn((issues["track-00-phase-4"]["number"], "closed"), writes)
            self.assertEqual(issues["roadmap-epic"]["state"], "open")
            for node in hierarchy["nodes"]:
                self.assertEqual(issues[node["key"]]["state"], node["desired_state"])

    def test_all_issue_apply_reopens_unfinished_work_and_second_pass_is_idempotent(self):
        with projection(unfinished_phase=3) as (hierarchy, root), remote_session(hierarchy, root, initial_state="closed") as (issues, writes, invoke):
            receipt = invoke(["--apply", "--sync-issue-state"])
            self.assertTrue(receipt["issue_state_apply"])
            self.assertTrue(writes)
            self.assertEqual(issues["track-00"]["state"], "open")
            self.assertEqual(issues["track-00-phase-3"]["state"], "open")
            self.assertEqual(issues["track-00-phase-2"]["state"], "closed")
            first_writes = list(writes)
            second = invoke(["--apply", "--sync-issue-state"])
            self.assertEqual(writes, first_writes)
            self.assertTrue(all(x["state_action"] == "unchanged" for x in second["issues"]))

    def test_all_issue_flag_cannot_bypass_existing_state_opt_in(self):
        with projection() as (hierarchy, root), remote_session(hierarchy, root) as (_, writes, invoke):
            receipt = invoke(["--apply", "--sync-issue-state"], state_opt_in=False)
            self.assertFalse(receipt["issue_state_apply"])
            self.assertEqual(writes, [])

    def test_audit_parent_state_is_explicit_and_reports_real_drift(self):
        with projection() as (hierarchy, root), remote_session(hierarchy, root) as (issues, _writes, _invoke):
            for node in hierarchy["nodes"]:
                if node["kind"] == "task":
                    issues[node["key"]]["state"] = node["desired_state"]
            children = {}
            for node in hierarchy["nodes"]:
                if node["parent_key"]:
                    parent_number = issues[node["parent_key"]]["number"]
                    children.setdefault(parent_number, set()).add(issues[node["key"]]["id"])
            with (
                patch.object(audit, "existing_issues", return_value=issues),
                patch.object(audit, "child_ids", side_effect=lambda _r, n: children[n]),
            ):
                errors = []
                _, legacy = audit.compare_issues(hierarchy, errors)
                self.assertEqual(errors, [])
                self.assertIsNone(legacy["parent_state_drift"])
                _, expanded = audit.compare_issues(hierarchy, errors, check_issue_state=True)
                self.assertGreater(expanded["parent_state_drift"], 0)
                self.assertTrue(any("parent issues have state drift" in e for e in errors))
                for node in hierarchy["nodes"]:
                    issues[node["key"]]["state"] = node["desired_state"]
                errors = []
                _, converged = audit.compare_issues(hierarchy, errors, check_issue_state=True)
                self.assertEqual(errors, [])
                self.assertEqual(converged["parent_state_drift"], 0)


if __name__ == "__main__":
    unittest.main()
