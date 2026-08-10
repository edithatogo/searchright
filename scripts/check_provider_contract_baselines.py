#!/usr/bin/env python3
"""Validate pinned, rights-clear provider response baselines without network access."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "integration" / "provider-contract-baselines.json"
CONNECTOR_SOURCE = ROOT / "crates" / "searchright-connectors" / "src" / "lib.rs"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pointer(value: object, path: str) -> object:
    current = value
    if path == "":
        return current
    for token in path.lstrip("/").split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        if isinstance(current, list):
            current = current[int(token)]
        elif isinstance(current, dict):
            current = current[token]
        else:
            raise KeyError(path)
    return current


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="refresh exact fixture digests")
    args = parser.parse_args()
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    errors: list[str] = []
    warnings: list[str] = []
    source = CONNECTOR_SOURCE.read_text(encoding="utf-8")
    seen: set[str] = set()
    for item in data.get("providers", []):
        provider = item.get("provider_id")
        if not isinstance(provider, str) or not provider:
            errors.append("provider baseline has no provider_id")
            continue
        if provider in seen:
            errors.append(f"duplicate provider baseline {provider}")
        seen.add(provider)
        endpoint = item.get("endpoint")
        parsed = urlparse(endpoint or "")
        if parsed.scheme != "https" or not parsed.hostname:
            errors.append(f"{provider}: endpoint must be an HTTPS origin")
        elif parsed.hostname not in item.get("allowed_hosts", []):
            errors.append(f"{provider}: endpoint host is not allowlisted")
        if endpoint and endpoint.split("?", 1)[0] not in source:
            errors.append(f"{provider}: connector source does not contain the declared endpoint")
        fixture_path = ROOT / str(item.get("fixture", ""))
        if not fixture_path.is_file():
            errors.append(f"{provider}: missing fixture {item.get('fixture')}")
            continue
        actual = digest(fixture_path)
        if args.write:
            item["fixture_sha256"] = actual
        elif item.get("fixture_sha256") != actual:
            errors.append(f"{provider}: fixture digest drifted")
        try:
            fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{provider}: malformed fixture: {exc}")
            continue
        for assertion in item.get("shape_assertions", []):
            try:
                actual_value = pointer(fixture, assertion["pointer"])
            except (KeyError, IndexError, ValueError, TypeError):
                errors.append(f"{provider}: missing fixture pointer {assertion.get('pointer')}")
                continue
            if "equals" in assertion and actual_value != assertion["equals"]:
                errors.append(
                    f"{provider}: {assertion['pointer']} expected {assertion['equals']!r}, found {actual_value!r}"
                )
            if assertion.get("nonempty") and not actual_value:
                errors.append(f"{provider}: {assertion['pointer']} must be non-empty")
        if item.get("live_canary_status") != "not_executed":
            warnings.append(f"{provider}: baseline unexpectedly records a live status")
    if args.write:
        MANIFEST.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    receipt = {
        "schema_version": "org.searchright.provider-contract-baseline-receipt.v1",
        "status": "failed" if errors else "passed",
        "providers_checked": len(seen),
        "mode": "write" if args.write else "check",
        "errors": errors,
        "warnings": warnings,
        "limitations": [
            "Static response-shape and fixture-integrity checks only; upstream APIs were not contacted.",
            "Rust parser behaviour requires compiler-backed fixture tests before promotion."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
