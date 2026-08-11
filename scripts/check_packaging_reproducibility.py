#!/usr/bin/env python3
"""Prove deterministic source packaging and archive/manifest parity."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_package(destination: Path) -> None:
    completed = subprocess.run(
        [sys.executable, "scripts/package_source.py", "--output-dir", str(destination)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode:
        details = [f"source packaging failed with exit code {completed.returncode}"]
        if completed.stdout.strip():
            details.append(f"child stdout:\n{completed.stdout.rstrip()}")
        if completed.stderr.strip():
            details.append(f"child stderr:\n{completed.stderr.rstrip()}")
        raise RuntimeError("\n".join(details))


def verify_members(archive_members: set[str], manifest_paths: set[str]) -> list[str]:
    errors: list[str] = []
    expected = {f"searchright/{path}" for path in manifest_paths}
    if archive_members != expected:
        missing = sorted(expected - archive_members)[:10]
        unexpected = sorted(archive_members - expected)[:10]
        errors.append(f"archive membership differs; missing={missing}, unexpected={unexpected}")
    if any("/.git/" in f"/{member}/" for member in archive_members):
        errors.append("archive contains .git content")
    return errors


def main() -> int:
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix="searchright-package-a-") as first_raw, tempfile.TemporaryDirectory(
        prefix="searchright-package-b-"
    ) as second_raw:
        first = Path(first_raw)
        second = Path(second_raw)
        run_package(first)
        run_package(second)
        names = (
            "searchright-source.zip",
            "searchright-source.tar.gz",
            "searchright-source-manifest.json",
            "searchright-source-packaging-receipt.json",
        )
        for name in names:
            if digest(first / name) != digest(second / name):
                errors.append(f"non-reproducible artifact: {name}")

        manifest = json.loads((first / "searchright-source-manifest.json").read_text(encoding="utf-8"))
        manifest_entries = manifest.get("files", [])
        manifest_paths = {
            entry["path"] for entry in manifest_entries if isinstance(entry, dict) and isinstance(entry.get("path"), str)
        }
        if len(manifest_paths) != len(manifest_entries):
            errors.append("source manifest paths are missing or duplicated")

        with zipfile.ZipFile(first / "searchright-source.zip") as archive:
            zip_members = {name for name in archive.namelist() if not name.endswith("/")}
            errors.extend(verify_members(zip_members, manifest_paths))
            for entry in manifest_entries:
                member = f"searchright/{entry['path']}"
                if hashlib.sha256(archive.read(member)).hexdigest() != entry["sha256"]:
                    errors.append(f"ZIP digest differs from manifest: {entry['path']}")
                    break

        with tarfile.open(first / "searchright-source.tar.gz", "r:gz") as archive:
            tar_files = [member for member in archive.getmembers() if member.isfile()]
            tar_members = {member.name for member in tar_files}
            errors.extend(verify_members(tar_members, manifest_paths))
            by_name = {member.name: member for member in tar_files}
            for entry in manifest_entries:
                member_name = f"searchright/{entry['path']}"
                extracted = archive.extractfile(by_name[member_name])
                if extracted is None or hashlib.sha256(extracted.read()).hexdigest() != entry["sha256"]:
                    errors.append(f"tar digest differs from manifest: {entry['path']}")
                    break

    receipt = {
        "schema_version": "org.searchright.packaging-reproducibility-receipt.v1",
        "status": "passed" if not errors else "failed",
        "files_checked": len(manifest_paths) if "manifest_paths" in locals() else 0,
        "artifacts_compared": 4,
        "errors": errors,
        "limitations": [
            "Source archive reproducibility only; release binaries and signed provenance require the release workflow."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
