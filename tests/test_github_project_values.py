from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "sync_github_project", ROOT / "scripts" / "sync_github_project.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
observed_field_value = MODULE.observed_field_value


class ObservedFieldValueTests(unittest.TestCase):
    def test_reads_cli_mixed_case_dynamic_keys(self) -> None:
        item = {"moSCoW": "Must", "track ID": "31"}

        self.assertEqual(observed_field_value(item, "MoSCoW"), (True, "Must"))
        self.assertEqual(observed_field_value(item, "Track ID"), (True, "31"))

    def test_reads_mixed_case_keys_in_fields_object(self) -> None:
        item = {"fields": {"conductor Key": {"text": "track-31"}}}

        self.assertEqual(
            observed_field_value(item, "Conductor key"),
            (True, "track-31"),
        )

    def test_unknown_field_remains_fail_closed(self) -> None:
        self.assertEqual(observed_field_value({"title": "item"}, "Track ID"), (False, None))


if __name__ == "__main__":
    unittest.main()
