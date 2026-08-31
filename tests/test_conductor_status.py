"""Adversarial temporary-tree tests; never mutate the real Conductor records."""

import json
import importlib.util
import shutil
import tempfile
import unittest
from unittest.mock import patch
import contextlib
import io
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("conductor_status", Path(__file__).resolve().parents[1] / "scripts/conductor_status.py")
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
audit = MODULE.audit


class StatusTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        source = Path(__file__).resolve().parents[1]
        shutil.copytree(source / "conductor", self.root / "conductor")

    def edit(self, relative, transform):
        path = self.root / relative
        path.write_text(transform(path.read_text()))

    def test_native_counts_and_no_writes(self):
        before = {str(p): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
        report = audit(self.root)
        self.assertEqual(report["errors"], [])
        self.assertEqual(report["counts"]["tracks"], 38)
        self.assertEqual(report["counts"]["archived"], 7)
        self.assertEqual(report["isolation"]["state"], "unconfigured")
        self.assertEqual(before, {str(p): p.read_bytes() for p in self.root.rglob("*") if p.is_file()})

    def test_unsafe_registry_link(self):
        self.edit("conductor/tracks.md", lambda s: s.replace("tracks/00-foundation-conductor-toolchain/spec.md", "../../outside.md"))
        self.assertTrue(audit(self.root)["errors"])

    def test_wrong_local_target(self):
        self.edit("conductor/tracks.md", lambda s: s.replace("tracks/00-foundation-conductor-toolchain/spec.md", "tracks/00-foundation-conductor-toolchain/plan.md"))
        self.assertTrue(audit(self.root)["errors"])

    def test_metadata_mismatch(self):
        self.edit("conductor/tracks/00-foundation-conductor-toolchain/metadata.json", lambda s: s.replace('"track_id": "00"', '"track_id": "99"'))
        self.assertTrue(audit(self.root)["errors"])

    def test_archive_cannot_hide_open_task(self):
        self.edit("conductor/tracks/09-cli-mvp/plan.md", lambda s: s.replace("- [x]", "- [~]", 1))
        self.assertTrue(audit(self.root)["errors"])

    def test_ledger_optin_fails_closed(self):
        def optin(s):
            data = json.loads(s)
            data["evidence_schema"] = "1.0"
            return json.dumps(data)
        self.edit("conductor/tracks/00-foundation-conductor-toolchain/metadata.json", optin)
        self.assertTrue(audit(self.root)["errors"])

    def test_missing_or_duplicate_registration(self):
        self.edit("conductor/tracks.md", lambda s: s + next(line for line in s.splitlines() if line.startswith("| 00 |")) + "\n")
        self.assertTrue(audit(self.root)["errors"])

    def test_isolation_configuration_not_ownership(self):
        self.edit("conductor/workflow.md", lambda s: s + "\nisolation mode: worktree\n")
        report = audit(self.root)
        self.assertEqual(report["isolation"]["state"], "inconsistent")
        self.assertTrue(report["errors"])

    def test_malformed_json_record_shapes_fail_structurally(self):
        for filename in ("metadata.json", "evidence.json"):
            for value in ([], None, True, "text"):
                with self.subTest(filename=filename, value=value):
                    path = self.root / "conductor/tracks/00-foundation-conductor-toolchain" / filename
                    original = path.read_text()
                    path.write_text(json.dumps(value))
                    self.assertEqual(audit(self.root)["status"], "failed")
                    path.write_text(original)

    def test_truncated_registry_row_returns_failed_report(self):
        self.edit("conductor/tracks.md", lambda s: s + "\n| 00 |\n")
        self.assertEqual(audit(self.root)["status"], "failed")

    def test_consistent_unknown_lifecycle_is_not_valid(self):
        def change_entry(s):
            data = json.loads(s)
            if "tracks" in data:
                data["tracks"][0]["lifecycle"] = "invented-lifecycle"
            else:
                data["lifecycle"] = "invented-lifecycle"
            return json.dumps(data)
        for path in ("conductor/roadmap-coverage.json",
                     "conductor/tracks/00-foundation-conductor-toolchain/metadata.json",
                     "conductor/tracks/00-foundation-conductor-toolchain/evidence.json"):
            self.edit(path, change_entry)
        self.assertEqual(audit(self.root)["status"], "failed")

    def test_cli_never_executes_code_from_inspected_root(self):
        scripts = self.root / "scripts"
        scripts.mkdir()
        marker = self.root / "executed"
        payload = f"from pathlib import Path\nPath({str(marker)!r}).touch()\n"
        for name in ("check_roadmap_coverage.py", "sync_track_evidence.py"):
            (scripts / name).write_text(payload)
        output = io.StringIO()
        with patch("sys.argv", ["conductor_status.py", "--root", str(self.root)]), contextlib.redirect_stdout(output):
            self.assertEqual(MODULE.main(), 0)
        self.assertFalse(marker.exists())
        self.assertEqual(json.loads(output.getvalue())["status"], "passed")


if __name__ == "__main__":
    unittest.main()
