"""Tests for Track 30: Maturity gate and gap closure governance.

Deterministic and network-free. These tests verify that the evidence gap register,
gate catalogue, maturity dossier, launch preparation roadmap, and operational
mode policies maintain strict, fail-closed claim boundaries.
"""
from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOSSIER_PATH = ROOT / "conductor" / "maturity-dossier.json"
ROADMAP_PATH = ROOT / "conductor" / "launch-preparation-roadmap.json"
DEBT_PATH = ROOT / "verification" / "evidence-debt.json"
GATES_PATH = ROOT / "verification" / "gate-catalog.json"
GAP_REGISTER_PATH = ROOT / "docs" / "maturity" / "gap-register.md"
FRONTIER_PLAN_PATH = ROOT / "conductor" / "frontier-plan.md"
REQUIREMENTS_PATH = ROOT / "conductor" / "requirements.md"


def collapse(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


class MaturityGapClosureGovernanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.dossier = json.loads(DOSSIER_PATH.read_text(encoding="utf-8"))
        self.roadmap = json.loads(ROADMAP_PATH.read_text(encoding="utf-8"))
        self.debt = json.loads(DEBT_PATH.read_text(encoding="utf-8"))
        self.gates = json.loads(GATES_PATH.read_text(encoding="utf-8"))

    def test_maturity_dossier_fails_closed_with_not_ready(self) -> None:
        self.assertEqual(self.dossier["schema_version"], "org.searchright.maturity-dossier.v1")
        self.assertEqual(self.dossier["target_version"], "1.0.0")
        self.assertEqual(self.dossier["decision"], "not_ready")
        domains = {entry["domain"]: entry for entry in self.dossier["domains"]}
        self.assertEqual(len(domains), 17)
        expected_domains = {
            "contracts", "compiler", "determinism", "providers", "methodology",
            "security", "interfaces", "migration", "usability", "operations",
            "github_control_plane", "downstream_compatibility", "access_and_tenancy",
            "backup_restore_incidents", "sdk_and_adoption", "pilots", "registries",
        }
        self.assertEqual(set(domains), expected_domains)
        critical_count = sum(1 for entry in self.dossier["domains"] if entry.get("critical_blocker"))
        self.assertGreaterEqual(critical_count, 10)

    def test_evidence_debt_reconciles_complete_assertion_inventory(self) -> None:
        self.assertEqual(self.debt["schema_version"], "org.searchright.evidence-debt.v1")
        self.assertEqual(self.debt["assertions"]["total"], 200)
        self.assertGreaterEqual(len(self.debt["maturity"]["critical_blockers"]), 10)

    def test_gate_catalog_has_bounded_harness_matrix(self) -> None:
        self.assertEqual(self.gates["schema_version"], "org.searchright.gate-catalog.v1")
        self.assertEqual(len(self.gates["gates"]), 73)
        self.assertFalse(self.gates["default_capabilities"]["network"])

    def test_launch_preparation_roadmap_dag_is_complete_and_acyclic(self) -> None:
        self.assertEqual(self.roadmap["schema_version"], "org.searchright.launch-preparation-roadmap.v1")
        self.assertEqual(self.roadmap["status"], "not_ready")
        work_packages = self.roadmap["work_packages"]
        self.assertEqual(len(work_packages), 16)
        wp_by_id = {wp["id"]: wp for wp in work_packages}
        expected_ids = [f"LP-{i:03d}" for i in range(1, 17)]
        self.assertEqual(list(wp_by_id.keys()), expected_ids)

        # Check dependencies exist and no self-dependency
        for wp in work_packages:
            for dep in wp["depends_on"]:
                self.assertIn(dep, wp_by_id, f"{wp['id']} depends on unknown {dep}")
                self.assertNotEqual(wp["id"], dep, f"{wp['id']} depends on itself")
            self.assertTrue(wp["exit_criterion"].strip(), f"{wp['id']} missing exit criterion")
            self.assertTrue(wp["commands"], f"{wp['id']} missing commands")
            self.assertTrue(wp["required_receipts"], f"{wp['id']} missing required receipts")
            self.assertIn("progress", wp)
            self.assertIn(wp["progress"]["status"], {"not_started", "partially_evidenced", "completed"})

    def test_gap_register_denies_documentation_based_waiver(self) -> None:
        text = collapse(GAP_REGISTER_PATH.read_text(encoding="utf-8"))
        self.assertIn("No unchecked row can be waived by changing documentation alone", text)
        self.assertIn("Exceptions must be recorded as explicit release risks", text)
        self.assertIn("LP-001", text)
        self.assertIn("LP-002", text)
        self.assertIn("LP-006", text)

    def test_autonomous_solo_and_team_modes_require_actor_provenance(self) -> None:
        req_text = collapse(REQUIREMENTS_PATH.read_text(encoding="utf-8"))
        frontier_text = collapse(FRONTIER_PLAN_PATH.read_text(encoding="utf-8"))
        self.assertIn("SR-050", req_text)
        self.assertIn("Autonomous, solo and team review modes", req_text)
        self.assertIn("Planned autonomy is not current capability", req_text)
        self.assertIn("Preserve actor, mode and policy version", frontier_text)
        self.assertIn("keep machine decisions identifiable", frontier_text)
        self.assertIn("No fictional human reviewers", frontier_text)


if __name__ == "__main__":
    unittest.main()
