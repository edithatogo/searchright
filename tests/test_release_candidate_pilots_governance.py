"""Unit tests for Release-Candidate Rehearsal, Staged Pilots and Ecosystem Rehearsal Governance (Track 36)."""

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ReleaseCandidatePilotsGovernanceTests(unittest.TestCase):
    def setUp(self):
        self.rehearsal_data = json.loads(
            (ROOT / "release/rehearsal.json").read_text(encoding="utf-8")
        )
        self.registry_status = json.loads(
            (ROOT / "registry/status.json").read_text(encoding="utf-8")
        )

    def test_release_rehearsal_script_conformance(self):
        """Verify check_release_rehearsal.py runs deterministically and passes with valid schema."""
        proc = subprocess.run(
            ["python3", str(ROOT / "scripts/check_release_rehearsal.py")],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, f"check_release_rehearsal failed: {proc.stderr}")
        receipt = json.loads(proc.stdout)
        self.assertEqual(
            receipt.get("schema_version"),
            "org.searchright.release-rehearsal-receipt.v1",
        )
        self.assertEqual(receipt.get("status"), "passed")
        self.assertEqual(receipt.get("gates"), 11)
        self.assertEqual(receipt.get("pilot_profiles"), 3)
        self.assertEqual(receipt.get("errors"), [])

    def test_rehearsal_mandatory_safeguards_and_boundaries(self):
        """Verify release rehearsal contract enforces explicit unexecuted status, dry-run bounds, and rollback."""
        self.assertEqual(
            self.rehearsal_data.get("schema_version"),
            "org.searchright.release-rehearsal.v1",
        )
        self.assertEqual(
            self.rehearsal_data.get("status"),
            "prepared_not_executed",
        )
        self.assertFalse(self.rehearsal_data.get("automatic_release", True))
        self.assertFalse(self.rehearsal_data.get("automatic_registry_submission", True))
        self.assertTrue(self.rehearsal_data.get("rollback_required", False))

        required_gates = self.rehearsal_data.get("required_gates", [])
        expected_gates = {
            "clean_clone",
            "locked_cross_platform_build",
            "msrv_and_nightly_forward_compatibility",
            "unit_integration_property_metamorphic_tests",
            "coverage_mutation_fuzz_formal_harness",
            "fixture_and_authorised_live_provider_receipts",
            "mcp_cli_facade_interoperability",
            "sourceright_citeweft_downstream_canaries",
            "sbom_checksums_attestation_reproducibility",
            "accessibility_usability_security_privacy_review",
            "pilot_exit_and_rollback_rehearsal",
        }
        self.assertEqual(set(required_gates), expected_gates)
        self.assertIn("A prepared rehearsal is not an executed release candidate", self.rehearsal_data.get("claim_boundary", ""))

    def test_pilot_deployment_protocol_and_safety_bounds(self):
        """Verify pilot protocol strictly enforces deployment staging, no autonomous exclusions, and data boundaries."""
        protocol_path = ROOT / "docs/pilots/pilot-protocol.md"
        self.assertTrue(protocol_path.is_file())
        content = protocol_path.read_text(encoding="utf-8")

        normalized = " ".join(content.split())
        self.assertIn("fixture-only local", normalized)
        self.assertIn("institution self-hosting", normalized)
        self.assertIn("remote single-tenant", normalized)
        self.assertIn("No pilot may use autonomous final exclusions", normalized)
        self.assertIn("Restricted full text, licensed provider credentials and personal data remain deployment-controlled", normalized)

        profiles = self.rehearsal_data.get("pilot_profiles", [])
        self.assertEqual(
            sorted(profiles),
            sorted(["local_researcher", "institution_self_hosted", "remote_single_tenant"]),
        )

    def test_registry_preparation_and_explicit_blockers(self):
        """Verify ecosystem and registry rehearsal maintains explicit blockers across all submission targets."""
        self.assertEqual(
            self.registry_status.get("schema_version"),
            "searchright.registry-status.v1",
        )
        self.assertEqual(
            self.registry_status.get("overall_status"),
            "prepared_not_submitted",
        )
        targets = {t["target"]: t for t in self.registry_status.get("targets", [])}
        expected_targets = {
            "github",
            "crates.io",
            "github-container-registry",
            "official-mcp-registry",
            "glama",
            "smithery",
            "github-releases",
            "zenodo-osf",
            "joss",
        }
        self.assertEqual(set(targets.keys()), expected_targets)
        for name, entry in targets.items():
            self.assertIn(entry.get("status"), {"prepared", "not_started"}, f"Target {name} must not claim submitted or published status")
            self.assertIsNone(entry.get("evidence_url"), f"Target {name} evidence URL must be null prior to submission")
            self.assertTrue(len(entry.get("blockers", [])) > 0, f"Target {name} must declare explicit unresolved blockers")

    def test_release_candidate_workflow_invariants(self):
        """Verify release-candidate GitHub Actions workflow is gated on manual dispatch and dry-run defaults."""
        workflow_path = ROOT / ".github/workflows/release-candidate.yml"
        self.assertTrue(workflow_path.is_file())
        content = workflow_path.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", content)
        self.assertIn("default: false", content)
        self.assertIn("python scripts/check_release_rehearsal.py", content)
        self.assertIn("python scripts/check_roadmap_coverage.py", content)
        self.assertIn("python scripts/check_maturity_dossier.py", content)
        self.assertIn("python scripts/run_static_harness.py", content)

    def test_source_packaging_reproducibility_conformance(self):
        """Verify packaging reproducibility checker confirms deterministic build artifacts."""
        proc = subprocess.run(
            ["python3", str(ROOT / "scripts/check_packaging_reproducibility.py"), "--allow-dirty"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, f"check_packaging_reproducibility failed: {proc.stderr}")
        receipt = json.loads(proc.stdout)
        self.assertEqual(
            receipt.get("schema_version"),
            "org.searchright.packaging-reproducibility-receipt.v1",
        )
        self.assertEqual(receipt.get("status"), "passed")
        self.assertEqual(receipt.get("errors"), [])
        self.assertGreater(receipt.get("artifacts_compared", 0), 0)


if __name__ == "__main__":
    unittest.main()
