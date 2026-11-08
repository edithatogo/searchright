#!/usr/bin/env python3
"""Governance and contract tests for Track 37: Final mature 1.0 release and long-term operations."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest

from scripts.check_maturity_dossier import (
    DECISIONS,
    EXPECTED,
    READY_EVIDENCE_FIELDS,
    READY_STATES,
    validate,
    validate_status_snapshot,
)

ROOT = Path(__file__).resolve().parents[1]
DOSSIER_PATH = ROOT / "conductor/maturity-dossier.json"


class Maturity10GovernanceTests(unittest.TestCase):
    """Verify maturity dossier schema, domain invariants, and 1.0 release decision safeguards."""

    def setUp(self):
        self.assertTrue(DOSSIER_PATH.is_file(), f"Missing dossier at {DOSSIER_PATH}")
        self.dossier = json.loads(DOSSIER_PATH.read_text(encoding="utf-8"))

    def test_maturity_dossier_conformance_and_domain_coverage(self):
        """Verify maturity dossier schema and exact 17 domain coverage."""
        self.assertEqual(
            self.dossier.get("schema_version"),
            "org.searchright.maturity-dossier.v1",
        )
        self.assertEqual(self.dossier.get("target_version"), "1.0.0")
        self.assertIn(self.dossier.get("decision"), DECISIONS)

        domains = self.dossier.get("domains", [])
        self.assertEqual(len(domains), 17)
        domain_names = {d.get("domain") for d in domains}
        self.assertEqual(domain_names, EXPECTED)

        for d in domains:
            self.assertIsInstance(d.get("domain"), str)
            self.assertIsInstance(d.get("state"), str)
            self.assertTrue(d.get("state"), f"Domain {d.get('domain')} has empty state")
            self.assertIsInstance(
                d.get("critical_blocker"),
                bool,
                f"Domain {d.get('domain')} critical_blocker must be boolean",
            )

    def test_decision_boundary_and_critical_blockers(self):
        """Verify that not_ready retains critical blockers and passes static validation."""
        self.assertEqual(self.dossier.get("decision"), "not_ready")

        blockers = [d for d in self.dossier.get("domains", []) if d.get("critical_blocker")]
        self.assertTrue(
            len(blockers) > 0,
            "not_ready decision must retain at least one critical blocker",
        )

        errors = validate(self.dossier, check_documents=True)
        self.assertEqual(errors, [], f"Validation failed on repository dossier: {errors}")

    def test_release_decision_evidence_requirements(self):
        """Verify ready decision strictly demands complete release decision evidence."""
        # Simulated ready dossier with missing evidence fields
        ready_payload = {
            "schema_version": "org.searchright.maturity-dossier.v1",
            "target_version": "1.0.0",
            "decision": "ready",
            "domains": [
                {"domain": name, "state": "passed", "critical_blocker": False}
                for name in EXPECTED
            ],
            "claim": "All domains ready",
        }
        errors = validate(ready_payload, check_documents=False)
        self.assertIn("ready decision requires release_decision_evidence", errors)

        # Partially populated evidence
        ready_payload["release_decision_evidence"] = {
            "accountable_reviewer": "Lead Architect",
            "approved": False,
        }
        errors = validate(ready_payload, check_documents=False)
        self.assertIn("ready decision evidence requires explicit approval", errors)
        self.assertTrue(any("ready decision evidence is incomplete" in err for err in errors))

        # Fully populated evidence
        ready_payload["release_decision_evidence"] = {
            field: f"mock_{field}" for field in READY_EVIDENCE_FIELDS
        }
        ready_payload["release_decision_evidence"]["approved"] = True
        errors = validate(ready_payload, check_documents=False)
        self.assertEqual(errors, [])

    def test_maturity_documentation_contracts(self):
        """Verify mandatory markdown documentation contracts and release doctrine."""
        for relative_path in (
            "docs/maturity/1.0-gate.md",
            "docs/maturity/gap-register.md",
            "docs/maturity/release-decision.md",
        ):
            path = ROOT / relative_path
            self.assertTrue(path.is_file(), f"Mandatory doc missing: {relative_path}")
            content = path.read_text(encoding="utf-8")
            self.assertGreater(len(content), 50)

        # Confirm non-negotiable doctrine in release-decision.md
        decision_doc = (ROOT / "docs/maturity/release-decision.md").read_text(encoding="utf-8")
        norm_decision = " ".join(decision_doc.split())
        self.assertIn("Current decision: **not ready**.", norm_decision)
        self.assertIn(
            "Source implementation, closed GitHub issues or a populated Project cannot make the decision ready.",
            norm_decision,
        )

        # Confirm 1.0-gate.md domain requirements
        gate_doc = (ROOT / "docs/maturity/1.0-gate.md").read_text(encoding="utf-8")
        self.assertIn("Version 1.0 is a release decision, not a source-code milestone.", gate_doc)
        self.assertIn("PRESS/human calibration", gate_doc)

    def test_status_snapshot_validation_conformance(self):
        """Verify status snapshot checker validates committed snapshot and rejects volatile assertions."""
        snapshot_path = ROOT / "verification/receipts/project-status-snapshot.json"
        snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        project_status = (ROOT / "PROJECT_STATUS.md").read_text(encoding="utf-8")

        errors = validate_status_snapshot(
            snapshot,
            dossier_decision=self.dossier.get("decision"),
            readme=readme,
            project_status=project_status,
            cargo_lock_exists=(ROOT / "Cargo.lock").exists(),
        )
        self.assertEqual(errors, [])

        # Volatile phrase in project_status should be rejected
        volatile_status = "Local `main` and `origin/main` are identical in this test."
        errors = validate_status_snapshot(
            snapshot,
            dossier_decision=self.dossier.get("decision"),
            readme=readme,
            project_status=volatile_status,
            cargo_lock_exists=(ROOT / "Cargo.lock").exists(),
        )
        self.assertTrue(any("asserts volatile" in err for err in errors))

    def test_dossier_checker_script_execution(self):
        """Verify scripts/check_maturity_dossier.py executes successfully and returns valid receipt."""
        proc = subprocess.run(
            ["python3", str(ROOT / "scripts/check_maturity_dossier.py")],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, f"check_maturity_dossier failed: {proc.stderr}")
        receipt = json.loads(proc.stdout)
        self.assertEqual(receipt.get("status"), "passed")
        self.assertEqual(receipt.get("decision"), "not_ready")
        self.assertEqual(receipt.get("schema_version"), "org.searchright.maturity-dossier-receipt.v1")
        self.assertEqual(receipt.get("domains"), 17)


if __name__ == "__main__":
    unittest.main()
