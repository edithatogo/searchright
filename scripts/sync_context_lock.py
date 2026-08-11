#!/usr/bin/env python3
"""Generate or check content hashes for the canonical context spine."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "context/manifest.json"
LOCK = ROOT / "context/context-lock.json"


def canonical_bytes(p: Path) -> bytes:
    data = p.read_bytes()
    relative = p.relative_to(ROOT).as_posix()
    attribute = (
        subprocess.run(
            ["git", "check-attr", "text", "--", relative],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        .stdout.rstrip()
        .rsplit(": ", 1)[-1]
    )
    is_text = attribute == "set" or (attribute == "auto" and b"\0" not in data[:8000])
    return data.replace(b"\r\n", b"\n") if is_text else data


def digest(p: Path) -> str:
    return hashlib.sha256(canonical_bytes(p)).hexdigest()


def build() -> dict:
    m = json.loads(MANIFEST.read_text(encoding="utf-8"))
    paths = [ROOT / x["path"] for x in m["required_context"]]
    ordered = sorted(
        paths, key=lambda p: p.relative_to(ROOT).as_posix().encode("utf-8")
    )
    return {
        "schema_version": "org.searchright.context-lock.v1",
        "generated_at": "source-epoch:2026-08-09",
        "files": [
            {
                "path": p.relative_to(ROOT).as_posix(),
                "sha256": digest(p),
                "size": len(canonical_bytes(p)),
            }
            for p in ordered
        ],
        "claim_boundary": "Hash parity detects context drift; it does not establish semantic correctness or promote evidence.",
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    expected = json.dumps(build(), indent=2, sort_keys=True) + "\n"
    if a.check:
        ok = LOCK.is_file() and LOCK.read_text(encoding="utf-8") == expected
        print(
            json.dumps(
                {
                    "schema_version": "org.searchright.context-lock-receipt.v1",
                    "status": "passed" if ok else "failed",
                    "files": len(build()["files"]),
                },
                indent=2,
            )
        )
        return 0 if ok else 1
    LOCK.write_text(expected, encoding="utf-8", newline="\n")
    print(expected, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
