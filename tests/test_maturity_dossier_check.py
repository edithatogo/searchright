"""Adversarial tests for fail-closed maturity and status-claim validation."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
from typing import Any
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_maturity_dossier", ROOT / "scripts" / "check_maturity_dossier.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def dossier() -> dict[str, Any]:
    return json.loads(
        (ROOT / "conductor" / "maturity-dossier.json").read_text(encoding="utf-8")
    )


def status_snapshot() -> dict[str, Any]:
    return json.loads(
        (ROOT / "verification/receipts/project-status-snapshot.json").read_text(
            encoding="utf-8"
        )
    )


def validate_current_snapshot(value: dict[str, Any]) -> list[str]:
    return MODULE.validate_status_snapshot(
        value,
        dossier_decision=dossier()["decision"],
        readme=(ROOT / "README.md").read_text(encoding="utf-8"),
        project_status=(ROOT / "PROJECT_STATUS.md").read_text(encoding="utf-8"),
        cargo_lock_exists=(ROOT / "Cargo.lock").is_file(),
    )


class MaturityDossierTests(unittest.TestCase):
    def test_current_not_ready_dossier_is_consistent(self) -> None:
        self.assertEqual(MODULE.validate(dossier(), check_documents=False), [])

    def test_ready_cannot_be_claimed_by_only_flipping_flags(self) -> None:
        value = dossier()
        value["decision"] = "ready"
        for row in value["domains"]:
            row["critical_blocker"] = False
        errors = MODULE.validate(value, check_documents=False)
        self.assertTrue(any("non-ready domains" in error for error in errors))
        self.assertIn("ready decision requires release_decision_evidence", errors)

    def test_ready_requires_complete_approved_decision_evidence(self) -> None:
        value = dossier()
        value["decision"] = "ready"
        for row in value["domains"]:
            row["critical_blocker"] = False
            row["state"] = "passed"
        value["release_decision_evidence"] = {
            "approved": False,
            "exact_git_commit": "abc",
        }
        errors = MODULE.validate(value, check_documents=False)
        self.assertTrue(any("evidence is incomplete" in error for error in errors))
        self.assertIn("ready decision evidence requires explicit approval", errors)

    def test_release_risk_exception_must_remain_accountable(self) -> None:
        value = dossier()
        value["release_risk_exceptions"] = [
            {"id": "RISK-1", "domain": "unknown", "disposition": "waived"}
        ]
        errors = MODULE.validate(value, check_documents=False)
        self.assertTrue(any("is incomplete" in error for error in errors))
        self.assertTrue(any("unknown domain" in error for error in errors))
        self.assertTrue(any("invalid disposition" in error for error in errors))

    def test_ready_decision_keeps_accepted_exception_visible(self) -> None:
        value = dossier()
        value["decision"] = "ready"
        for row in value["domains"]:
            row["critical_blocker"] = False
            row["state"] = "passed"
        value["release_decision_evidence"] = {
            field: "recorded" for field in MODULE.READY_EVIDENCE_FIELDS
        }
        value["release_decision_evidence"]["approved"] = True
        value["release_risk_exceptions"] = [
            {
                "id": "RISK-1",
                "domain": "security",
                "risk": "open risk",
                "disposition": "accepted",
                "approved_by": "accountable reviewer",
            }
        ]
        self.assertIn(
            "ready decision must enumerate every release risk exception",
            MODULE.validate(value, check_documents=False),
        )
        value["release_decision_evidence"]["release_risk_exceptions"] = ["RISK-1"]
        self.assertEqual(MODULE.validate(value, check_documents=False), [])

    def test_duplicate_or_non_boolean_domain_entries_fail_closed(self) -> None:
        value = dossier()
        value["domains"].append(copy.deepcopy(value["domains"][0]))
        value["domains"][0]["critical_blocker"] = 1
        errors = MODULE.validate(value, check_documents=False)
        self.assertTrue(any("maturity domains differ" in error for error in errors))
        self.assertTrue(any("boolean critical_blocker" in error for error in errors))

    def test_current_status_snapshot_and_documents_are_consistent(self) -> None:
        self.assertEqual(validate_current_snapshot(status_snapshot()), [])

    def test_status_snapshot_rejects_invalid_revision(self) -> None:
        value = status_snapshot()
        value["observed_main_revision"] = "abc"
        errors = validate_current_snapshot(value)
        self.assertIn(
            "observed_main_revision must be a lowercase 40-character SHA",
            errors,
        )

    def test_status_snapshot_rejects_promoted_exact_head_observation(self) -> None:
        value = status_snapshot()
        observation = value["exact_head_observations"][0]
        observation["head_sha"] = value["last_fully_admitted_revision"]
        observation["full_admission_matrix"] = True
        errors = validate_current_snapshot(value)
        self.assertTrue(any("head_sha differs" in error for error in errors))
        self.assertTrue(any("must not claim a full admission matrix" in error for error in errors))

    def test_status_snapshot_rejects_stale_lockfile_denial(self) -> None:
        value = status_snapshot()
        stale_readme = (
            (ROOT / "README.md").read_text(encoding="utf-8")
            + "\nThis environment has not produced Rust compilation, a committed `Cargo.lock`.\n"
        )
        errors = MODULE.validate_status_snapshot(
            value,
            dossier_decision=dossier()["decision"],
            readme=stale_readme,
            project_status=(ROOT / "PROJECT_STATUS.md").read_text(encoding="utf-8"),
            cargo_lock_exists=True,
        )
        self.assertIn("README denies compiler or committed lockfile evidence", errors)
        self.assertIn("README contradicts the committed Cargo.lock", errors)

    def test_status_snapshot_rejects_volatile_remote_state_claim(self) -> None:
        value = status_snapshot()
        stale_status = (
            (ROOT / "PROJECT_STATUS.md").read_text(encoding="utf-8")
            + "\nThe working tree is clean; there are no open pull requests.\n"
        )
        errors = MODULE.validate_status_snapshot(
            value,
            dossier_decision=dossier()["decision"],
            readme=(ROOT / "README.md").read_text(encoding="utf-8"),
            project_status=stale_status,
            cargo_lock_exists=True,
        )
        self.assertIn("PROJECT_STATUS asserts volatile working-tree state", errors)
        self.assertIn("PROJECT_STATUS asserts volatile pull-request state", errors)

    def test_status_snapshot_must_match_dossier_decision(self) -> None:
        value = status_snapshot()
        value["maturity_decision"] = "ready"
        self.assertIn(
            "snapshot maturity_decision differs from maturity dossier",
            validate_current_snapshot(value),
        )


if __name__ == "__main__":
    unittest.main()
