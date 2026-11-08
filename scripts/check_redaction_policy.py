#!/usr/bin/env python3
"""Validate the redaction profile and execute deterministic adversarial cases."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from redaction import load_profile, redact_value

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "policy" / "redaction-profile.json"


def connector_transport_errors(source: str) -> list[str]:
    """Check the known byte-reader source shape, not arbitrary Rust semantics."""
    start = source.find("async fn fetch_bytes(")
    end = source.find("fn decode_json(", start)
    if start < 0 or end <= start:
        return ["live connector byte-reader boundaries are missing"]
    fetch = source[start:end]
    status = fetch.find("let status = response.status()")
    body = fetch.find("while let Some(chunk) = response")
    if status < 0 or body <= status:
        return ["live connector request/body failure boundaries are missing"]
    errors = []
    for name, region in (("request", fetch[:status]), ("body", fetch[body:])):
        if "endpoint and query details were redacted" not in region:
            errors.append(f"live connector {name} failure lacks the redaction boundary")
        if "error.to_string()" in region:
            errors.append(f"live connector {name} failure may serialise a query-bearing error")
    return errors


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
    errors.extend(connector_transport_errors(connector_source))
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
