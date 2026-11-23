"""Deterministic claim-boundary checks for reference-only external directories.

These tests deliberately do not call directory APIs or infer a live provider.
"""

from __future__ import annotations

import json
from pathlib import Path
import re
import unittest
from urllib.parse import parse_qsl, urlsplit

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "data/source-selection/directories.v1.json"
SLUG = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")


class SourceDirectoryRegistryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = json.loads(REGISTRY.read_text(encoding="utf-8"))
        cls.rows = cls.data["directories"]

    def test_version_and_authority_are_explicit(self):
        self.assertEqual(
            self.data["schema_version"], "org.searchright.source-directory-register.v1"
        )
        self.assertEqual(self.data["authority"], "advisory_only")
        self.assertIn("not a supported Searchright provider", self.data["claim_boundary"])

    def test_directory_identifiers_are_unique_and_sorted(self):
        identifiers = [row["directory_id"] for row in self.rows]
        self.assertEqual(identifiers, sorted(identifiers))
        self.assertEqual(len(identifiers), len(set(identifiers)))
        self.assertTrue(all(SLUG.fullmatch(item) for item in identifiers))

    def test_every_directory_has_dated_first_party_evidence(self):
        self.assertGreaterEqual(len(self.rows), 8)
        for row in self.rows:
            with self.subTest(directory=row["directory_id"]):
                self.assertEqual(row["observed_on"], "2026-11-23")
                self.assertTrue(row["name"])
                for field in ("homepage", "first_party_evidence"):
                    value = urlsplit(row[field])
                    self.assertEqual(value.scheme, "https")
                    self.assertTrue(value.hostname)
                    self.assertFalse(value.username)
                    self.assertFalse(value.password)
                    forbidden_keys = {"token", "password", "api_key", "apikey", "access_token", "secret", "session"}
                    for key, _ in parse_qsl(value.query, keep_blank_values=True):
                        self.assertNotIn(key.casefold(), forbidden_keys)

    def test_reference_catalogue_never_grants_execution_or_ingestion(self):
        for row in self.rows:
            with self.subTest(directory=row["directory_id"]):
                self.assertIs(row["ingestion_authorized"], False)
                self.assertIs(row["execution_authorized"], False)
                self.assertEqual(row["documented_test_status"], "not_tested")
                self.assertEqual(row["review_state"], "intake")

    def test_reuse_decisions_are_explicit_and_conservative(self):
        for row in self.rows:
            self.assertEqual(
                row["copyright_decision"], "link_only_pending_source_specific_review"
            )
            self.assertTrue(row["rights_note"])
            self.assertLessEqual(len(row["rights_note"]), 240)
            self.assertTrue(row["recommended_use"])

    def test_distinct_directory_types_are_preserved(self):
        by_id = {row["directory_id"]: row for row in self.rows}
        self.assertEqual(
            by_id["dbis"]["directory_type"], "academic_database_directory"
        )
        self.assertEqual(
            by_id["fairsharing"]["directory_type"], "research_database_registry"
        )
        self.assertEqual(
            by_id["re3data"]["directory_type"], "research_data_repository_directory"
        )
        self.assertEqual(
            by_id["who-ictrp-network"]["directory_type"],
            "clinical_trial_registry_directory",
        )
        self.assertEqual(
            by_id["cda-amc-grey-matters"]["directory_type"],
            "grey_literature_sources_guide",
        )

    def test_api_documentation_does_not_promote_integration(self):
        by_id = {row["directory_id"]: row for row in self.rows}
        self.assertTrue(by_id["dbis"]["api_documentation"])
        self.assertTrue(by_id["re3data"]["api_documentation"])
        for row in self.rows:
            self.assertNotIn("live", row["documented_test_status"])
            if row["api_documentation"] is not None:
                self.assertEqual(urlsplit(row["api_documentation"]).scheme, "https")

    def test_issues_are_linked_but_not_claimed_native_subissues(self):
        for row in self.rows:
            expected = {"refhunter": 834, "dbis": 847, "re3data": 848}
            self.assertEqual(row["issue_number"], expected.get(row["directory_id"], 845))

    def test_no_directory_is_misrepresented_as_retrieval_provider(self):
        types = {row["directory_type"] for row in self.rows}
        self.assertNotIn("live_provider", types)
        self.assertNotIn("search_adapter", types)
        for row in self.rows:
            self.assertTrue(row["scope"])
            self.assertIn(row["priority"], {"high", "medium", "low"})

    def test_api_documentation_is_explicitly_versioned(self):
        by_id = {row["directory_id"]: row for row in self.rows}
        self.assertEqual(by_id["dbis"]["api_documentation"], "https://dbis-api.ur.de/api-description")
        self.assertEqual(by_id["re3data"]["api_documentation"], "https://www.re3data.org/api/doc/")

    def test_registry_is_deterministically_serialized(self):
        result = json.dumps(self.data, indent=2, ensure_ascii=True) + "\n"
        self.assertEqual(REGISTRY.read_text(encoding="utf-8"), result)


if __name__ == "__main__":
    unittest.main()
