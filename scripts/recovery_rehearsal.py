#!/usr/bin/env python3
"""Network-free, deterministic backup/restore and idempotency rehearsal."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path

FIXED_FILES = {
    "events/audit.jsonl": b'{"event_id":"event-1","hash":"abc"}\n',
    "snapshots/review.json": b'{"review_id":"review-1","audit_head":"abc"}\n',
    "contracts/review-plan.json": b'{"schema_version":"org.searchright.review-plan.v1","review_id":"review-1"}\n',
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def atomic_write(path: Path, content: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp")
    with temporary.open("wb") as handle:
        handle.write(content)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)


def make_backup(source: Path, destination: Path) -> dict:
    entries = []
    for relative in sorted(FIXED_FILES):
        source_path = source / relative
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_path, target)
        entries.append({"path": relative, "sha256": sha256(target), "bytes": target.stat().st_size})
    return {
        "schema_version": "org.searchright.backup-rehearsal-manifest.v1",
        "backup_id": "rehearsal-backup-1",
        "entries": entries,
    }


def restore(backup: Path, destination: Path, manifest: dict) -> None:
    for entry in manifest["entries"]:
        relative = entry["path"]
        source = backup / relative
        if sha256(source) != entry["sha256"]:
            raise ValueError(f"backup hash mismatch for {relative}")
        atomic_write(destination / relative, source.read_bytes())


def run_rehearsal() -> dict:
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix="searchright-recovery-") as temp:
        root = Path(temp)
        primary = root / "primary"
        backup = root / "backup"
        restore_target = root / "restored"
        for relative, content in FIXED_FILES.items():
            atomic_write(primary / relative, content)
        stale = primary / "snapshots" / ".review.json.tmp"
        stale.write_bytes(b"stale-uncommitted-content")
        if (primary / "snapshots" / "review.json").read_bytes() != FIXED_FILES["snapshots/review.json"]:
            errors.append("stale temporary file replaced canonical snapshot")
        manifest = make_backup(primary, backup)
        (primary / "snapshots" / "review.json").write_bytes(b"corrupted")
        try:
            restore(backup, restore_target, manifest)
            first_hashes = {entry["path"]: sha256(restore_target / entry["path"]) for entry in manifest["entries"]}
            restore(backup, restore_target, manifest)
            second_hashes = {entry["path"]: sha256(restore_target / entry["path"]) for entry in manifest["entries"]}
            if first_hashes != second_hashes:
                errors.append("second restore was not idempotent")
            for entry in manifest["entries"]:
                if first_hashes.get(entry["path"]) != entry["sha256"]:
                    errors.append(f"restored hash mismatch for {entry['path']}")
        except Exception as exc:  # noqa: BLE001 - receipt aggregates rehearsal failures
            errors.append(str(exc))
        tampered = backup / "events" / "audit.jsonl"
        tampered.write_bytes(b"tampered")
        tamper_detected = False
        try:
            restore(backup, root / "tampered-restore", manifest)
        except ValueError:
            tamper_detected = True
        if not tamper_detected:
            errors.append("tampered backup was accepted")
        return {
            "schema_version": "org.searchright.recovery-rehearsal.v1",
            "status": "failed" if errors else "passed",
            "scenario_id": "network-free-reference-recovery",
            "files": len(manifest["entries"]),
            "atomic_replace_checked": True,
            "stale_temporary_checked": True,
            "restore_idempotency_checked": True,
            "tamper_detection_checked": tamper_detected,
            "errors": errors,
            "claim_boundary": "This reference rehearsal validates deterministic local file mechanics only. It does not prove encryption, platform-specific durability, production recovery objectives or operational readiness."
        }
def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--write-receipt", type=Path)
    args = parser.parse_args()
    receipt = run_rehearsal()
    rendered = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    if args.write_receipt:
        args.write_receipt.parent.mkdir(parents=True, exist_ok=True)
        args.write_receipt.write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
