#!/usr/bin/env python3
"""Validate Track 29's fail-closed external-evaluation preparation contract."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "evaluation/external-evaluation.json"
PREPARED_STATUS = "prepared_not_preregistered"
OBSERVATION_STATUS = "prepared_not_observed"
TOPICS = {"clinical", "economic", "policy", "scoping"}
OUTCOMES = {
    "seed_set_recall",
    "critical_press_findings",
    "unreported_lossy_translations",
    "reproduction_success",
    "consequential_error_rate",
    "unassisted_task_completion",
}
OBSERVATIONS = {
    "maintenance_release_history",
    "standards_surveillance_history",
    "deprecation_or_migration_exercise",
    "succession_rehearsal",
}


def validate(data: dict[str, object]) -> list[str]:
    errors: list[str] = []
    if data.get("schema_version") != "org.searchright.external-evaluation.v1":
        errors.append("unexpected schema_version")
    if data.get("status") != PREPARED_STATUS:
        errors.append("source contract must remain prepared_not_preregistered")

    independence = data.get("independence")
    if not isinstance(independence, dict):
        errors.append("independence must be an object")
    else:
        if independence.get("minimum_information_specialists", 0) < 2:
            errors.append("at least two independent information specialists are required")
        if independence.get("development_team_excluded_from_adjudication") is not True:
            errors.append("development team must be excluded from adjudication")
        if independence.get("conflicts_must_be_disclosed") is not True:
            errors.append("conflict disclosure must be mandatory")

    design = data.get("design")
    if not isinstance(design, dict):
        errors.append("design must be an object")
    else:
        if set(design.get("topic_strata", [])) != TOPICS:
            errors.append("evaluation topic strata are incomplete")
        if set(design.get("required_outcomes", [])) != OUTCOMES:
            errors.append("required outcome set differs")
        for field in ("blinded_adjudication", "living_review_required", "sealed_labels_required"):
            if design.get(field) is not True:
                errors.append(f"{field} must be true")

    external = data.get("external_evidence")
    expected_external = {
        "preregistration",
        "participant_attestation",
        "results",
        "response_to_findings",
    }
    if not isinstance(external, dict) or set(external) != expected_external:
        errors.append("external evidence slots differ")
    elif any(external.values()):
        errors.append("prepared source contract cannot claim external evidence")

    sustainability = data.get("sustainability")
    if not isinstance(sustainability, dict):
        errors.append("sustainability must be an object")
    else:
        if sustainability.get("status") != OBSERVATION_STATUS:
            errors.append("source contract must remain prepared_not_observed")
        if set(sustainability.get("required_observations", [])) != OBSERVATIONS:
            errors.append("sustainability observation set differs")
        if sustainability.get("observation_receipts") != []:
            errors.append("prepared source contract cannot claim observed maintenance")

    disclosures = data.get("disclosures")
    if not isinstance(disclosures, dict) or set(disclosures) != {
        "funding",
        "conflicts",
        "provider_relationships",
    }:
        errors.append("funding and conflict disclosures are incomplete")
    elif not all(isinstance(value, str) and value.strip() for value in disclosures.values()):
        errors.append("disclosures must be non-empty strings")

    boundary = data.get("claim_boundary")
    if not isinstance(boundary, str) or "not preregistration" not in boundary:
        errors.append("claim boundary must deny preregistration")
    return errors


def main() -> int:
    payload: dict[str, object] = {}
    errors: list[str]
    try:
        payload = json.loads(CONTRACT.read_text(encoding="utf-8"))
        errors = validate(payload) if isinstance(payload, dict) else ["contract root must be an object"]
    except (OSError, json.JSONDecodeError) as exc:
        errors = [f"cannot read evaluation contract: {exc}"]
    receipt = {
        "schema_version": "org.searchright.external-evaluation-check.v1",
        "status": "failed" if errors else "passed",
        "contract_status": payload.get("status") if not errors and isinstance(payload, dict) else None,
        "errors": errors,
        "limitations": [
            "Static preparation check only; no preregistration, participants, evaluation, publication or sustainability observation was evidenced."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
