#!/usr/bin/env python3
"""Generate or verify the repository-local SHA-256 source manifest.

The manifest excludes Git metadata, generated build directories and derived
verification receipts (including itself), avoiding a self-referential evidence
cycle. It is distinct from the release source-archive JSON manifest: this
compact form is useful for offline integrity checks after extracting a source
package.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "verification" / "receipts" / "source-manifest.sha256"
EXCLUDED_PARTS = {".git", "target", "dist", "__pycache__", ".pytest_cache", ".mypy_cache"}


def source_files() -> list[Path]:
    files: list[Path] = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path == OUTPUT:
            continue
        relative = path.relative_to(ROOT)
        if any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        if relative.parts[:2] == ("verification", "receipts"):
            continue
        files.append(path)
    return files


def canonical_bytes(path: Path) -> bytes:
    """Return Git-compatible LF bytes for text while preserving binary payloads."""
    value = path.read_bytes()
    if b"\0" in value[:8000]:
        return value
    return value.replace(b"\r\n", b"\n")


def digest(path: Path) -> str:
    return hashlib.sha256(canonical_bytes(path)).hexdigest()


def render() -> str:
    return "".join(
        f"{digest(path)}  ./{path.relative_to(ROOT).as_posix()}\n" for path in source_files()
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = render()
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected:
            print(f"stale {OUTPUT.relative_to(ROOT)}", file=sys.stderr)
            return 1
        print(f"verified {len(source_files())} source files")
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
    print(OUTPUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
