"""Bounded synthetic acceptance for the proposed EQUATOR discovery consumer."""
import copy
import json
import unittest

from scripts.check_equator_directory_reference import git_blob_sha, validate


class EquatorDirectoryReferenceTests(unittest.TestCase):
    def setUp(self):
        self.row = {
            "id": "equator:srqr",
            "page_url": "https://equator-network.org/reporting-guidelines/srqr/",
            "source_path": "reporting-guidelines/srqr/index.qmd",
            "source_blob_sha": "a" * 40,
            "metadata_status": "not_extracted",
        }
        self.catalog = {
            "schema_version": 1,
            "source_repository": "https://github.com/EQUATOR-Network/equator-website",
            "source_revision": "b" * 40,
            "source_directory": "reporting-guidelines",
            "rights_status": "not_assessed_for_individual_guidelines",
            "published_count": 1,
            "guidelines": [self.row],
        }
        self.ref = {
            "status": "proposed_not_active",
            "auto_promote": False,
            "default_network": False,
            "default_external_writes": False,
            "rights_status": "not_assessed_for_individual_guidelines",
            "upstream_repository": self.catalog["source_repository"],
            "upstream_revision": self.catalog["source_revision"],
            "expected_published_count": 1,
        }

    def check(self, data=None, **overrides):
        data = self.catalog if data is None else data
        content = (json.dumps(data, sort_keys=True) + "\n").encode()
        reference = {**self.ref, "producer_blob_sha": git_blob_sha(content), **overrides}
        return validate(reference, content)

    def test_accepts_exact_pinned_read_only_discovery_only(self):
        self.assertIsNone(self.check())

    def test_rejects_changed_bytes_and_revision_drift(self):
        source = (json.dumps(self.catalog, sort_keys=True) + "\n").encode()
        with self.assertRaisesRegex(ValueError, "pinned"):
            validate({**self.ref, "producer_blob_sha": "c" * 40}, source)
        with self.assertRaisesRegex(ValueError, "source_revision"):
            self.check(upstream_revision="d" * 40)

    def test_rejects_unqualified_content_rights_or_automatic_promotion(self):
        for change in (
            {"auto_promote": True},
            {"default_network": True},
            {"rights_status": "cleared"},
        ):
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.check(**change)
        for addition in (
            {"title": "not a qualified text source"},
            {"checklist_items": ["not approved"]},
        ):
            edited = copy.deepcopy(self.catalog)
            edited["guidelines"][0].update(addition)
            with self.assertRaisesRegex(ValueError, "unexpected row"):
                self.check(edited)

    def test_rejects_unknown_ids_and_dropped_or_development_records(self):
        for field, value in (
            ("id", "equator:wrong"),
            ("source_path", "reporting-guidelines/under-development/index.qmd"),
            ("source_blob_sha", ""),
            ("metadata_status", "source_reconciled"),
        ):
            edited = copy.deepcopy(self.catalog)
            edited["guidelines"][0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.check(edited)
        duplicate = copy.deepcopy(self.catalog)
        duplicate["guidelines"].append(duplicate["guidelines"][0])
        duplicate["published_count"] = 2
        with self.assertRaisesRegex(ValueError, "source identity"):
            self.check(duplicate)


if __name__ == "__main__":
    unittest.main()
