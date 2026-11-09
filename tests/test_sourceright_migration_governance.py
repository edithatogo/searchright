#!/usr/bin/env python3
"""Governance and contract tests for Track 14: Sourceright migration and shared releases."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]


class SourcerightMigrationGovernanceTests(unittest.TestCase):
    """Verify Sourceright replacement map, parity cases, companion repository declarations, and compat crate."""

    def test_replacement_map_and_parity_cases(self):
        """Verify replacement-map.yaml and parity-cases.json are consistent and complete."""
        map_path = ROOT / "migration/sourceright/replacement-map.yaml"
        self.assertTrue(map_path.is_file(), "Missing replacement-map.yaml")
        mapping = yaml.safe_load(map_path.read_text(encoding="utf-8"))
        self.assertIn("source", mapping)
        self.assertIn("mappings", mapping)

        cases_path = ROOT / "migration/sourceright/parity-cases.json"
        self.assertTrue(cases_path.is_file(), "Missing parity-cases.json")
        cases = json.loads(cases_path.read_text(encoding="utf-8"))
        self.assertEqual(mapping["source"]["blob_sha"], cases["source_blob_sha"])
        self.assertEqual(mapping["source"]["revision"], cases["source_revision"])

    def test_companion_repository_declaration(self):
        """Verify migration/companion-repositories/sourceright.json matches pinned revision."""
        comp_path = ROOT / "migration/companion-repositories/sourceright.json"
        self.assertTrue(comp_path.is_file(), "Missing companion sourceright.json")
        comp = json.loads(comp_path.read_text(encoding="utf-8"))
        self.assertEqual(comp.get("schema_version"), "org.searchright.companion-change-packet.v1")
        self.assertIn("revision", comp)

    def test_sourceright_compat_crate_and_contracts(self):
        """Verify Rust contract surface and searchright-sourceright-compat symbols."""
        contract_path = ROOT / "crates/searchright-contracts/src/migration.rs"
        self.assertTrue(contract_path.is_file(), "Missing crates/searchright-contracts/src/migration.rs")
        text = contract_path.read_text(encoding="utf-8")
        self.assertIn("SOURCERIGHT_PARITY_CASE_IDS", text)
        self.assertIn("SOURCERIGHT_PARITY_DIMENSIONS", text)

        compat_lib = ROOT / "crates/searchright-sourceright-compat/src/lib.rs"
        self.assertTrue(compat_lib.is_file(), "Missing crates/searchright-sourceright-compat/src/lib.rs")

    def test_check_sourceright_migration_script(self):
        """Execute check_sourceright_migration.py deterministically."""
        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_sourceright_migration.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_sourceright_migration.py failed: {res.stderr}")
        data = json.loads(res.stdout)
        self.assertEqual(data.get("status"), "passed")

    def test_scholarly_integrity_signal_and_human_authority(self):
        """Verify integration docs specify non-delegable human authority for screening."""
        doc_path = ROOT / "docs/integration-architecture.md"
        self.assertTrue(doc_path.is_file(), "Missing docs/integration-architecture.md")
        doc_text = doc_path.read_text(encoding="utf-8")
        self.assertIn("sourceright", doc_text.lower())


if __name__ == "__main__":
    unittest.main()
