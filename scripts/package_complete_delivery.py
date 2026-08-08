#!/usr/bin/env python3
"""Build and verify the complete Searchright delivery, including Git history.

The complete ZIP contains the clean tracked worktree and the entire local
``.git`` directory. A source-only ZIP/tarball, complete Git bundle, checksums
and a machine-readable verification receipt are produced alongside it.
Artifacts must be written outside the repository so packaging cannot dirty the
source it is attesting.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import shutil
import stat
import subprocess
import tempfile
import zipfile
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
GIT_EXCLUDES = {
    ".git/index.lock",
    ".git/shallow.lock",
    ".git/config.lock",
    ".git/packed-refs.lock",
}


def command(args: list[str], *, cwd: Path = ROOT) -> str:
    process = subprocess.run(args, cwd=cwd, text=True, capture_output=True, check=False)
    if process.returncode != 0:
        raise RuntimeError(
            f"command failed ({process.returncode}): {' '.join(args)}\n"
            f"stdout:\n{process.stdout}\nstderr:\n{process.stderr}"
        )
    return process.stdout


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def source_epoch() -> int:
    configured = os.environ.get("SOURCE_DATE_EPOCH")
    if configured:
        return int(configured)
    return int(command(["git", "log", "-1", "--format=%ct"]).strip())


def tracked_files() -> list[Path]:
    payload = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT)
    return sorted(
        ROOT / Path(raw.decode("utf-8"))
        for raw in payload.split(b"\0")
        if raw and (ROOT / Path(raw.decode("utf-8"))).is_file()
    )


def git_files() -> list[Path]:
    result: list[Path] = []
    for path in sorted((ROOT / ".git").rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT).as_posix()
        if relative in GIT_EXCLUDES or relative.endswith(".lock"):
            continue
        result.append(path)
    return result


def zip_datetime(epoch: int) -> tuple[int, int, int, int, int, int]:
    value = max(epoch, 315532800)
    return dt.datetime.fromtimestamp(value, tz=dt.timezone.utc).timetuple()[:6]


def write_complete_zip(path: Path, prefix: str, epoch: int) -> dict[str, int]:
    worktree = tracked_files()
    git = git_files()
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for member in [*worktree, *git]:
            relative = member.relative_to(ROOT).as_posix()
            info = zipfile.ZipInfo(f"{prefix}/{relative}", date_time=zip_datetime(epoch))
            mode = stat.S_IMODE(member.stat().st_mode)
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | mode) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, member.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return {"tracked_files": len(worktree), "git_files": len(git)}


def extract_preserving_modes(archive_path: Path, destination: Path) -> Path:
    with zipfile.ZipFile(archive_path) as archive:
        roots: set[str] = set()
        for member in archive.infolist():
            target = destination / member.filename
            roots.add(Path(member.filename).parts[0])
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(archive.read(member))
            mode = (member.external_attr >> 16) & 0o777
            if mode:
                target.chmod(mode)
    if len(roots) != 1:
        raise RuntimeError(f"complete ZIP must contain one root, observed {sorted(roots)}")
    return destination / next(iter(roots))


def verify_repository(path: Path, *, expected_head: str, run_harness: bool) -> dict[str, Any]:
    head = command(["git", "rev-parse", "HEAD"], cwd=path).strip()
    tree = command(["git", "rev-parse", "HEAD^{tree}"], cwd=path).strip()
    status = command(["git", "status", "--porcelain"], cwd=path).strip()
    fsck = command(["git", "fsck", "--full"], cwd=path).strip()
    if head != expected_head:
        raise RuntimeError(f"archive HEAD {head} differs from source {expected_head}")
    if status:
        raise RuntimeError(f"archive worktree is not clean:\n{status}")
    harness_status = "not_requested"
    if run_harness:
        command(["python", "scripts/run_static_harness.py"], cwd=path)
        harness_status = "passed"
    return {
        "head": head,
        "tree": tree,
        "working_tree": "clean",
        "git_fsck": "passed" if not fsck else "passed_with_messages",
        "static_harness": harness_status,
    }


def artifact(path: Path) -> dict[str, Any]:
    return {"path": path.name, "size": path.stat().st_size, "sha256": digest(path)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--prefix", default="searchright")
    parser.add_argument("--skip-harness", action="store_true")
    args = parser.parse_args()

    output_dir = args.output_dir.resolve()
    if output_dir == ROOT or ROOT in output_dir.parents:
        raise SystemExit("delivery output directory must be outside the repository")
    if command(["git", "status", "--porcelain"]).strip():
        raise SystemExit("complete delivery packaging requires a clean Git working tree")
    remotes = command(["git", "remote", "-v"]).strip()
    if remotes:
        raise SystemExit("remove delivery-local Git remotes before packaging the portable repository")
    output_dir.mkdir(parents=True, exist_ok=True)
    for path in output_dir.iterdir():
        if path.is_file() or path.is_symlink():
            path.unlink()
        elif path.is_dir():
            shutil.rmtree(path)

    epoch = source_epoch()
    head = command(["git", "rev-parse", "HEAD"]).strip()
    tree = command(["git", "rev-parse", "HEAD^{tree}"]).strip()
    commit_count = int(command(["git", "rev-list", "--count", "HEAD"]).strip())
    complete_zip = output_dir / f"{args.prefix}-complete-git.zip"
    bundle = output_dir / f"{args.prefix}.bundle"
    counts = write_complete_zip(complete_zip, args.prefix, epoch)
    command(["git", "bundle", "create", str(bundle), "--all"])
    command(["git", "bundle", "verify", str(bundle)])

    command([
        "python", "scripts/package_source.py",
        "--output-dir", str(output_dir), "--prefix", args.prefix,
    ])
    source_zip = output_dir / f"{args.prefix}-source.zip"
    source_tar = output_dir / f"{args.prefix}-source.tar.gz"
    source_manifest = output_dir / f"{args.prefix}-source-manifest.json"
    source_receipt = output_dir / f"{args.prefix}-source-packaging-receipt.json"

    with tempfile.TemporaryDirectory(prefix="searchright-delivery-") as temporary:
        temporary_path = Path(temporary)
        complete_root = extract_preserving_modes(complete_zip, temporary_path / "complete")
        complete_verification = verify_repository(
            complete_root,
            expected_head=head,
            run_harness=not args.skip_harness,
        )
        bundle_clone = temporary_path / "bundle-clone"
        command(["git", "clone", str(bundle), str(bundle_clone)], cwd=temporary_path)
        bundle_verification = verify_repository(
            bundle_clone,
            expected_head=head,
            run_harness=not args.skip_harness,
        )
        source_root = extract_preserving_modes(source_zip, temporary_path / "source")
        source_harness = "not_requested"
        if not args.skip_harness:
            command(["python", "scripts/run_static_harness.py"], cwd=source_root)
            source_harness = "passed"

    artifacts = [
        complete_zip,
        bundle,
        source_zip,
        source_tar,
        source_manifest,
        source_receipt,
    ]
    checksums_path = output_dir / f"{args.prefix}-artifacts.sha256"
    checksums_path.write_text(
        "".join(f"{digest(path)}  {path.name}\n" for path in artifacts),
        encoding="utf-8",
    )
    receipt = {
        "schema_version": "org.searchright.complete-delivery-receipt.v1",
        "source_date_epoch": epoch,
        "head": head,
        "tree": tree,
        "commit_count": commit_count,
        "working_tree": "clean",
        "configured_remotes": 0,
        "complete_zip_counts": counts,
        "complete_zip_verification": complete_verification,
        "bundle_verification": bundle_verification,
        "source_zip_static_harness": source_harness,
        "artifacts": [artifact(path) for path in [*artifacts, checksums_path]],
        "claim_boundary": "Packaging and static verification do not establish compiler, live-provider, external-validation or publication evidence.",
    }
    receipt_path = output_dir / f"{args.prefix}-delivery-receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
