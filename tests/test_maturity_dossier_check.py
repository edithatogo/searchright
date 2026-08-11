"""Adversarial tests for the maturity decision's fail-closed validator."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_maturity_dossier", ROOT / "scripts" / "check_maturity_dossier.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def dossier() -> dict:
    return json.loads(
        (ROOT / "conductor" / "maturity-dossier.json").read_text(encoding="utf-8")
    )


def test_current_not_ready_dossier_is_consistent() -> None:
    assert MODULE.validate(dossier(), check_documents=False) == []


def test_ready_cannot_be_claimed_by_only_flipping_flags() -> None:
    value = dossier()
    value["decision"] = "ready"
    for row in value["domains"]:
        row["critical_blocker"] = False
    errors = MODULE.validate(value, check_documents=False)
    assert any("non-ready domains" in error for error in errors)
    assert "ready decision requires release_decision_evidence" in errors


def test_ready_requires_complete_approved_decision_evidence() -> None:
    value = dossier()
    value["decision"] = "ready"
    for row in value["domains"]:
        row["critical_blocker"] = False
        row["state"] = "passed"
    value["release_decision_evidence"] = {"approved": False, "exact_git_commit": "abc"}
    errors = MODULE.validate(value, check_documents=False)
    assert any("evidence is incomplete" in error for error in errors)
    assert "ready decision evidence requires explicit approval" in errors


def test_release_risk_exception_must_remain_explicit_and_accountable() -> None:
    value = dossier()
    value["release_risk_exceptions"] = [
        {"id": "RISK-1", "domain": "unknown", "disposition": "waived"}
    ]
    errors = MODULE.validate(value, check_documents=False)
    assert any("is incomplete" in error for error in errors)
    assert any("unknown domain" in error for error in errors)
    assert any("invalid disposition" in error for error in errors)


def test_ready_decision_must_keep_accepted_exception_visible() -> None:
    value = dossier()
    value["decision"] = "ready"
    for row in value["domains"]:
        row["critical_blocker"] = False
        row["state"] = "passed"
    value["release_decision_evidence"] = {
        field: "recorded" for field in MODULE.READY_EVIDENCE_FIELDS
    }
    value["release_decision_evidence"]["approved"] = True
    value["release_risk_exceptions"] = [{
        "id": "RISK-1", "domain": "security", "risk": "open risk",
        "disposition": "accepted", "approved_by": "accountable reviewer",
    }]
    assert "ready decision must enumerate every release risk exception" in MODULE.validate(
        value, check_documents=False
    )
    value["release_decision_evidence"]["release_risk_exceptions"] = ["RISK-1"]
    assert MODULE.validate(value, check_documents=False) == []


def test_duplicate_or_non_boolean_domain_entries_fail_closed() -> None:
    value = dossier()
    value["domains"].append(copy.deepcopy(value["domains"][0]))
    value["domains"][0]["critical_blocker"] = 1
    errors = MODULE.validate(value, check_documents=False)
    assert any("maturity domains differ" in error for error in errors)
    assert any("boolean critical_blocker" in error for error in errors)
