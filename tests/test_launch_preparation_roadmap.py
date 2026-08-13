from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "launch_roadmap", ROOT / "scripts" / "check_launch_preparation_roadmap.py"
)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def valid_payload() -> dict:
    return {
        "schema_version": "org.searchright.launch-preparation-roadmap.v1",
        "status": "not_ready",
        "work_packages": [
            {
                "id": "LP-001",
                "owner_track": "30",
                "depends_on": [],
                "commands": ["python scripts/check_maturity_dossier.py"],
                "required_receipts": ["verification/receipts/example.json"],
                "exit_criterion": "A concrete criterion with reproducible evidence is required.",
                "external_gate": False,
                "progress": {
                    "status": "not_started",
                    "evidence_receipts": [],
                    "remaining_gates": [
                        "Produce the required reproducible completion evidence."
                    ],
                },
            }
        ],
    }


def test_valid_minimal_roadmap() -> None:
    assert MODULE.validate(valid_payload(), {"30"}) == []


def test_unknown_owner_fails_closed() -> None:
    payload = valid_payload()
    payload["work_packages"][0]["owner_track"] = "99"
    assert any("unknown owner_track" in error for error in MODULE.validate(payload, {"30"}))


def test_cycle_fails_closed() -> None:
    payload = valid_payload()
    payload["work_packages"][0]["depends_on"] = ["LP-001"]
    assert any("depends on itself" in error or "cycle" in error for error in MODULE.validate(payload, {"30"}))


def test_ready_flag_cannot_promote_plan() -> None:
    payload = valid_payload()
    payload["status"] = "ready"
    assert any("must remain not_ready" in error for error in MODULE.validate(payload, {"30"}))


def test_receipt_path_cannot_escape_repository_receipt_area() -> None:
    payload = valid_payload()
    payload["work_packages"][0]["required_receipts"] = [
        "verification/receipts/../../outside.json"
    ]
    assert any("unsafe receipt path" in error for error in MODULE.validate(payload, {"30"}))


def test_command_must_use_an_admitted_runner() -> None:
    payload = valid_payload()
    payload["work_packages"][0]["commands"] = ["curl https://example.test"]
    assert any("executable command" in error for error in MODULE.validate(payload, {"30"}))


def test_partial_progress_requires_existing_evidence() -> None:
    payload = valid_payload()
    payload["work_packages"][0]["progress"] = {
        "status": "partially_evidenced",
        "evidence_receipts": ["verification/receipts/missing.json"],
        "remaining_gates": ["Complete the residual higher-evidence launch gate."],
    }
    errors = MODULE.validate(payload, {"30"}, set())
    assert any("progress evidence does not exist" in error for error in errors)


def test_completed_package_requires_all_receipts_and_dependencies() -> None:
    payload = valid_payload()
    payload["work_packages"].append(
        {
            "id": "LP-002",
            "owner_track": "30",
            "depends_on": ["LP-001"],
            "commands": ["python scripts/check_maturity_dossier.py"],
            "required_receipts": ["verification/receipts/second.json"],
            "exit_criterion": "A second concrete criterion with reproducible evidence is required.",
            "external_gate": False,
            "progress": {
                "status": "completed",
                "evidence_receipts": ["verification/receipts/second.json"],
                "remaining_gates": [],
            },
        }
    )
    errors = MODULE.validate(
        payload,
        {"30"},
        {"verification/receipts/second.json"},
    )
    assert any("cannot complete before dependency LP-001" in error for error in errors)


def test_not_started_package_cannot_claim_evidence() -> None:
    payload = valid_payload()
    payload["work_packages"][0]["progress"]["evidence_receipts"] = [
        "verification/receipts/example.json"
    ]
    errors = MODULE.validate(
        payload,
        {"30"},
        {"verification/receipts/example.json"},
    )
    assert any("cannot attach evidence while not_started" in error for error in errors)
