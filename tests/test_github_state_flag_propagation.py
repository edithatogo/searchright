from __future__ import annotations

import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import bootstrap_github as bootstrap


class BootstrapStateFlagTests(unittest.TestCase):
    def test_preview_does_not_apply_and_records_all_issue_request(self):
        output = io.StringIO()
        with (
            patch.object(sys, "argv", ["bootstrap", "--sync-issue-state"]),
            patch.dict(os.environ, {}, clear=True),
            patch.object(bootstrap, "run") as run,
            contextlib.redirect_stdout(output),
        ):
            self.assertEqual(bootstrap.main(), 0)
        self.assertEqual(json.loads(output.getvalue())["mode"], "dry_run")
        self.assertTrue(json.loads(output.getvalue())["sync_issue_state"])
        run.assert_not_called()

    def test_apply_forwards_selected_mode_and_existing_authority(self):
        cases = [([], None), (["--sync-task-state"], "--sync-task-state"),
                 (["--sync-issue-state"], "--sync-issue-state"),
                 (["--sync-task-state", "--sync-issue-state"], "--sync-issue-state")]
        for flags, selected in cases:
            with self.subTest(flags=flags), TemporaryDirectory() as temporary, contextlib.ExitStack() as stack:
                calls = []

                def run(command, **kwargs):
                    if command[:3] == ["gh", "api", "user"]:
                        result = "edithatogo\n"
                    elif command[:3] == ["git", "branch", "--show-current"]:
                        result = "main\n"
                    elif command[:3] == ["gh", "project", "list"]:
                        result = "{}"
                    elif command[1] == "scripts/sync_github_issues.py":
                        calls.append((command, kwargs["env"]))
                        result = json.dumps({"issue_state_apply": selected == "--sync-issue-state",
                                             "state_scope": "all_issues" if selected == "--sync-issue-state" else "task_issues_only"})
                    elif command[1] == "scripts/sync_github_project.py":
                        result = "{}"
                    else:
                        raise AssertionError(f"Unexpected command: {command}")
                    return subprocess.CompletedProcess(command, 0, result, "")

                stack.enter_context(patch.object(bootstrap, "ROOT", Path(temporary)))
                stack.enter_context(patch.object(bootstrap, "run", side_effect=run))
                for name in ("require_clean_tree", "require_gh", "ensure_remote", "apply_repository_settings",
                             "ensure_environments", "ensure_ruleset", "set_repository_variables", "maybe_set_project_secret"):
                    stack.enter_context(patch.object(bootstrap, name, return_value="observed"))
                stack.enter_context(patch.object(sys, "argv", ["bootstrap", "--apply", *flags]))
                stack.enter_context(patch.dict(os.environ, {"SEARCHRIGHT_GITHUB_BOOTSTRAP_APPLY": "1"}, clear=True))
                output = stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                self.assertEqual(bootstrap.main(), 0)
                command, env = calls[0]
                self.assertEqual([x for x in command if x in {"--sync-task-state", "--sync-issue-state"}],
                                 [selected] if selected else [])
                self.assertEqual(env.get("SEARCHRIGHT_GITHUB_TASK_STATE_APPLY"), "1" if selected else None)
                self.assertEqual(env["SEARCHRIGHT_GITHUB_APPLY"], "1")
                self.assertEqual(json.loads(output.getvalue())["issue_sync"]["issue_state_apply"],
                                 selected == "--sync-issue-state")


class HostedStateFlagTests(unittest.TestCase):
    @staticmethod
    def workflow(name):
        return yaml.safe_load((ROOT / ".github/workflows" / name).read_text())

    @staticmethod
    def shell_arguments(step, *, all_issues, task_state):
        # Execute the real workflow shell with a harmless Python-command stand-in.
        command = 'python() { printf "%s\\n" "$@"; }\n' + step["run"]
        result = subprocess.run(["bash", "-eu", "-c", command], check=True, capture_output=True, text=True,
                                env={**os.environ, "SEARCHRIGHT_REPOSITORY": "edithatogo/searchright",
                                     "SEARCHRIGHT_SYNC_ISSUE_STATE": str(all_issues).lower(),
                                     "SEARCHRIGHT_SYNC_TASK_STATE": str(task_state).lower()})
        return result.stdout.splitlines()

    def test_hosted_apply_preserves_task_only_default_and_forwards_all_issue_opt_in(self):
        for name in ("github-control-plane.yml", "issue-sync.yml"):
            workflow = self.workflow(name)
            inputs = workflow.get("on", workflow.get(True))["workflow_dispatch"]["inputs"]
            self.assertFalse(inputs["sync_issue_state"]["default"])
            self.assertTrue(inputs["sync_task_state"]["default"])
            step = next(s for s in workflow["jobs"]["apply"]["steps"]
                        if "scripts/sync_github_issues.py" in s.get("run", ""))
            self.assertEqual(step["env"]["SEARCHRIGHT_SYNC_ISSUE_STATE"], "${{ inputs.sync_issue_state }}")
            for all_issues, task_state in ((False, False), (False, True), (True, False), (True, True)):
                with self.subTest(workflow=name, all_issues=all_issues, task_state=task_state):
                    args = self.shell_arguments(step, all_issues=all_issues, task_state=task_state)
                    self.assertEqual("--sync-issue-state" in args, all_issues)
                    self.assertEqual("--sync-task-state" in args, task_state and not all_issues)

    def test_hosted_audit_checks_parent_state_only_after_all_issue_sync(self):
        workflow = self.workflow("github-control-plane.yml")
        step = next(s for s in workflow["jobs"]["apply"]["steps"]
                    if "scripts/audit_github_control_plane.py" in s.get("run", ""))
        self.assertEqual(step["env"]["SEARCHRIGHT_SYNC_ISSUE_STATE"], "${{ inputs.sync_issue_state }}")
        for enabled in (False, True):
            args = self.shell_arguments(step, all_issues=enabled, task_state=True)
            self.assertEqual("--check-issue-state" in args, enabled)


if __name__ == "__main__":
    unittest.main()
