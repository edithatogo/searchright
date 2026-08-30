"""Network-free mutations of the exact Sourceright migration catalogue."""
from __future__ import annotations

import contextlib
import copy
import io
import json
from pathlib import Path
import runpy
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
CASES = ROOT / "migration/sourceright/parity-cases.json"
CONTRACT = ROOT / "crates/searchright-contracts/src/migration.rs"


class SourcerightMigrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.cases = json.loads(CASES.read_text())
        self.contract = CONTRACT.read_text()

    def run_gate(self) -> tuple[int, dict]:
        original = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == CASES:
                return json.dumps(self.cases)
            if path == CONTRACT:
                return self.contract
            return original(path, *args, **kwargs)

        output = io.StringIO()
        with patch.object(Path, "read_text", read_text), contextlib.redirect_stdout(output):
            with self.assertRaises(SystemExit) as result:
                runpy.run_path(str(ROOT / "scripts/check_sourceright_migration.py"), run_name="__main__")
        return result.exception.code, json.loads(output.getvalue())

    def assert_rejected(self, fragment: str) -> None:
        status, receipt = self.run_gate()
        self.assertEqual(status, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any(fragment in error for error in receipt["errors"]), receipt)

    def test_checked_in_catalogue_passes(self) -> None:
        status, receipt = self.run_gate()
        self.assertEqual(status, 0)
        self.assertEqual(receipt["errors"], [])
        self.assertEqual(receipt["cases_checked"], 7)
        self.assertIn("does not itself execute", receipt["limitations"][0])

    def test_each_missing_case_is_rejected(self) -> None:
        original = copy.deepcopy(self.cases)
        for case in original["cases"]:
            with self.subTest(case=case["case_id"]):
                self.cases = copy.deepcopy(original)
                self.cases["cases"] = [row for row in self.cases["cases"] if row["case_id"] != case["case_id"]]
                self.assert_rejected("exact catalogue once each")

    def test_invented_case_is_rejected(self) -> None:
        self.cases["cases"][0]["case_id"] = "invented-case"
        self.assert_rejected("exact catalogue once each")

    def test_duplicate_case_is_rejected(self) -> None:
        self.cases["cases"].append(copy.deepcopy(self.cases["cases"][0]))
        self.assert_rejected("exact catalogue once each")

    def test_dimension_moved_between_cases_is_rejected(self) -> None:
        dimension = self.cases["cases"][0]["dimensions"].pop()
        self.cases["cases"][1]["dimensions"].append(dimension)
        self.assert_rejected("exact case coverage")

    def test_each_missing_dimension_is_rejected(self) -> None:
        original = copy.deepcopy(self.cases)
        for index, case in enumerate(original["cases"]):
            for dimension in case["dimensions"]:
                with self.subTest(case=case["case_id"], dimension=dimension):
                    self.cases = copy.deepcopy(original)
                    self.cases["cases"][index]["dimensions"].remove(dimension)
                    self.assert_rejected("exact case coverage")

    def test_invented_and_duplicate_dimensions_are_rejected(self) -> None:
        original = copy.deepcopy(self.cases)
        for dimension in ["invented", original["cases"][0]["dimensions"][0]]:
            self.cases = copy.deepcopy(original)
            self.cases["cases"][0]["dimensions"].append(dimension)
            self.assert_rejected("exact case coverage")

    def test_malformed_case_id_is_rejected_without_exception(self) -> None:
        for value in [None, [], {}, " "]:
            self.cases["cases"][0]["case_id"] = value
            self.assert_rejected("nonblank strings")

    def test_malformed_dimensions_are_rejected_without_exception(self) -> None:
        for value in [None, "host policy", [[]], [None], [" "]]:
            self.cases["cases"][0]["dimensions"] = value
            self.assert_rejected("dimensions must be nonblank strings")

    def test_rust_case_drift_is_rejected(self) -> None:
        self.contract = self.contract.replace('"undeclared-host"', '"invented-case"', 1)
        self.assert_rejected("Rust SOURCERIGHT_PARITY_CASE_IDS differs")

    def test_rust_dimension_drift_is_rejected(self) -> None:
        self.contract = self.contract.replace('"host policy"', '"invented dimension"', 1)
        self.assert_rejected("Rust SOURCERIGHT_PARITY_DIMENSIONS differs")

    def test_unparseable_rust_catalogue_fails_closed(self) -> None:
        self.contract = self.contract.replace('"host policy",', 'concat!("host", " policy"),', 1)
        self.assert_rejected("Rust SOURCERIGHT_PARITY_DIMENSIONS differs")

    def test_duplicate_rust_catalogue_entry_fails_closed(self) -> None:
        self.contract = self.contract.replace('"host policy",', '"host policy", "host policy",', 1)
        self.assert_rejected("Rust SOURCERIGHT_PARITY_DIMENSIONS differs")


if __name__ == "__main__":
    unittest.main()
