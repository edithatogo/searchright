"""Regression checks for canonical CSL-linked research artifacts."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from check_research_links import validate_links  # noqa: E402


class ResearchLinkTests(unittest.TestCase):
    def setUp(self) -> None:
        self.refs = [{"id": "ref1", "type": "article-journal"}]
        self.protocol = {"protocol_id": "R7", "version": "0.1.0", "status": "draft", "references": ["ref1"], "registration": {"status": "not_registered", "identifier": None}}
        self.publication = {"publication_id": "R7-primary", "protocol_id": "R7", "protocol_version": "0.1.0", "status": "draft", "results_evidence": [], "canonical_references": "research/references.csl.json"}

    def test_valid_draft(self) -> None:
        self.assertEqual(validate_links(self.refs, self.protocol, self.publication), [])

    def test_duplicate_reference_rejected(self) -> None:
        self.assertIn("duplicate CSL identifiers", validate_links(self.refs + self.refs, self.protocol, self.publication))

    def test_unresolved_reference_rejected(self) -> None:
        p = dict(self.protocol, references=["missing"])
        self.assertTrue(any("unresolved CSL" in e for e in validate_links(self.refs, p, self.publication)))

    def test_publication_protocol_mismatch_rejected(self) -> None:
        p = dict(self.publication, protocol_version="0.2.0")
        self.assertTrue(any("protocol version" in e for e in validate_links(self.refs, self.protocol, p)))

    def test_registration_requires_real_identifier(self) -> None:
        p = dict(self.protocol, registration={"status": "registered", "identifier": None})
        self.assertTrue(any("registration identifier" in e for e in validate_links(self.refs, p, self.publication)))

    def test_nondraft_publication_needs_results(self) -> None:
        p = dict(self.publication, status="ready")
        self.assertTrue(any("result evidence" in e for e in validate_links(self.refs, self.protocol, p)))

    def test_nonstring_csl_id_rejected(self) -> None:
        self.assertTrue(any("string CSL" in e for e in validate_links([{"id": 7, "type": "article-journal"}], self.protocol, self.publication)))


if __name__ == "__main__":
    unittest.main()
