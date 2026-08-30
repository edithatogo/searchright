"""Mutation checks for the native corpus's dialect-to-fixture evidence binding."""
from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_native_query_corpus", ROOT / "scripts/check_native_query_corpus.py"
)
assert SPEC is not None and SPEC.loader is not None
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


class NativeQueryCorpusTests(unittest.TestCase):
    def setUp(self) -> None:
        self.index = json.loads(GATE.INDEX.read_text(encoding="utf-8"))
        self.matrix = json.loads(GATE.LOSS_MATRIX.read_text(encoding="utf-8"))

    def run_gate(self) -> tuple[int, dict]:
        output = io.StringIO()
        # Mutate only parsed manifests; read and hash the unchanged fixture bytes.
        with patch.object(GATE.json, "loads", side_effect=[self.index, self.matrix]):
            with contextlib.redirect_stdout(output):
                status = GATE.main()
        return status, json.loads(output.getvalue())

    def assert_rejected(self, message: str) -> None:
        status, receipt = self.run_gate()
        self.assertEqual(status, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any(message in error for error in receipt["errors"]))

    def test_checked_in_corpus_passes_without_promoting_review(self) -> None:
        status, receipt = self.run_gate()
        self.assertEqual(status, 0)
        self.assertEqual(receipt["errors"], [])
        self.assertEqual(receipt["fixtures_checked"], 7)
        self.assertEqual(receipt["external_methodological_review"], "pending")
        self.assertIn("accountable owner", receipt["limitations"][0])

    def test_duplicate_loss_dialect_is_rejected(self) -> None:
        self.matrix["dialects"].append(copy.deepcopy(self.matrix["dialects"][0]))
        self.assert_rejected("exactly once")

    def test_missing_loss_dialect_is_rejected(self) -> None:
        self.matrix["dialects"].pop()
        self.assert_rejected("exactly once")

    def test_replaced_loss_dialect_is_rejected(self) -> None:
        self.matrix["dialects"][-1] = copy.deepcopy(self.matrix["dialects"][0])
        self.assert_rejected("exactly once")

    def test_wrong_dialect_fixture_is_rejected(self) -> None:
        self.matrix["dialects"][0]["fixture_id"] = "scopus-basic"
        self.assert_rejected("different dialect")

    def test_unknown_fixture_is_rejected(self) -> None:
        self.matrix["dialects"][0]["fixture_id"] = "missing-fixture"
        self.assert_rejected("unknown fixture")

    def test_duplicate_fixture_identity_is_rejected(self) -> None:
        self.index["fixtures"].append(copy.deepcopy(self.index["fixtures"][0]))
        self.assert_rejected("duplicate or empty fixture id")

    def test_fixture_byte_drift_is_rejected(self) -> None:
        self.index["fixtures"][0]["sha256"] = "0" * 64
        self.assert_rejected("sha256 mismatch")


if __name__ == "__main__":
    unittest.main()
