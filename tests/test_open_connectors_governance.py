#!/usr/bin/env python3
"""Governance and contract tests for Track 04: Open provider connectors MVP."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class OpenConnectorsGovernanceTests(unittest.TestCase):
    """Verify provider baselines, rate guidance, redaction policies, and connector integrity."""

    def test_provider_contract_baselines_integrity(self):
        """Verify integration/provider-contract-baselines.json defines expected providers and fixtures."""
        baseline_path = ROOT / "integration/provider-contract-baselines.json"
        self.assertTrue(baseline_path.is_file(), "Missing integration/provider-contract-baselines.json")
        baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
        self.assertEqual(baseline.get("schema_version"), "org.searchright.provider-contract-baselines.v1")

        providers = {p["provider_id"]: p for p in baseline.get("providers", [])}
        expected_providers = [
            "pubmed-esearch",
            "pubmed-esummary",
            "pubmed-efetch",
            "europe-pmc",
            "crossref",
            "openalex",
        ]
        for ep in expected_providers:
            self.assertIn(ep, providers, f"Provider {ep} missing from baselines")
            prov_data = providers[ep]
            fixture_rel = prov_data.get("fixture")
            self.assertTrue(fixture_rel, f"Provider {ep} missing fixture reference")
            fixture_path = ROOT / fixture_rel
            self.assertTrue(fixture_path.is_file(), f"Fixture file {fixture_path} does not exist")

    def test_provider_policies_integrity(self):
        """Verify provider policies index exists and enforces governance constraints."""
        policy_path = ROOT / "integration/provider-policies/index.json"
        self.assertTrue(policy_path.is_file(), "Missing integration/provider-policies/index.json")
        policy_index = json.loads(policy_path.read_text(encoding="utf-8"))
        self.assertEqual(policy_index.get("schema_version"), "org.searchright.provider-policy-set.v1")

        policies = {p["provider_id"]: p for p in policy_index.get("providers", [])}
        self.assertGreaterEqual(len(policies), 5)
        for name, details in policies.items():
            self.assertTrue(details.get("manual_review_required_before_live_release"))
            self.assertIn("minimum_interval_ms_without_key", details)

    def test_redaction_profile_integrity(self):
        """Verify policy/redaction-profile.json provides strict redaction patterns."""
        redaction_path = ROOT / "policy/redaction-profile.json"
        self.assertTrue(redaction_path.is_file(), "Missing policy/redaction-profile.json")
        redaction = json.loads(redaction_path.read_text(encoding="utf-8"))
        self.assertEqual(redaction.get("schema_version"), "org.searchright.redaction-profile.v1")
        sensitive_keys = redaction.get("sensitive_object_keys", [])
        self.assertGreaterEqual(len(sensitive_keys), 10)
        self.assertIn("api_key", sensitive_keys)
        self.assertIn("authorization", sensitive_keys)

    def test_documentation_and_design_specifications(self):
        """Verify required architecture and mapping documents exist."""
        required_docs = [
            "docs/pubmed-efetch-mapping.md",
            "docs/efetch-orchestration-design.md",
            "docs/provider-parser-integrity.md",
            "docs/provider-governance.md",
        ]
        for doc in required_docs:
            doc_path = ROOT / doc
            self.assertTrue(doc_path.is_file(), f"Missing required documentation: {doc}")

    def test_connector_crate_source_symbols(self):
        """Verify crates/searchright-connectors/src/lib.rs declares parser versions and core structs."""
        lib_rs = ROOT / "crates/searchright-connectors/src/lib.rs"
        self.assertTrue(lib_rs.is_file(), "Missing crates/searchright-connectors/src/lib.rs")
        lib_content = lib_rs.read_text(encoding="utf-8")
        self.assertIn("PROVIDER_PARSER_VERSION", lib_content)
        self.assertIn("PubMedProvider", lib_content)

        efetch_rs = ROOT / "crates/searchright-connectors/src/efetch.rs"
        self.assertTrue(efetch_rs.is_file(), "Missing crates/searchright-connectors/src/efetch.rs")
        efetch_content = efetch_rs.read_text(encoding="utf-8")
        self.assertIn("PubMedFetchRequest", efetch_content)

    def test_provider_contract_baseline_script_execution(self):
        """Execute check_provider_contract_baselines.py deterministically."""
        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_provider_contract_baselines.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_provider_contract_baselines.py failed: {res.stderr}")

    def test_provider_policies_script_execution(self):
        """Execute check_provider_policies.py deterministically."""
        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_provider_policies.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_provider_policies.py failed: {res.stderr}")


if __name__ == "__main__":
    unittest.main()
