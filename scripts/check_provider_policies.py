#!/usr/bin/env python3
"""Validate conservative provider policy manifests against runtime baselines."""
from __future__ import annotations

import json
import re
from datetime import date, timedelta
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
BASELINES = ROOT / "integration" / "provider-contract-baselines.json"
POLICIES = ROOT / "integration" / "provider-policies" / "index.json"
SECRET_NAME = re.compile(r"(?:secret|token|password|credential|api[_-]?key)$", re.I)
ALLOWED_STATUS = {
    "source_identified_not_legally_approved",
    "reviewed_with_evidence",
    "blocked",
}


def valid_https(value: object) -> bool:
    if not isinstance(value, str):
        return False
    parsed = urlparse(value)
    return parsed.scheme == "https" and bool(parsed.hostname)


def main() -> int:
    errors: list[str] = []
    baselines = json.loads(BASELINES.read_text(encoding="utf-8"))
    policies = json.loads(POLICIES.read_text(encoding="utf-8"))
    source_epoch = date.fromisoformat(policies["source_epoch"])
    baseline_by_id = {row["provider_id"]: row for row in baselines.get("providers", [])}
    policy_rows = policies.get("providers")
    if not isinstance(policy_rows, list) or not policy_rows:
        errors.append("provider policy set must contain providers")
        policy_rows = []
    policy_by_id: dict[str, dict] = {}
    for row in policy_rows:
        provider_id = row.get("provider_id") if isinstance(row, dict) else None
        if not isinstance(provider_id, str) or not provider_id:
            errors.append("provider policy has missing provider_id")
            continue
        if provider_id in policy_by_id:
            errors.append(f"duplicate provider policy {provider_id}")
        policy_by_id[provider_id] = row
        baseline = baseline_by_id.get(provider_id)
        if baseline is None:
            errors.append(f"provider policy {provider_id} has no response baseline")
            continue
        if row.get("endpoint") != baseline.get("endpoint"):
            errors.append(f"provider policy endpoint mismatch for {provider_id}")
        for field in ("endpoint", "documentation_url", "terms_or_usage_url", "privacy_url"):
            if not valid_https(row.get(field)):
                errors.append(f"{provider_id}.{field} must be an HTTPS URL")
        if row.get("credential_receipt_policy") not in {
            "never_persist", "contact_value_redacted", "not_applicable"
        }:
            errors.append(f"{provider_id} has invalid credential receipt policy")
        if row.get("raw_response_retention") != "disabled_by_default":
            errors.append(f"{provider_id} must disable raw response retention by default")
        if row.get("live_canary_requires_opt_in") is not True:
            errors.append(f"{provider_id} must require live-canary opt-in")
        if row.get("manual_review_required_before_live_release") is not True:
            errors.append(f"{provider_id} must require manual policy review before live release")
        try:
            checked_at = date.fromisoformat(str(row.get("source_checked_at")))
            review_due = date.fromisoformat(str(row.get("review_due")))
        except ValueError:
            errors.append(f"{provider_id} has invalid policy review dates")
        else:
            if checked_at > source_epoch:
                errors.append(f"{provider_id}.source_checked_at exceeds the reproducible source epoch")
            if review_due < source_epoch:
                errors.append(f"{provider_id} policy review was already due at the source epoch")
            if review_due > checked_at + timedelta(days=184):
                errors.append(f"{provider_id} policy review interval exceeds six months")
        status = row.get("policy_review_status")
        if status not in ALLOWED_STATUS:
            errors.append(f"{provider_id} has invalid review status {status!r}")
        evidence = row.get("review_evidence")
        if not isinstance(evidence, list):
            errors.append(f"{provider_id}.review_evidence must be an array")
        elif status == "reviewed_with_evidence" and not evidence:
            errors.append(f"{provider_id} claims review without evidence")
        if row.get("query_classification") not in {"public_metadata", "internal_review_data", "confidential"}:
            errors.append(f"{provider_id} has invalid query classification")
        if row.get("response_classification") != "public_metadata":
            errors.append(f"{provider_id} must not promote response data above public metadata without a new policy")
        envs = row.get("credential_environment_variables")
        if not isinstance(envs, list) or any(not isinstance(name, str) or not name for name in envs):
            errors.append(f"{provider_id}.credential_environment_variables must be string array")
        for key, value in row.items():
            if SECRET_NAME.search(key) and value not in (None, [], "never_persist", "not_applicable"):
                errors.append(f"{provider_id} embeds a value in secret-like field {key}")
    if set(policy_by_id) != set(baseline_by_id):
        errors.append(
            f"provider policy/baseline mismatch missing={sorted(set(baseline_by_id)-set(policy_by_id))} "
            f"extra={sorted(set(policy_by_id)-set(baseline_by_id))}"
        )
    if policies.get("automatic_approval") is not False:
        errors.append("provider policy set must prohibit automatic approval")
    receipt = {
        "schema_version": "org.searchright.provider-policy-receipt.v1",
        "status": "failed" if errors else "passed",
        "providers": len(policy_by_id),
        "reviewed_with_evidence": sum(
            1 for row in policy_by_id.values() if row.get("policy_review_status") == "reviewed_with_evidence"
        ),
        "manual_review_required": sum(
            1 for row in policy_by_id.values() if row.get("manual_review_required_before_live_release") is True
        ),
        "errors": errors,
        "limitations": [
            "Static policy-shape and cross-manifest validation only; no legal review, URL availability, terms acceptance or live compatibility is inferred."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
