from __future__ import annotations

import hashlib
import importlib.util
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "audit_github_control_plane_source", ROOT / "scripts" / "audit_github_control_plane.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class SourceBindingTests(unittest.TestCase):
    def test_source_state_records_revision_tree_and_input_digests(self) -> None:
        responses = iter(
            [
                Mock(stdout=""),
                Mock(stdout="revision\n"),
                Mock(stdout="tree\n"),
            ]
        )
        expected = {
            path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in (MODULE.HIERARCHY_PATH, MODULE.PROJECT_PATH, MODULE.SETTINGS_PATH)
        }
        with patch.object(MODULE, "run", side_effect=lambda *_args, **_kwargs: next(responses)):
            observed = MODULE.source_state()
        self.assertEqual(observed["revision"], "revision")
        self.assertEqual(observed["tree"], "tree")
        self.assertTrue(observed["tracked_worktree_clean"])
        self.assertEqual(observed["tracked_status_sha256"], hashlib.sha256(b"").hexdigest())
        self.assertEqual(observed["input_sha256"], expected)

    def test_dirty_console_result_is_diagnostic_not_passed(self) -> None:
        self.assertEqual(
            MODULE.receipt_status([], tracked_worktree_clean=False),
            "diagnostic_dirty_source",
        )
        self.assertEqual(
            MODULE.receipt_status(["remote drift"], tracked_worktree_clean=False),
            "failed",
        )

    def test_receipt_write_rejects_dirty_tracked_source_before_github_calls(self) -> None:
        dirty = {
            "revision": "revision",
            "tree": "tree",
            "tracked_worktree_clean": False,
            "tracked_status_sha256": "status-digest",
            "input_sha256": {},
        }
        with TemporaryDirectory() as temporary:
            receipt_path = Path(temporary) / "audit.json"
            with (
                patch.object(sys, "argv", ["audit", "--receipt-path", str(receipt_path)]),
                patch.object(MODULE, "source_state", return_value=dirty),
                patch.object(MODULE, "require_gh") as require_gh,
                self.assertRaisesRegex(
                    MODULE.GitHubCommandError,
                    "durable audit receipt requires a clean tracked Git working tree",
                ),
            ):
                MODULE.main()
            require_gh.assert_not_called()
            self.assertFalse(receipt_path.exists())


if __name__ == "__main__":
    unittest.main()
