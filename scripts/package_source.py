#!/usr/bin/env python3
"""Create reproducible Searchright source archives and a content manifest."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import os
import subprocess
import tarfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXCLUDED_PARTS = {".git", "target", "__pycache__", ".pytest_cache", ".mypy_cache"}
EXCLUDED_PREFIXES = {"dist"}


def epoch() -> int:
    configured = os.environ.get("SOURCE_DATE_EPOCH")
    if configured:
        return int(configured)
    try:
        value = subprocess.check_output(["git", "log", "-1", "--format=%ct"], cwd=ROOT, text=True).strip()
        return int(value)
    except Exception:  # noqa: BLE001 - reproducible fallback
        return 0


def files() -> list[Path]:
    """Return the exact tracked source set, never incidental working files."""
    payload = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT)
    result: list[Path] = []
    for raw in payload.split(b"\0"):
        if not raw:
            continue
        relative = Path(raw.decode("utf-8"))
        path = ROOT / relative
        if not path.is_file() or any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        if relative.parts and relative.parts[0] in EXCLUDED_PREFIXES:
            continue
        result.append(path)
    return sorted(result)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, default=ROOT / "dist" / "source")
    parser.add_argument("--prefix", default="searchright")
    args = parser.parse_args()
    output_dir = args.output_dir if args.output_dir.is_absolute() else ROOT / args.output_dir
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True)
    if status.strip():
        raise SystemExit("source packaging requires a clean Git working tree")
    output_dir.mkdir(parents=True, exist_ok=True)
    source_epoch = epoch()
    members = files()
    manifest = {
        "schema_version": "org.searchright.source-archive-manifest.v1",
        "source_date_epoch": source_epoch,
        "files": [
            {
                "path": path.relative_to(ROOT).as_posix(),
                "size": path.stat().st_size,
                "sha256": sha256(path),
                "executable": bool(path.stat().st_mode & 0o111),
            }
            for path in members
        ],
    }
    manifest_bytes = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()
    zip_path = output_dir / f"{args.prefix}-source.zip"
    tar_path = output_dir / f"{args.prefix}-source.tar.gz"
    manifest_path = output_dir / f"{args.prefix}-source-manifest.json"
    manifest_path.write_bytes(manifest_bytes)

    zip_time = max(source_epoch, 315532800)  # ZIP cannot encode dates before 1980.
    import datetime
    date_time = datetime.datetime.fromtimestamp(zip_time, tz=datetime.timezone.utc).timetuple()[:6]
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in members:
            relative = f"{args.prefix}/{path.relative_to(ROOT).as_posix()}"
            info = zipfile.ZipInfo(relative, date_time=date_time)
            mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
            info.external_attr = mode << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)

    tar_buffer = io.BytesIO()
    with tarfile.open(fileobj=tar_buffer, mode="w", format=tarfile.PAX_FORMAT) as archive:
        for path in members:
            relative = f"{args.prefix}/{path.relative_to(ROOT).as_posix()}"
            info = archive.gettarinfo(str(path), arcname=relative)
            info.uid = 0
            info.gid = 0
            info.uname = ""
            info.gname = ""
            info.mtime = source_epoch
            info.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
            info.pax_headers = {}
            with path.open("rb") as stream:
                archive.addfile(info, stream)
    with tar_path.open("wb") as raw_stream:
        with gzip.GzipFile(
            filename="",
            mode="wb",
            compresslevel=9,
            fileobj=raw_stream,
            mtime=source_epoch,
        ) as compressed:
            compressed.write(tar_buffer.getvalue())

    receipt = {
        "schema_version": "org.searchright.source-packaging-receipt.v1",
        "source_date_epoch": source_epoch,
        "file_count": len(members),
        "artifacts": [
            {"path": zip_path.name, "sha256": sha256(zip_path), "size": zip_path.stat().st_size},
            {"path": tar_path.name, "sha256": sha256(tar_path), "size": tar_path.stat().st_size},
            {"path": manifest_path.name, "sha256": sha256(manifest_path), "size": manifest_path.stat().st_size},
        ],
    }
    receipt_path = output_dir / f"{args.prefix}-source-packaging-receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
