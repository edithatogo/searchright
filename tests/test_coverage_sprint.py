"""Regression tests for the Track 16 test inventory (not coverage measurement)."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "coverage_sprint", Path(__file__).resolve().parents[1] / "scripts/coverage_sprint.py"
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("Cannot load scripts/coverage_sprint.py; check repository layout")
coverage_sprint = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(coverage_sprint)


class CoverageSprintTests(unittest.TestCase):
    def test_mixed_sync_async_and_nested_modules_are_counted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative, source in {
                "demo/src/lib.rs": "#[test]\nfn sync() {}\n#[tokio::test]\nasync fn asynchronous() {}",
                "demo/src/nested/tests.rs": "#[test]\nfn nested() {}",
                "demo/tests/integration.rs": "#[test]\nfn sync() {}\n#[tokio::test]\nasync fn asynchronous() {}",
                "demo/tests/nested/cases.rs": "#[tokio::test]\nasync fn nested() {}",
                "demo/tests/fixture.txt": "#[test]",
            }.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            with patch.object(coverage_sprint, "CRATES", root):
                self.assertEqual(coverage_sprint.count_tests("demo"), (3, 3))

    def test_absent_source_and_test_directories_have_zero_items(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(coverage_sprint, "CRATES", Path(directory)):
                self.assertEqual(coverage_sprint.count_tests("missing"), (0, 0))
