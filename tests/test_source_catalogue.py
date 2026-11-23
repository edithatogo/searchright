"""Offline regression tests for the advisory source-selection inventory (#836)."""

from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
MODULE = ROOT / "scripts/check_source_catalogue.py"
CATALOGUE = ROOT / "data/source-selection/catalogue.v1.json"


def load_checker():
    spec = importlib.util.spec_from_file_location("source_catalogue_checker", MODULE)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SourceCatalogueTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker()
        self.data = json.loads(CATALOGUE.read_text(encoding="utf-8"))

    def check_rejected(self, edited, phrase):
        problems = self.checker.validate_catalogue(edited)
        self.assertTrue(any(phrase in message for message in problems), problems)

    def test_baseline_is_valid_and_reference_only(self):
        self.assertEqual(self.checker.validate_catalogue(self.data), [])
        self.assertEqual(self.data["authority"], "advisory_only")
        self.assertGreaterEqual(len(self.data["sources"]), 8)

    def test_medline_platforms_are_distinct(self):
        medline = next(s for s in self.data["sources"] if s["source_id"] == "medline")
        self.assertEqual({p["platform_id"] for p in medline["platforms"]}, {"pubmed", "ovid"})
        self.assertTrue(any("non-MEDLINE" in note for note in medline["limitations"]))

    def test_cochrane_sources_share_platform_but_not_identity(self):
        central = next(s for s in self.data["sources"] if s["source_id"] == "cochrane-central")
        cdsr = next(s for s in self.data["sources"] if s["source_id"] == "cochrane-cdsr")
        self.assertNotEqual(central["source_id"], cdsr["source_id"])
        self.assertEqual(central["platforms"][0]["platform_id"], cdsr["platforms"][0]["platform_id"])

    def test_duplicate_source_identifier_is_rejected(self):
        data = copy.deepcopy(self.data)
        data["sources"].append(copy.deepcopy(data["sources"][0]))
        self.check_rejected(data, "duplicate source_id")

    def test_duplicate_source_platform_pair_is_rejected(self):
        data = copy.deepcopy(self.data)
        data["sources"][0]["platforms"].append(copy.deepcopy(data["sources"][0]["platforms"][0]))
        self.check_rejected(data, "duplicate source/platform pair")

    def test_platform_metadata_conflict_is_rejected(self):
        data = copy.deepcopy(self.data)
        cdsr = next(s for s in data["sources"] if s["source_id"] == "cochrane-cdsr")
        cdsr["platforms"][0]["platform_name"] = "Wrong platform name"
        self.check_rejected(data, "inconsistent platform")

    def test_live_provider_promotion_is_rejected(self):
        data = copy.deepcopy(self.data)
        data["sources"][0]["platforms"][0]["execution_evidence"] = "opt_in_live"
        self.check_rejected(data, "invalid execution_evidence")

    def test_missing_first_party_evidence_is_rejected(self):
        data = copy.deepcopy(self.data)
        data["sources"][0]["first_party_url"] = "https://refhunter.org/en/"
        self.check_rejected(data, "RefHunter is secondary")

    def test_disposition_requires_review_for_unverified_source(self):
        data = copy.deepcopy(self.data)
        data["sources"][0]["verification_state"] = "conflict_unresolved"
        data["sources"][0]["disposition"] = "assessed"
        self.check_rejected(data, "cannot be assessed")

    def test_retired_and_no_issue_source_are_permitted(self):
        data = copy.deepcopy(self.data)
        data["sources"][0]["disposition"] = "retired"
        data["sources"][0]["issue_number"] = None
        self.assertEqual(self.checker.validate_catalogue(data), [])

    def test_licensed_web_interfaces_are_not_api_contracts(self):
        for source_id, api, interface in (
            ("scopus", "elsevier-api", "scopus-web"),
            ("web-of-science-core", "web-of-science-api", "web-of-science-web"),
        ):
            source = next(s for s in self.data["sources"] if s["source_id"] == source_id)
            by_id = {p["platform_id"]: p for p in source["platforms"]}
            self.assertEqual(by_id[api]["execution_evidence"], "existing_contract_unverified")
            self.assertEqual(by_id[interface]["execution_evidence"], "not_claimed")
            self.assertIsNone(by_id[interface]["existing_contract_path"])

    def test_existing_contract_requires_a_path(self):
        data = copy.deepcopy(self.data)
        medline = next(s for s in data["sources"] if s["source_id"] == "medline")
        pubmed = next(p for p in medline["platforms"] if p["platform_id"] == "pubmed")
        pubmed["existing_contract_path"] = None
        self.check_rejected(data, "requires exact path")

    def test_repeated_evaluation_is_deterministic(self):
        first = self.checker.validate_catalogue(self.data)
        second = self.checker.validate_catalogue(copy.deepcopy(self.data))
        self.assertEqual(first, second)


if __name__ == "__main__":
    unittest.main()
