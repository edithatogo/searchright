#!/usr/bin/env python3
"""Validate the redaction profile and execute deterministic adversarial cases."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from redaction import load_profile, redact_value

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "policy" / "redaction-profile.json"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.parse_args()
    errors: list[str] = []
    profile = load_profile(PROFILE)
    if profile.get("replacement") != "[REDACTED]":
        errors.append("default replacement must remain [REDACTED]")
    if not set(profile.get("sensitive_query_keys", [])).issuperset({"api_key", "token", "email", "query"}):
        errors.append("sensitive query key baseline is incomplete")
    if not set(profile.get("sensitive_object_keys", [])).issuperset({"authorization", "password", "cookie"}):
        errors.append("sensitive object key baseline is incomplete")

    secret = "A1b2C3d4E5f6G7h8I9j0K1L2M3N4"
    payload = {
        "endpoint": f"https://user:{secret}@api.example.test/{secret}/search?db=pubmed&query=rare+disease&api_key={secret}&retmax=20",
        "authorization": f"Bearer {secret}",
        "headers": {
            "Cookie": f"session={secret}",
            "X-Api-Key": secret,
            "X-Trace": "public-trace",
        },
        "contact": "researcher@example.org",
        "nested": [f"token={secret}", {"client_secret": secret}],
    }
    redacted = redact_value(payload, profile)
    rendered = json.dumps(redacted, sort_keys=True)
    for forbidden in (secret, "rare+disease", "researcher@example.org", "session=", "user:"):
        if forbidden in rendered:
            errors.append(f"adversarial payload leaked {forbidden!r}")
    endpoint = redacted.get("endpoint", "")
    if "db=pubmed" not in endpoint or "retmax=20" not in endpoint:
        errors.append("redaction removed safe, non-sensitive endpoint controls")
    if endpoint.startswith("https://user") or "@api.example.test" in endpoint:
        errors.append("redaction retained URL user information")
    if rendered.count("[REDACTED]") < 5:
        errors.append("adversarial payload did not exercise expected redaction paths")
    if redact_value(payload, profile) != redacted:
        errors.append("redaction is not deterministic")
    connector_source = (ROOT / "crates" / "searchright-connectors" / "src" / "lib.rs").read_text(encoding="utf-8")
    live_start = connector_source.find("async fn fetch_json(")
    live_end = connector_source.find("fn open_manifest(", live_start)
    live_fetch = connector_source[live_start:live_end] if live_start >= 0 and live_end > live_start else ""
    if live_fetch.count("endpoint and query details were redacted") < 2:
        errors.append("live connector request failures do not preserve the redaction claim boundary")
    request_failure = live_fetch[: live_fetch.find("let status = response.status()")]
    body_start = live_fetch.find("let bytes = response")
    body_end = live_fetch.find("let maximum = request", body_start)
    body_failure = live_fetch[body_start:body_end]
    if "error.to_string()" in request_failure or "error.to_string()" in body_failure:
        errors.append("live connector transport failure may serialise a query-bearing error")
    receipt = {
        "schema_version": "org.searchright.redaction-policy-receipt.v1",
        "status": "failed" if errors else "passed",
        "profile_id": profile.get("profile_id"),
        "adversarial_cases": 1,
        "replacement_count": rendered.count("[REDACTED]"),
        "errors": errors,
        "limitations": [
            "Pattern-based receipt minimisation cannot prove that arbitrary free text contains no personal, licensed or confidential information. Raw payload persistence remains disabled by default."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
