#!/usr/bin/env python3
"""Governance and contract tests for Track 12: CiteWeft scholarly extraction and document evidence."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class CiteWeftGovernanceTests(unittest.TestCase):
    """Verify CiteWeft pinned dependencies, non-canonical boundaries, contracts, and integration."""

    def test_citeweft_compatibility_manifest_and_cargo_pins(self):
        """Verify integration/citeweft-compatibility.json and Cargo workspace pins."""
        manifest_path = ROOT / "integration/citeweft-compatibility.json"
        self.assertTrue(manifest_path.is_file(), "Missing integration/citeweft-compatibility.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        self.assertEqual(manifest.get("schema_version"), "org.searchright.citeweft-compatibility.v1")
        self.assertEqual(manifest.get("adapter_crate"), "searchright-citeweft")
        self.assertFalse(manifest.get("canonical_writes_permitted", True))
        upstream_commit = manifest.get("upstream_commit")
        self.assertTrue(upstream_commit, "Missing upstream_commit in compatibility manifest")

        cargo_path = ROOT / "Cargo.toml"
        cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
        citeweft_dep = cargo.get("workspace", {}).get("dependencies", {}).get("citeweft")
        self.assertIsInstance(citeweft_dep, dict)
        self.assertEqual(citeweft_dep.get("rev"), upstream_commit)

        # Ensure no other workspace member depends on citeweft directly
        members = cargo.get("workspace", {}).get("members", [])
        for member in members:
            member_cargo_path = ROOT / member / "Cargo.toml"
            if not member_cargo_path.is_file():
                continue
            member_cargo = tomllib.loads(member_cargo_path.read_text(encoding="utf-8"))
            deps = {}
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                deps.update(member_cargo.get(section, {}))
            if "citeweft" in deps:
                self.assertEqual(
                    member,
                    "crates/searchright-citeweft",
                    f"Forbidden direct citeweft dependency in {member}",
                )

    def test_document_evidence_schema_and_example(self):
        """Verify document evidence contract schema and fixture invariants."""
        schema_path = ROOT / "contracts/json-schema/document-evidence.v1.schema.json"
        self.assertTrue(schema_path.is_file(), "Missing document-evidence schema")
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        self.assertEqual(schema.get("$schema"), "https://json-schema.org/draft/2020-12/schema")

        example_path = ROOT / "contracts/examples/document-evidence.json"
        self.assertTrue(example_path.is_file(), "Missing document-evidence example")
        example = json.loads(example_path.read_text(encoding="utf-8"))
        self.assertEqual(example.get("schema_version"), "org.searchright.document-evidence.v1")
        self.assertFalse(example.get("canonical_write_permitted", True))
        self.assertFalse(example.get("retained_full_text", True))
        self.assertIn("references", example)
        self.assertIn("citation_callouts", example)
        self.assertIn("provenance", example)

    def test_non_canonical_write_and_span_retention_invariants(self):
        """Verify rust contract types enforce no canonical write and retained spans."""
        doc_contract = ROOT / "crates/searchright-contracts/src/document.rs"
        self.assertTrue(doc_contract.is_file(), "Missing crates/searchright-contracts/src/document.rs")
        text = doc_contract.read_text(encoding="utf-8")
        self.assertIn("canonical_write_permitted", text)
        self.assertIn("retained_full_text", text)
        self.assertIn("DocumentEvidence", text)
        self.assertIn("DocumentSpan", text)

        adr_path = ROOT / "docs/adrs/0013-citeweft-document-evidence-boundary.md"
        self.assertTrue(adr_path.is_file(), "Missing ADR 0013")
        adr = adr_path.read_text(encoding="utf-8")
        self.assertIn("documentevidence", adr.lower())
        self.assertIn("canonical", adr.lower())

    def test_check_citeweft_integration_script(self):
        """Execute check_citeweft_integration.py deterministically."""
        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_citeweft_integration.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_citeweft_integration.py failed: {res.stderr}")
        data = json.loads(res.stdout)
        self.assertEqual(data.get("status"), "passed")

    def test_downstream_sourceright_compatibility_boundary(self):
        """Verify docs explain downstream Sourceright compatibility and rollback boundaries."""
        docs_path = ROOT / "docs/citeweft-integration.md"
        self.assertTrue(docs_path.is_file(), "Missing docs/citeweft-integration.md")
        doc_text = docs_path.read_text(encoding="utf-8")
        self.assertIn("sourceright", doc_text.lower())
        self.assertIn("rollback", doc_text.lower())


if __name__ == "__main__":
    unittest.main()
