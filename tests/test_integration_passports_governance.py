#!/usr/bin/env python3
"""Governance and contract tests for Track 13: Integration passports, GitHub issue hierarchy and context spine."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class IntegrationPassportsGovernanceTests(unittest.TestCase):
    """Verify integration passports, consumer contract suites, issue hierarchy, and context spine."""

    def test_integration_passports_index_and_schemas(self):
        """Verify integration/passports/index.json and integration-passport schema."""
        idx_path = ROOT / "integration/passports/index.json"
        self.assertTrue(idx_path.is_file(), "Missing integration/passports/index.json")
        data = json.loads(idx_path.read_text(encoding="utf-8"))
        self.assertEqual(data.get("schema_version"), "org.searchright.integration-passport-index.v1")
        self.assertGreaterEqual(len(data.get("passports", [])), 3)

        schema_path = ROOT / "contracts/json-schema/integration-passport.v1.schema.json"
        self.assertTrue(schema_path.is_file(), "Missing integration-passport schema")

        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_integration_passports.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_integration_passports.py failed: {res.stderr}")

    def test_consumer_contract_suite_and_schema(self):
        """Verify consumer contract suite and its verification script."""
        suite_path = ROOT / "integration/consumer-contract-suite.json"
        self.assertTrue(suite_path.is_file(), "Missing consumer-contract-suite.json")
        suite = json.loads(suite_path.read_text(encoding="utf-8"))
        self.assertEqual(suite.get("schema_version"), "org.searchright.consumer-contract-suite.v1")

        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_consumer_contracts.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_consumer_contracts.py failed: {res.stderr}")

    def test_github_issue_hierarchy_structure_and_depth(self):
        """Verify issue hierarchy JSON conforms to tree invariants."""
        hierarchy_path = ROOT / "conductor/github/issue-hierarchy.json"
        self.assertTrue(hierarchy_path.is_file(), "Missing conductor/github/issue-hierarchy.json")
        hierarchy = json.loads(hierarchy_path.read_text(encoding="utf-8"))
        self.assertEqual(hierarchy.get("schema_version"), "org.searchright.github-issue-hierarchy.v2")
        self.assertIn("nodes", hierarchy)

        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_github_issue_hierarchy.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_github_issue_hierarchy.py failed: {res.stderr}")

    def test_context_spine_and_manifest(self):
        """Verify context spine manifest and context integrity check."""
        manifest_path = ROOT / "context/manifest.json"
        self.assertTrue(manifest_path.is_file(), "Missing context/manifest.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        self.assertIn("required_context", manifest)

        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_context_integrity.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_context_integrity.py failed: {res.stderr}")

    def test_integration_drift_and_portfolio_project(self):
        """Verify check_integration_drift.py and check_portfolio_project.py pass."""
        res1 = subprocess.run(
            ["python3", str(ROOT / "scripts/check_integration_drift.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res1.returncode, 0, f"check_integration_drift.py failed: {res1.stderr}")

        res2 = subprocess.run(
            ["python3", str(ROOT / "scripts/check_portfolio_project.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res2.returncode, 0, f"check_portfolio_project.py failed: {res2.stderr}")


if __name__ == "__main__":
    unittest.main()
