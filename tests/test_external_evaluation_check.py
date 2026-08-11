"""Adversarial checks for Track 29's prepared evaluation contract."""
from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_external_evaluation", ROOT / "scripts/check_external_evaluation.py"
)
assert SPEC is not None and SPEC.loader is not None
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


class ExternalEvaluationContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.contract = json.loads(
            (ROOT / "evaluation/external-evaluation.json").read_text(encoding="utf-8")
        )

    def test_checked_in_preparation_contract_passes(self) -> None:
        self.assertEqual(CHECKER.validate(self.contract), [])

    def test_rejects_self_attested_preregistration(self) -> None:
        promoted = copy.deepcopy(self.contract)
        promoted["status"] = "preregistered"
        promoted["external_evidence"]["preregistration"] = "self-attested"
        errors = CHECKER.validate(promoted)
        self.assertIn("source contract must remain prepared_not_preregistered", errors)
        self.assertIn("prepared source contract cannot claim external evidence", errors)

    def test_rejects_observed_sustainability_without_external_receipts(self) -> None:
        promoted = copy.deepcopy(self.contract)
        promoted["sustainability"]["status"] = "observed"
        promoted["sustainability"]["observation_receipts"] = ["internal-note.json"]
        errors = CHECKER.validate(promoted)
        self.assertIn("source contract must remain prepared_not_observed", errors)
        self.assertIn("prepared source contract cannot claim observed maintenance", errors)

    def test_rejects_weakened_independence_and_topic_coverage(self) -> None:
        weakened = copy.deepcopy(self.contract)
        weakened["independence"]["minimum_information_specialists"] = 1
        weakened["design"]["topic_strata"].remove("policy")
        errors = CHECKER.validate(weakened)
        self.assertIn("at least two independent information specialists are required", errors)
        self.assertIn("evaluation topic strata are incomplete", errors)


if __name__ == "__main__":
    unittest.main()
