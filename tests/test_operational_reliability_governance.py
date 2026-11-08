"""Unit tests for Operational Reliability, Backup, Restore and Incident Response Governance (Track 33)."""

import json
from pathlib import Path
import subprocess
import sys
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]


class OperationalReliabilityGovernanceTests(unittest.TestCase):
    def setUp(self):
        self.health = json.loads(
            (ROOT / "contracts/examples/component-health.json").read_text(encoding="utf-8")
        )
        self.telemetry = json.loads(
            (ROOT / "contracts/examples/telemetry-policy.json").read_text(encoding="utf-8")
        )
        self.backup = json.loads(
            (ROOT / "contracts/examples/backup-manifest.json").read_text(encoding="utf-8")
        )
        self.incident = json.loads(
            (ROOT / "contracts/examples/incident-record.json").read_text(encoding="utf-8")
        )
        self.rehearsal = json.loads(
            (ROOT / "verification/recovery/rehearsal.json").read_text(encoding="utf-8")
        )

    def test_component_health_contract_and_readiness_boundaries(self):
        """Verify component health contract conforms to schema and bounds readiness."""
        self.assertEqual(
            self.health.get("schema_version"),
            "org.searchright.component-health.v1",
        )
        self.assertEqual(self.health.get("component"), "searchright-mcp")
        self.assertEqual(self.health.get("state"), "healthy")
        self.assertTrue(self.health.get("ready"))

        doc = (ROOT / "docs/operations/reliability.md").read_text(encoding="utf-8")
        self.assertIn("liveness", doc.lower())
        self.assertIn("readiness", doc.lower())
        self.assertIn("degraded", doc.lower())

    def test_telemetry_policy_default_disabled_and_prohibited_attributes(self):
        """Verify telemetry defaults to disabled and strictly prohibits sensitive attributes."""
        self.assertEqual(
            self.telemetry.get("schema_version"),
            "org.searchright.telemetry-policy.v1",
        )
        self.assertFalse(self.telemetry.get("enabled", True))
        self.assertEqual(self.telemetry.get("sampling_per_million"), 0)
        self.assertEqual(self.telemetry.get("retention_days"), 0)
        self.assertIsNone(self.telemetry.get("endpoint"))

        prohibited = set(self.telemetry.get("prohibited_attributes", []))
        self.assertIn("query_text", prohibited)
        self.assertIn("full_text", prohibited)
        self.assertIn("api_key", prohibited)
        self.assertIn("personal_identifier", prohibited)

    def test_encrypted_backup_manifest_and_restore_authorization(self):
        """Verify backup manifests mandate encryption, key reference, and restore testing."""
        self.assertEqual(
            self.backup.get("schema_version"),
            "org.searchright.backup-manifest.v1",
        )
        self.assertTrue(self.backup.get("encrypted"))
        self.assertIsNotNone(self.backup.get("key_reference"))
        self.assertEqual(self.backup.get("digest_algorithm"), "sha256")
        self.assertEqual(len(self.backup.get("digest", "")), 64)
        self.assertTrue(self.backup.get("restore_test_required"))

        doc = (ROOT / "docs/operations/backup-restore.md").read_text(encoding="utf-8")
        self.assertIn("encrypted", doc.lower())
        self.assertIn("restore", doc.lower())

    def test_incident_record_contract_and_resilience_objectives(self):
        """Verify incident record structure, containment actions, and postmortem requirements."""
        self.assertEqual(
            self.incident.get("schema_version"),
            "org.searchright.incident-record.v1",
        )
        self.assertEqual(self.incident.get("status"), "contained")
        self.assertTrue(self.incident.get("postmortem_required"))
        self.assertFalse(self.incident.get("data_exposure_suspected", True))
        self.assertGreater(len(self.incident.get("containment", [])), 0)

        doc = (ROOT / "docs/operations/incident-response.md").read_text(encoding="utf-8")
        self.assertIn("contained", doc.lower())
        self.assertIn("post-incident review", doc.lower())

    def test_recovery_rehearsal_execution(self):
        """Verify recovery rehearsal self-test executes cleanly and produces valid receipt."""
        proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/recovery_rehearsal.py"), "--self-test"],
            capture_output=True,
            text=True,
            check=True,
        )
        data = json.loads(proc.stdout)
        self.assertEqual(data.get("status"), "passed")
        self.assertTrue(data.get("atomic_replace_checked"))
        self.assertTrue(data.get("restore_idempotency_checked"))
        self.assertTrue(data.get("tamper_detection_checked"))

        # Check existing rehearsal receipt on disk
        self.assertEqual(self.rehearsal.get("status"), "passed")
        self.assertTrue(self.rehearsal.get("atomic_replace_checked"))

    def test_resilience_workflow_hardening(self):
        """Verify that resilience workflow enforces read-only contents and unpersisted credentials."""
        workflow_path = ROOT / ".github/workflows/resilience.yml"
        self.assertTrue(workflow_path.is_file())

        parsed = yaml.safe_load(workflow_path.read_text(encoding="utf-8"))
        self.assertEqual(parsed.get("permissions"), {"contents": "read"})

        jobs = parsed.get("jobs", {})
        resilience_job = jobs.get("resilience", {})
        steps = resilience_job.get("steps", [])
        checkout_step = next(s for s in steps if s.get("name") == "Checkout")
        self.assertFalse(checkout_step.get("with", {}).get("persist-credentials", True))


if __name__ == "__main__":
    unittest.main()
