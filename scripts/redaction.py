#!/usr/bin/env python3
"""Deterministic receipt redaction helpers with no external dependencies."""
from __future__ import annotations

import copy
import json
import math
import re
from pathlib import Path
from typing import Any
from urllib.parse import parse_qsl, urlencode, urlsplit, urlunsplit

ROOT = Path(__file__).resolve().parents[1]
PROFILE_PATH = ROOT / "policy" / "redaction-profile.json"
EMAIL_RE = re.compile(r"(?<![\w.+-])[\w.+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![\w.-])")
BEARER_RE = re.compile(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{8,}")
SECRET_ASSIGNMENT_RE = re.compile(
    r"(?i)\b(api[_-]?key|token|password|secret|client[_-]?secret)\s*[:=]\s*([^\s,;]+)"
)
HIGH_ENTROPY_RE = re.compile(r"[A-Za-z0-9_~+/.=-]{20,}")


def load_profile(path: Path = PROFILE_PATH) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _entropy(value: str) -> float:
    if not value:
        return 0.0
    counts = {char: value.count(char) for char in set(value)}
    size = len(value)
    return -sum((count / size) * math.log2(count / size) for count in counts.values())


def redact_text(value: str, profile: dict[str, Any]) -> str:
    replacement = str(profile["replacement"])
    result = value
    if profile.get("redact_email_addresses"):
        result = EMAIL_RE.sub(replacement, result)
    if profile.get("redact_bearer_tokens"):
        result = BEARER_RE.sub(f"Bearer {replacement}", result)
    result = SECRET_ASSIGNMENT_RE.sub(lambda match: f"{match.group(1)}={replacement}", result)
    if profile.get("redact_probable_high_entropy_values"):
        minimum = int(profile.get("minimum_high_entropy_length", 20))
        def replace_entropy(match: re.Match[str]) -> str:
            token = match.group(0)
            if len(token) >= minimum and _entropy(token) >= 3.5 and any(ch.isdigit() for ch in token):
                return replacement
            return token
        result = HIGH_ENTROPY_RE.sub(replace_entropy, result)
    return result


def redact_url(value: str, profile: dict[str, Any]) -> str:
    parts = urlsplit(value)
    sensitive = {str(key).lower() for key in profile.get("sensitive_query_keys", [])}
    safe = {str(key).lower() for key in profile.get("safe_query_keys", [])}
    replacement = str(profile["replacement"])
    query: list[tuple[str, str]] = []
    for key, raw in parse_qsl(parts.query, keep_blank_values=True):
        lower = key.lower()
        if lower in sensitive or lower not in safe:
            query.append((key, replacement))
        else:
            query.append((key, redact_text(raw, profile)))
    # User information must never survive in a receipt URL. Rebuild the
    # authority from parsed host/port rather than carrying the original netloc.
    host = parts.hostname or ""
    if ":" in host and not host.startswith("["):
        host = f"[{host}]"
    try:
        port = parts.port
    except ValueError:
        port = None
    netloc = f"{host}:{port}" if port is not None else host
    path = "/".join(redact_text(segment, profile) for segment in parts.path.split("/"))
    return urlunsplit((parts.scheme, netloc, path, urlencode(query), ""))


def redact_value(value: Any, profile: dict[str, Any], key: str | None = None) -> Any:
    sensitive = {str(item).lower() for item in profile.get("sensitive_object_keys", [])}
    replacement = str(profile["replacement"])
    if key is not None and key.lower() in sensitive:
        return replacement
    if isinstance(value, dict):
        return {str(item_key): redact_value(item, profile, str(item_key)) for item_key, item in value.items()}
    if isinstance(value, list):
        return [redact_value(item, profile) for item in value]
    if isinstance(value, str):
        if value.startswith(("https://", "http://")):
            return redact_url(value, profile)
        return redact_text(value, profile)
    return copy.deepcopy(value)
