#!/usr/bin/env python3
"""Validate pinned, rights-clear provider response baselines without network access."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from urllib.parse import urlparse
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "integration" / "provider-contract-baselines.json"
CONNECTOR_SOURCE = ROOT / "crates" / "searchright-connectors" / "src" / "lib.rs"
MAX_XML_FIXTURE_BYTES = 256 * 1024
PARSER_SOURCES = {
    "crates/searchright-connectors/src/lib.rs",
    "crates/searchright-connectors/src/efetch.rs",
}
XML_PATH = re.compile(r"[A-Za-z_][A-Za-z0-9_.-]*(?:/[A-Za-z_][A-Za-z0-9_.-]*)*")


def parser_source(item: dict) -> Path:
    declared = item.get("parser_source", "crates/searchright-connectors/src/lib.rs")
    if not isinstance(declared, str) or declared not in PARSER_SOURCES:
        raise ValueError("parser_source must name an allowed connector source file")
    return ROOT / declared


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


def fixture_errors(item: dict, raw: bytes) -> list[str]:
    """Check local bytes and declared shape, never execute the Rust parser."""
    errors = []
    if item.get("fixture_sha256") != hashlib.sha256(raw).hexdigest():
        errors.append("fixture digest drifted")
    format_name = item.get("format", "json")
    if format_name not in ("json", "xml"):
        return errors + ["fixture format must be json or xml"]
    try:
        if format_name == "xml":
            if len(raw) > MAX_XML_FIXTURE_BYTES:
                raise ValueError("XML fixture exceeds the static byte limit")
            text = raw.decode("utf-8")
            if re.search(r"<!\s*(?:DOCTYPE|ENTITY)\b", text, re.I):
                raise ValueError("DTD and entity declarations are forbidden")
            if re.search(r"&(?!(?:amp|lt|gt|quot|apos|#[0-9]+|#x[0-9A-Fa-f]+);)", text):
                raise ValueError("custom or malformed entity references are forbidden")
            fixture = ET.fromstring(text)
            root_name = item.get("xml_root")
            if not isinstance(root_name, str) or not root_name or fixture.tag != root_name:
                raise ValueError("XML root differs from the declared root")
        else:
            fixture = json.loads(raw.decode("utf-8"))
    except (ValueError, UnicodeError, ET.ParseError) as exc:
        return errors + [f"malformed fixture: {exc}"]
    for assertion in item.get("shape_assertions", []):
        label = assertion.get("path") if format_name == "xml" else assertion.get("pointer")
        try:
            if format_name == "xml":
                if not isinstance(label, str) or not XML_PATH.fullmatch(label):
                    raise ValueError("XML paths must be explicit relative element paths")
                matches = fixture.findall(label)
                if len(matches) != 1:
                    raise ValueError("XML path must identify exactly one element")
                actual_value = "".join(matches[0].itertext()).strip()
            else:
                actual_value = pointer(fixture, assertion["pointer"])
        except (KeyError, IndexError, ValueError, TypeError):
            errors.append(f"missing, ambiguous or invalid fixture path {label}")
            continue
        if "equals" in assertion and actual_value != assertion["equals"]:
            errors.append(f"{label} expected {assertion['equals']!r}, found {actual_value!r}")
        if assertion.get("nonempty") and not actual_value:
            errors.append(f"{label} must be non-empty")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="refresh exact fixture digests")
    args = parser.parse_args()
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    errors: list[str] = []
    warnings: list[str] = []
    seen: set[str] = set()
    for item in data.get("providers", []):
        provider = item.get("provider_id")
        if not isinstance(provider, str) or not provider:
            errors.append("provider baseline has no provider_id")
            continue
        if provider in seen:
            errors.append(f"duplicate provider baseline {provider}")
        seen.add(provider)
        try:
            source = parser_source(item).read_text(encoding="utf-8")
        except (ValueError, OSError) as exc:
            errors.append(f"{provider}: invalid parser source: {exc}")
            continue
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
        if item.get("format", "json") == "xml":
            with fixture_path.open("rb") as stream:
                raw = stream.read(MAX_XML_FIXTURE_BYTES + 1)
        else:
            raw = fixture_path.read_bytes()
        actual = hashlib.sha256(raw).hexdigest()
        if args.write:
            if item.get("format", "json") != "xml" or len(raw) <= MAX_XML_FIXTURE_BYTES:
                item["fixture_sha256"] = actual
        errors.extend(f"{provider}: {error}" for error in fixture_errors(item, raw))
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
            "XML checks cover bounded synthetic bytes, declared root and element paths only; they do not execute EFetch or the Rust parser.",
            "Rust parser behaviour requires compiler-backed fixture tests before promotion."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
