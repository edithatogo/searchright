#!/usr/bin/env python3
"""Governance and contract tests for Track 02: Portable query AST and dialect compilers."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]

EXPECTED_DIALECTS = {
    "pub_med",
    "ovid_medline",
    "embase",
    "cinahl_ebsco",
    "psyc_info_ovid",
    "scopus",
    "web_of_science",
}


class QueryAstDialectsGovernanceTests(unittest.TestCase):
    """Verify seven-dialect corpus integrity, schema conformance, and non-negotiable boundaries."""

    def test_query_corpus_manifest_and_fixtures(self):
        """Verify the 7 dialect fixtures exist, match manifest hashes, and preserve native text."""
        index_path = ROOT / "contracts/query-corpus/index.json"
        self.assertTrue(index_path.is_file(), "Missing contracts/query-corpus/index.json")
        index = json.loads(index_path.read_text(encoding="utf-8"))
        self.assertEqual(index.get("schema_version"), "org.searchright.native-query-corpus.v1")

        fixtures = index.get("fixtures", [])
        self.assertEqual(len(fixtures), 7)
        dialects_found = set()

        import hashlib

        for fix in fixtures:
            dialect = fix.get("dialect")
            dialects_found.add(dialect)
            fixture_rel = fix.get("path")
            fixture_file = ROOT / fixture_rel
            self.assertTrue(fixture_file.is_file(), f"Missing fixture file: {fixture_rel}")

            raw_bytes = fixture_file.read_bytes()
            computed_sha = hashlib.sha256(raw_bytes).hexdigest()
            self.assertEqual(
                computed_sha,
                fix.get("sha256"),
                f"Hash mismatch for fixture {fixture_rel}",
            )
            self.assertGreater(len(raw_bytes), 20)

        self.assertEqual(dialects_found, EXPECTED_DIALECTS)

    def test_loss_matrix_integrity(self):
        """Verify the seven-dialect loss matrix specifies lossless or documented loss transformations."""
        loss_path = ROOT / "contracts/query-corpus/loss-matrix.json"
        self.assertTrue(loss_path.is_file(), "Missing contracts/query-corpus/loss-matrix.json")
        matrix = json.loads(loss_path.read_text(encoding="utf-8"))
        self.assertEqual(matrix.get("schema_version"), "org.searchright.query-dialect-loss-matrix.v1")

        dialects = matrix.get("dialects", [])
        self.assertEqual(len(dialects), 7)
        loss_dialects = {d.get("dialect") for d in dialects}
        self.assertEqual(loss_dialects, EXPECTED_DIALECTS)

        for entry in dialects:
            self.assertIn("supported_constructs", entry)
            self.assertIn("known_losses", entry)
            self.assertIsInstance(entry["supported_constructs"], list)
            self.assertIsInstance(entry["known_losses"], list)

    def test_native_search_strategy_schema_and_example(self):
        """Verify native search strategy schema and example payload validity."""
        schema_path = ROOT / "contracts/json-schema/native-search-strategy.v1.schema.json"
        self.assertTrue(schema_path.is_file(), "Missing native strategy schema")
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        self.assertEqual(schema.get("$schema"), "https://json-schema.org/draft/2020-12/schema")

        example_path = ROOT / "contracts/examples/native-search-strategy.json"
        self.assertTrue(example_path.is_file(), "Missing native strategy example")
        example = json.loads(example_path.read_text(encoding="utf-8"))
        self.assertEqual(example.get("schema_version"), "org.searchright.native-search-strategy.v1")
        self.assertIn("lines", example)
        self.assertGreaterEqual(len(example["lines"]), 1)

    def test_named_filter_pack_schema_and_example(self):
        """Verify named filter pack schema, example, and synthetic validation boundary."""
        schema_path = ROOT / "contracts/json-schema/named-filter-pack.v1.schema.json"
        self.assertTrue(schema_path.is_file(), "Missing named filter pack schema")
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        self.assertEqual(schema.get("$schema"), "https://json-schema.org/draft/2020-12/schema")

        example_path = ROOT / "contracts/examples/named-filter-pack.json"
        self.assertTrue(example_path.is_file(), "Missing named filter pack example")
        example = json.loads(example_path.read_text(encoding="utf-8"))
        self.assertEqual(example.get("schema_version"), "org.searchright.named-filter-pack.v1")
        self.assertIn("filters", example)

        evidence_note = ROOT / "contracts/named-filters/synthetic-validation-evidence.txt"
        self.assertTrue(evidence_note.is_file(), "Missing synthetic validation evidence note")
        content = evidence_note.read_text(encoding="utf-8")
        self.assertIn("synthetic", content.lower())

    def test_native_query_corpus_gate_execution(self):
        """Verify check_native_query_corpus.py passes cleanly network-free."""
        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_native_query_corpus.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 0, f"check_native_query_corpus failed: {res.stderr}")
        receipt = json.loads(res.stdout)
        self.assertEqual(receipt.get("status"), "passed")
        self.assertEqual(receipt.get("fixtures_checked"), 7)

    def test_traceability_assertions_mapped(self):
        """Verify all Track 02 assertions are source_implemented and mapped."""
        trace_path = ROOT / "conductor/tracks/02-query-ast-dialects/traceability.json"
        self.assertTrue(trace_path.is_file(), "Missing traceability.json")
        trace = json.loads(trace_path.read_text(encoding="utf-8"))
        self.assertEqual(trace.get("track_id"), "02")
        self.assertEqual(len(trace.get("assertions", [])), 6)
        for a in trace["assertions"]:
            self.assertEqual(a.get("state"), "source_implemented")
            self.assertEqual(a.get("mapping_confidence"), "symbol_and_gate_mapped")
            self.assertTrue(len(a.get("implementation_paths", [])) > 0)


if __name__ == "__main__":
    unittest.main()
