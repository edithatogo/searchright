#!/usr/bin/env python3
"""Offline CSL-JSON 1.0.2 input preflight; not canonicalisation or rights approval.

The original MIT-licensed upstream schema is pinned to commit
506040022f6c37846343edd36658b23e85b5b8ff (v1.0.2), Git blob
085a8124fded125c0f765ac56e340fbc7be75496:
https://github.com/citation-style-language/schema/blob/506040022f6c37846343edd36658b23e85b5b8ff/schemas/input/csl-data.json
The adjacent LICENSE.txt preserves the upstream notice. The schema alone is
not fully normative. Searchright additionally requires unique nonblank string
IDs (the local whitespace check is conservative). The 32 MiB preflight limit
is not a corpus-size limit for production Searchright. This gate never rewrites
fields, computes the BLAKE3 corpus revision,
asserts source rights, or accepts a downstream consumer.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
from typing import Any

from jsonschema import Draft7Validator
from referencing import Registry

SCHEMA = Path(__file__).resolve().parents[1] / "contracts/vendor/csl/1.0.2/csl-data.json"
SCHEMA_SHA256 = "257f86d4c3f19e15c7208657512ad4c9559a3b0d83b03360ed006c5f02d2977c"
MAX_BYTES = 32 * 1024 * 1024


class CorpusError(ValueError):
    """An input cannot be admitted by this bounded preflight."""


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CorpusError("duplicate JSON member")
        result[key] = value
    return result


def finite_float(value: str) -> float:
    parsed = float(value)
    if not math.isfinite(parsed):
        raise CorpusError("non-finite JSON number")
    return parsed


def check(raw: bytes) -> int:
    """Return validated item count, or a redacted failure; never rewrite input."""
    if len(raw) > MAX_BYTES:
        raise CorpusError("corpus exceeds preflight byte limit")
    try:
        schema_raw = SCHEMA.read_bytes()
    except OSError:
        raise CorpusError("pinned schema unavailable") from None
    if hashlib.sha256(schema_raw).hexdigest() != SCHEMA_SHA256:
        raise CorpusError("pinned schema digest mismatch")
    schema = json.loads(schema_raw)
    Draft7Validator.check_schema(schema)
    try:
        data = json.loads(
            raw.decode("utf-8"),
            object_pairs_hook=unique_object,
            parse_constant=finite_float,
            parse_float=finite_float,
        )
        # JSON escape sequences may decode to unpaired surrogates in Python.
        json.dumps(data, ensure_ascii=False, allow_nan=False).encode("utf-8")
    except CorpusError:
        raise
    except (ValueError, UnicodeError, RecursionError):
        raise CorpusError("invalid or unsupported JSON input") from None
    # No remote schema resolution: all references in this pinned schema are local.
    try:
        error = next(Draft7Validator(schema, registry=Registry()).iter_errors(data), None)
    except RecursionError:
        raise CorpusError("corpus exceeds validation nesting limit") from None
    if error is not None:
        raise CorpusError("CSL input schema validation failed")
    ids: set[str] = set()
    for item in data:
        identity = item["id"]
        if not isinstance(identity, str) or not identity.strip() or identity in ids:
            raise CorpusError("CSL identity must be a unique nonblank string")
        ids.add(identity)
    return len(ids)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("corpus", type=Path)
    args = parser.parse_args()
    try:
        with args.corpus.open("rb") as stream:
            count = check(stream.read(MAX_BYTES + 1))
    except OSError:
        print("CSL preflight failed: input unavailable")
        return 1
    except CorpusError as exc:
        print(f"CSL preflight failed: {exc}")
        return 1
    print(f"CSL 1.0.2 input preflight passed: {count} items; schema {SCHEMA_SHA256}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
