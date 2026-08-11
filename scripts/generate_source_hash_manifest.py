#!/usr/bin/env python3
"""Generate or verify the repository-local SHA-256 source manifest.

The manifest covers tracked source files while excluding derived verification
receipts (including itself), avoiding local-cache drift and a self-referential
evidence cycle. It is distinct from the release source-archive JSON manifest.
In an extracted package without Git metadata, check mode verifies every listed
file but cannot detect additional unlisted files.
"""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "verification" / "receipts" / "source-manifest.sha256"
EXCLUDED_PARTS = {
    ".git",
    "target",
    "dist",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
}


def tracked_source_files() -> list[Path]:
    """Return tracked source files so ignored local state cannot affect evidence."""
    files: list[Path] = []
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    for raw_relative in sorted(part for part in result.stdout.split(b"\0") if part):
        relative = Path(raw_relative.decode("utf-8"))
        path = ROOT / relative
        if not path.is_file() or path == OUTPUT:
            continue
        if any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        if relative.parts[:2] == ("verification", "receipts"):
            continue
        files.append(path)
    return files


def git_text_attributes(paths: list[Path]) -> dict[Path, str]:
    """Return each path's effective Git text attribute."""
    relative = [path.relative_to(ROOT).as_posix().encode("utf-8") for path in paths]
    result = subprocess.run(
        ["git", "check-attr", "-z", "--stdin", "text"],
        cwd=ROOT,
        check=True,
        input=b"\0".join(relative) + b"\0",
        stdout=subprocess.PIPE,
    )
    fields = result.stdout.split(b"\0")
    attributes: dict[Path, str] = {}
    for index in range(0, len(fields) - 1, 3):
        attributes[ROOT / fields[index].decode("utf-8")] = fields[index + 2].decode(
            "utf-8"
        )
    return attributes


def extracted_text_attributes(paths: list[Path]) -> dict[Path, str]:
    """Interpret the repository's simple text rules without requiring Git."""
    rules: list[tuple[str, str]] = []
    attributes_file = ROOT / ".gitattributes"
    if not attributes_file.is_file():
        return {}
    for line in attributes_file.read_text(encoding="utf-8").splitlines():
        fields = line.strip().split()
        if not fields or fields[0].startswith("#"):
            continue
        value: str | None = None
        for token in fields[1:]:
            if token == "binary" or token == "-text":
                value = "unset"
            elif token == "!text":
                value = "unspecified"
            elif token == "text":
                value = "set"
            elif token.startswith("text="):
                value = token.split("=", 1)[1]
        if value is not None:
            rules.append((fields[0], value))
    result: dict[Path, str] = {}
    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        for pattern, value in rules:
            if fnmatch.fnmatchcase(relative, pattern):
                result[path] = value
    return result


def canonical_bytes(path: Path, attributes: dict[Path, str]) -> bytes:
    """Return Git-compatible LF bytes for attributed text; preserve other bytes."""
    value = path.read_bytes()
    attribute = attributes.get(path, "unspecified")
    is_text = attribute == "set" or (attribute == "auto" and b"\0" not in value[:8000])
    if is_text:
        return value.replace(b"\r\n", b"\n")
    return value


def digest(path: Path, attributes: dict[Path, str]) -> str:
    return hashlib.sha256(canonical_bytes(path, attributes)).hexdigest()


def render() -> str:
    files = tracked_source_files()
    attributes = git_text_attributes(files)
    return "".join(
        f"{digest(path, attributes)}  ./{path.relative_to(ROOT).as_posix()}\n"
        for path in files
    )


def verify_extracted_manifest() -> tuple[bool, int]:
    """Verify listed package files when Git metadata is intentionally absent."""
    if not OUTPUT.is_file():
        return False, 0
    entries = [
        line.split("  ./", 1)
        for line in OUTPUT.read_text(encoding="utf-8").splitlines()
    ]
    if not entries or any(len(entry) != 2 for entry in entries):
        return False, len(entries)
    relative_values = [relative for _, relative in entries]
    if any(
        not relative
        or "\\" in relative
        or relative.startswith("/")
        or ":" in relative.split("/", 1)[0]
        for relative in relative_values
    ):
        return False, len(entries)
    relative_paths = [Path(relative) for relative in relative_values]
    root = ROOT.resolve()
    if any(
        ".." in path.parts or not (root / path).resolve().is_relative_to(root)
        for path in relative_paths
    ):
        return False, len(entries)
    absolute_paths = [ROOT / relative for relative in relative_paths]
    attributes = extracted_text_attributes(absolute_paths)
    return all(
        path.is_file() and digest(path, attributes) == expected
        for (expected, _), path in zip(entries, absolute_paths, strict=True)
    ), len(entries)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check and not (ROOT / ".git").exists():
        verified, count = verify_extracted_manifest()
        message = (
            f"verified {count} listed source files (additional files not checked)"
            if verified
            else f"stale {OUTPUT.relative_to(ROOT)}"
        )
        print(message)
        return 0 if verified else 1
    expected = render()
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected:
            print(f"stale {OUTPUT.relative_to(ROOT)}", file=sys.stderr)
            return 1
        print(f"verified {len(tracked_source_files())} source files")
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
    print(OUTPUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
