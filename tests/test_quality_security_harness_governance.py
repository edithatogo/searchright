"""Governance and verification unit tests for Track 16: Maximal quality, context and security harness."""

import json
from pathlib import Path
import unittest

REPO_ROOT = Path(__file__).resolve().parent.parent

class QualitySecurityHarnessGovernanceTests(unittest.TestCase):
    def test_track_16_metadata(self):
        meta_path = REPO_ROOT / "conductor" / "tracks" / "16-quality-security-harness" / "metadata.json"
        self.assertTrue(meta_path.exists())
        with meta_path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        self.assertEqual(data.get("track_id"), "16")
        self.assertEqual(data.get("status"), "source_implemented")
        self.assertEqual(data.get("implementation_state"), "source_implemented")

    def test_harness_matrix_specification(self):
        matrix_path = REPO_ROOT / "verification" / "harness-matrix.json"
        self.assertTrue(matrix_path.exists())
        with matrix_path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        self.assertIn("dimensions", data)
        self.assertGreaterEqual(len(data["dimensions"]), 4)

    def test_gate_catalog_integrity(self):
        catalog_path = REPO_ROOT / "verification" / "gate-catalog.json"
        self.assertTrue(catalog_path.exists())
        with catalog_path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        self.assertIn("gates", data)
        self.assertGreaterEqual(len(data["gates"]), 30)

    def test_traceability_assertions_mapped(self):
        trace_path = REPO_ROOT / "conductor" / "tracks" / "16-quality-security-harness" / "traceability.json"
        self.assertTrue(trace_path.exists())
        with trace_path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        assertions = data.get("assertions", [])
        self.assertEqual(len(assertions), 6)
        assertion_ids = {a.get("assertion_id") for a in assertions}
        self.assertEqual(assertion_ids, {f"T16-A00{i}" for i in range(1, 7)})
        for a in assertions:
            self.assertEqual(a.get("state"), "source_implemented")
            self.assertGreaterEqual(len(a.get("implementation_paths", [])), 1)
            self.assertGreaterEqual(len(a.get("deterministic_tests", [])), 1)

    def test_security_controls_and_policies(self):
        threat_model = REPO_ROOT / "docs" / "security" / "threat-model.md"
        redaction_profile = REPO_ROOT / "policy" / "redaction-profile.json"
        cargo_vet_gov = REPO_ROOT / "docs" / "security" / "cargo-vet-governance.md"
        self.assertTrue(threat_model.exists())
        self.assertTrue(redaction_profile.exists())
        self.assertTrue(cargo_vet_gov.exists())

if __name__ == "__main__":
    unittest.main()
