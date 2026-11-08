"""Unit tests for Cross-Repository Release Train and Ecosystem Integration Governance (Track 32)."""

import json
from pathlib import Path
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]


class CrossRepositoryReleaseTrainGovernanceTests(unittest.TestCase):
    def setUp(self):
        self.release_train = json.loads(
            (ROOT / "integration/release-train.json").read_text(encoding="utf-8")
        )
        self.ecosystem_lock = json.loads(
            (ROOT / "integration/ecosystem-lock.json").read_text(encoding="utf-8")
        )
        self.public_packages = json.loads(
            (ROOT / "release/public-packages.json").read_text(encoding="utf-8")
        )
        self.consumer_contracts = json.loads(
            (ROOT / "integration/consumer-contract-suite.json").read_text(encoding="utf-8")
        )

    def test_release_train_structure_and_order(self):
        """Verify release train schema, component uniqueness, and contiguous promotion order."""
        self.assertEqual(
            self.release_train.get("schema_version"),
            "org.searchright.integration-release-train.v1",
        )
        self.assertFalse(self.release_train.get("automatic_promotion", True))

        components = self.release_train.get("components", [])
        self.assertEqual(len(components), 3)

        orders = [c.get("promotion_order") for c in components]
        self.assertEqual(orders, [1, 2, 3])

        repos = [c.get("repository") for c in components]
        self.assertEqual(repos, ["edithatogo/citeweft", "edithatogo/searchright", "edithatogo/sourceright"])

        for c in components:
            passport = c.get("passport")
            if passport:
                self.assertTrue((ROOT / passport).is_file(), f"Passport missing: {passport}")

        for surface in ("ecosystem_lock", "contract_surface", "public_package_policy"):
            surface_path = self.release_train.get(surface)
            self.assertIsNotNone(surface_path)
            self.assertTrue((ROOT / surface_path).is_file(), f"Control surface missing: {surface_path}")

    def test_no_automatic_promotion_across_stages(self):
        """Verify that every stage and ecosystem lock strictly prohibits automatic promotion."""
        stages = self.release_train.get("stages", [])
        expected_stages = [
            "contract",
            "consumer_fixture",
            "compiler",
            "downstream_canary",
            "release_candidate",
            "promotion",
        ]
        stage_ids = [s.get("id") for s in stages]
        self.assertEqual(stage_ids, expected_stages)

        for stage in stages:
            self.assertFalse(
                stage.get("automatic", True),
                f"Stage {stage.get('id')} must not be automatic",
            )

        self.assertFalse(
            self.ecosystem_lock.get("automatic_updates", True),
            "Ecosystem lock must disable automatic updates",
        )

    def test_rollback_procedures_and_evidence_preservation(self):
        """Verify explicit rollback steps requiring evidence preservation without deletion."""
        rollback = self.release_train.get("rollback", [])
        self.assertGreaterEqual(len(rollback), 3)

        rollback_text = " ".join(rollback).lower()
        self.assertIn("restore", rollback_text)
        self.assertIn("disable", rollback_text)
        self.assertIn("do not delete", rollback_text)

    def test_public_package_policy_default_deny(self):
        """Verify that all packages default to non-publishable until compiler/release evidence."""
        packages = self.public_packages.get("packages", [])
        self.assertGreater(len(packages), 0)
        for pkg in packages:
            self.assertFalse(
                pkg.get("publish_ready", False),
                f"Package {pkg.get('name')} must not be marked publish_ready without compiler release evidence",
            )

    def test_consumer_contract_suite_conformance(self):
        """Verify consumer contract interactions, distinct producer/consumer, and fixture paths."""
        self.assertEqual(
            self.consumer_contracts.get("schema_version"),
            "org.searchright.consumer-contract-suite.v1",
        )
        interactions = self.consumer_contracts.get("interactions", [])
        self.assertGreaterEqual(len(interactions), 2)

        for interaction in interactions:
            self.assertNotEqual(interaction.get("producer"), interaction.get("consumer"))
            self.assertFalse(interaction.get("automatic_promotion", True))
            for fixture in interaction.get("fixture_paths", []):
                self.assertTrue((ROOT / fixture).is_file(), f"Fixture missing: {fixture}")

    def test_release_train_workflow_hardening(self):
        """Verify that integration-release-train workflow enforces read-only permissions and unpersisted credentials."""
        workflow_path = ROOT / ".github/workflows/integration-release-train.yml"
        self.assertTrue(workflow_path.is_file())

        content = workflow_path.read_text(encoding="utf-8")
        parsed = yaml.safe_load(content)

        # Enforce read-only permissions
        permissions = parsed.get("permissions")
        self.assertEqual(permissions, {"contents": "read"})

        # Verify checkout does not persist credentials
        jobs = parsed.get("jobs", {})
        train_job = jobs.get("contract-train", {})
        steps = train_job.get("steps", [])
        checkout_step = next(s for s in steps if s.get("name") == "Checkout")
        self.assertFalse(checkout_step.get("with", {}).get("persist-credentials", True))


if __name__ == "__main__":
    unittest.main()
