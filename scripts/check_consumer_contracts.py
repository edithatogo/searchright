#!/usr/bin/env python3
"""Validate consumer-driven integration contracts without executing downstream writes."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUITE = ROOT / "integration" / "consumer-contract-suite.json"
INDEX = ROOT / "integration" / "passports" / "index.json"
EXTERNAL_PREFIX = "external://"


def main() -> int:
    errors: list[str] = []
    suite = json.loads(SUITE.read_text(encoding="utf-8"))
    index = json.loads(INDEX.read_text(encoding="utf-8"))
    active = {entry["integration_id"] for entry in index.get("passports", [])}
    interactions = suite.get("interactions", [])
    seen: set[str] = set()
    covered: set[str] = set()
    local_paths = 0

    for interaction in interactions:
        identifier = interaction.get("id")
        integration_id = interaction.get("integration_id")
        if not isinstance(identifier, str) or not identifier:
            errors.append("interaction id must be non-empty")
            continue
        if identifier in seen:
            errors.append(f"duplicate interaction id {identifier}")
        seen.add(identifier)
        if integration_id not in active:
            errors.append(f"{identifier}: unknown active integration {integration_id}")
        else:
            covered.add(integration_id)
        if interaction.get("producer") == interaction.get("consumer"):
            errors.append(f"{identifier}: producer and consumer must differ")
        if interaction.get("automatic_promotion") is not False:
            errors.append(f"{identifier}: automatic_promotion must be false")
        if interaction.get("status") not in {
            "prepared_not_executed",
            "fixture_verified",
            "downstream_verified",
            "suspended",
        }:
            errors.append(f"{identifier}: invalid status")
        for field in (
            "producer_contracts",
            "consumer_contracts",
            "fixture_paths",
            "producer_gates",
            "consumer_gates",
        ):
            values = interaction.get(field)
            if not isinstance(values, list) or not values or any(
                not isinstance(value, str) or not value.strip() for value in values
            ):
                errors.append(f"{identifier}: {field} must contain non-empty strings")
                continue
            if field.endswith("contracts") or field == "fixture_paths":
                for value in values:
                    if value.startswith(EXTERNAL_PREFIX):
                        if "@" not in value or not value.rsplit("@", 1)[1]:
                            errors.append(f"{identifier}: external contract lacks revision: {value}")
                        continue
                    path = ROOT / value
                    local_paths += 1
                    if not path.is_file():
                        errors.append(f"{identifier}: local contract/fixture is missing: {value}")
        failure = interaction.get("failure_semantics")
        if not isinstance(failure, str) or not failure.strip():
            errors.append(f"{identifier}: failure_semantics must be non-empty")

    missing = sorted(active - covered)
    extra = sorted(covered - active)
    if missing:
        errors.append(f"active integrations without consumer contracts: {missing}")
    if extra:
        errors.append(f"consumer contracts without active passports: {extra}")

    receipt = {
        "schema_version": "org.searchright.consumer-contract-receipt.v1",
        "status": "failed" if errors else "passed",
        "active_integrations": len(active),
        "interactions_checked": len(interactions),
        "local_paths_checked": local_paths,
        "errors": errors,
        "limitations": [
            "This source-level gate verifies declarations and local fixtures; producer/downstream execution remains separate evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
