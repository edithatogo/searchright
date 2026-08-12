#!/usr/bin/env python3
"""Create and verify deterministic, self-describing Searchright review bundles.

The implementation is deliberately network-free and standard-library only. A
bundle is a ZIP/`.srpack` containing `manifest.json` and immutable payload
entries under `payload/`. Packing rejects symlinks, path traversal, high-risk
secret material, duplicate destinations, and configured size limits.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import mimetypes
import os
import re
import stat
import tempfile
import zipfile
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA_VERSION = "org.searchright.review-bundle-manifest.v1"
FORMAT_VERSION = "1"
FIXED_ZIP_TIME = (1980, 1, 1, 0, 0, 0)
DEFAULT_MAX_FILES = 10_000
DEFAULT_MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
DEFAULT_MAX_FILE_BYTES = 512 * 1024 * 1024
VERIFY_MAX_FILES = 10_000
VERIFY_MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
VERIFY_MAX_FILE_BYTES = 512 * 1024 * 1024
VERIFY_MAX_MANIFEST_BYTES = 4 * 1024 * 1024
HIGH_RISK_BASENAMES = {
    ".env",
    ".env.local",
    "id_rsa",
    "id_ed25519",
    "credentials",
    "credentials.json",
    "service-account.json",
}
SECRET_PATTERNS = [
    re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    re.compile(rb"\bgh[pousr]_[A-Za-z0-9_]{20,}\b"),
    re.compile(rb"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(rb"(?i)\b(?:password|passwd|api[_-]?key|client[_-]?secret)\s*[:=]\s*['\"]?[^\s'\"]{8,}"),
]


class BundleError(ValueError):
    """A review bundle violates its deterministic or safety contract."""


@dataclass(frozen=True)
class PlannedEntry:
    source: Path
    destination: str
    role: str
    media_type: str


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def safe_destination(value: str) -> str:
    normalised = value.replace("\\", "/").strip("/")
    path = PurePosixPath(normalised)
    if not normalised or path.is_absolute() or ".." in path.parts or "." in path.parts:
        raise BundleError(f"unsafe bundle destination: {value!r}")
    if any(part in {"", ".git"} for part in path.parts):
        raise BundleError(f"prohibited bundle destination: {value!r}")
    return path.as_posix()


def ensure_regular_file(path: Path) -> None:
    try:
        mode = path.lstat().st_mode
    except FileNotFoundError as exc:
        raise BundleError(f"source file does not exist: {path}") from exc
    if stat.S_ISLNK(mode):
        raise BundleError(f"symlinks are prohibited: {path}")
    if not stat.S_ISREG(mode):
        raise BundleError(f"source must be a regular file: {path}")


def resolve_plan_source(base: Path, value: str) -> Path:
    """Resolve one plan source without following symlinks outside the plan root."""
    relative = PurePosixPath(value.replace("\\", "/"))
    if relative.is_absolute() or not relative.parts or any(part in {"", ".", ".."} for part in relative.parts):
        raise BundleError(f"unsafe plan source: {value!r}")
    candidate = base.joinpath(*relative.parts)
    current = base
    for part in relative.parts:
        current = current / part
        try:
            mode = current.lstat().st_mode
        except FileNotFoundError as exc:
            raise BundleError(f"source file does not exist: {current}") from exc
        if stat.S_ISLNK(mode):
            raise BundleError(f"symlinks are prohibited: {current}")
    ensure_regular_file(candidate)
    resolved = candidate.resolve(strict=True)
    try:
        resolved.relative_to(base)
    except ValueError as exc:
        raise BundleError(f"source escapes the plan directory: {value!r}") from exc
    return resolved


def scan_for_secrets(path: Path, data: bytes) -> None:
    if path.name.lower() in HIGH_RISK_BASENAMES:
        raise BundleError(f"high-risk secret filename is prohibited: {path.name}")
    for pattern in SECRET_PATTERNS:
        if pattern.search(data):
            raise BundleError(f"potential secret material detected in {path}")


def media_type_for(path: Path, explicit: str | None) -> str:
    if explicit:
        return explicit
    guessed, _ = mimetypes.guess_type(path.name)
    return guessed or "application/octet-stream"


def parse_plan(plan_path: Path) -> tuple[dict[str, Any], list[PlannedEntry]]:
    try:
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise BundleError(f"could not parse bundle plan {plan_path}: {exc}") from exc
    if plan.get("schema_version") != "org.searchright.review-bundle-plan.v1":
        raise BundleError("unexpected review bundle plan schema_version")
    for field in ("bundle_id", "review_id", "source_epoch", "entries"):
        if field not in plan:
            raise BundleError(f"bundle plan is missing {field}")
    if not isinstance(plan["entries"], list) or not plan["entries"]:
        raise BundleError("bundle plan entries must be a non-empty array")
    base = plan_path.resolve().parent
    entries: list[PlannedEntry] = []
    destinations: set[str] = set()
    for index, raw in enumerate(plan["entries"], start=1):
        if not isinstance(raw, dict):
            raise BundleError(f"plan entry {index} must be an object")
        source_value = raw.get("source")
        role = raw.get("role")
        if not isinstance(source_value, str) or not source_value.strip():
            raise BundleError(f"plan entry {index} has no source")
        if not isinstance(role, str) or not role.strip():
            raise BundleError(f"plan entry {index} has no role")
        source = resolve_plan_source(base, source_value)
        destination = safe_destination(raw.get("path") or source.name)
        if destination in destinations:
            raise BundleError(f"duplicate destination: {destination}")
        destinations.add(destination)
        entries.append(
            PlannedEntry(
                source=source,
                destination=destination,
                role=role.strip(),
                media_type=media_type_for(source, raw.get("media_type")),
            )
        )
    return plan, sorted(entries, key=lambda item: item.destination)


def merkle_root(entry_digests: Iterable[str]) -> str:
    nodes = [bytes.fromhex(value) for value in entry_digests]
    if not nodes:
        return hashlib.sha256(b"").hexdigest()
    while len(nodes) > 1:
        if len(nodes) % 2:
            nodes.append(nodes[-1])
        nodes = [hashlib.sha256(nodes[index] + nodes[index + 1]).digest() for index in range(0, len(nodes), 2)]
    return nodes[0].hex()


def zip_info(name: str, executable: bool = False) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(name, FIXED_ZIP_TIME)
    info.compress_type = zipfile.ZIP_DEFLATED
    info.create_system = 3
    mode = 0o755 if executable else 0o644
    info.external_attr = (stat.S_IFREG | mode) << 16
    info.flag_bits |= 0x800
    return info


def build_manifest(plan: dict[str, Any], entries: list[PlannedEntry], *, max_files: int, max_total_bytes: int, max_file_bytes: int) -> tuple[dict[str, Any], dict[str, bytes]]:
    if len(entries) > max_files:
        raise BundleError(f"bundle exceeds maximum file count ({len(entries)} > {max_files})")
    payloads: dict[str, bytes] = {}
    manifest_entries: list[dict[str, Any]] = []
    total = 0
    for entry in entries:
        data = entry.source.read_bytes()
        if len(data) > max_file_bytes:
            raise BundleError(f"entry exceeds maximum file size: {entry.source}")
        total += len(data)
        if total > max_total_bytes:
            raise BundleError(f"bundle exceeds maximum total payload size ({total} > {max_total_bytes})")
        scan_for_secrets(entry.source, data)
        digest = sha256_bytes(data)
        archive_path = f"payload/{entry.destination}"
        payloads[archive_path] = data
        manifest_entries.append(
            {
                "path": entry.destination,
                "archive_path": archive_path,
                "role": entry.role,
                "media_type": entry.media_type,
                "size": len(data),
                "sha256": digest,
            }
        )
    descriptor_digests = [sha256_bytes(canonical_json_bytes(item)) for item in manifest_entries]
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "format_version": FORMAT_VERSION,
        "bundle_id": plan["bundle_id"],
        "review_id": plan["review_id"],
        "source_epoch": plan["source_epoch"],
        "entries": manifest_entries,
        "payload_bytes": total,
        "entry_count": len(manifest_entries),
        "descriptor_merkle_root": merkle_root(descriptor_digests),
        "policy": {
            "deterministic": True,
            "network_required": False,
            "symlinks_allowed": False,
            "external_writes_allowed": False,
            "secret_scan_required": True,
            "max_files": max_files,
            "max_total_bytes": max_total_bytes,
            "max_file_bytes": max_file_bytes,
        },
        "claim_boundary": "This bundle proves byte integrity and declared provenance only; it does not prove methodological adequacy, bibliographic truth, screening correctness, or registry acceptance.",
    }
    return manifest, payloads


def pack(plan_path: Path, output: Path, *, max_files: int, max_total_bytes: int, max_file_bytes: int) -> dict[str, Any]:
    plan, entries = parse_plan(plan_path)
    manifest, payloads = build_manifest(
        plan,
        entries,
        max_files=max_files,
        max_total_bytes=max_total_bytes,
        max_file_bytes=max_file_bytes,
    )
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(output.suffix + ".tmp")
    if temporary.exists():
        temporary.unlink()
    with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9, strict_timestamps=True) as archive:
        archive.writestr(zip_info("manifest.json"), canonical_json_bytes(manifest), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
        for archive_path in sorted(payloads):
            archive.writestr(zip_info(archive_path), payloads[archive_path], compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    os.replace(temporary, output)
    receipt = verify(output)
    receipt["bundle_sha256"] = sha256_bytes(output.read_bytes())
    return receipt


def load_manifest(archive: zipfile.ZipFile) -> dict[str, Any]:
    try:
        raw = archive.read("manifest.json")
        manifest = json.loads(raw)
    except (KeyError, json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise BundleError(f"bundle manifest is missing or malformed: {exc}") from exc
    if manifest.get("schema_version") != SCHEMA_VERSION or manifest.get("format_version") != FORMAT_VERSION:
        raise BundleError("unsupported review bundle manifest version")
    return manifest


def validate_archive_member(name: str) -> None:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "" in path.parts:
        raise BundleError(f"unsafe archive member: {name}")
    if name != "manifest.json" and not name.startswith("payload/"):
        raise BundleError(f"undeclared archive namespace: {name}")


def verify(bundle_path: Path) -> dict[str, Any]:
    errors: list[str] = []
    checked = 0
    try:
        with zipfile.ZipFile(bundle_path, "r") as archive:
            infos = archive.infolist()
            if len(infos) > VERIFY_MAX_FILES + 1:
                raise BundleError("archive member count exceeds the local verification limit")
            names = [info.filename for info in infos]
            if len(names) != len(set(names)):
                raise BundleError("duplicate archive member names")
            declared_total = 0
            for info in infos:
                name = info.filename
                validate_archive_member(name)
                mode = info.external_attr >> 16
                if stat.S_ISLNK(mode):
                    raise BundleError(f"symlink archive member is prohibited: {name}")
                if info.is_dir() or not stat.S_ISREG(mode):
                    raise BundleError(f"non-regular archive member is prohibited: {name}")
                if info.flag_bits & 0x1:
                    raise BundleError(f"encrypted archive member is prohibited: {name}")
                if info.compress_type not in {zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED}:
                    raise BundleError(f"unsupported compression method for {name}")
                member_limit = VERIFY_MAX_MANIFEST_BYTES if name == "manifest.json" else VERIFY_MAX_FILE_BYTES
                if info.file_size > member_limit:
                    raise BundleError(f"archive member exceeds the local verification limit: {name}")
                declared_total += info.file_size
                if declared_total > VERIFY_MAX_TOTAL_BYTES + VERIFY_MAX_MANIFEST_BYTES:
                    raise BundleError("archive payload exceeds the local verification limit")
            manifest = load_manifest(archive)
            expected = {"manifest.json"}
            descriptor_digests: list[str] = []
            total = 0
            entries = manifest.get("entries")
            if not isinstance(entries, list) or not entries:
                raise BundleError("manifest entries must be a non-empty array")
            destinations: set[str] = set()
            policy = manifest.get("policy", {})
            policy_limits = {
                "max_files": VERIFY_MAX_FILES,
                "max_total_bytes": VERIFY_MAX_TOTAL_BYTES,
                "max_file_bytes": VERIFY_MAX_FILE_BYTES,
            }
            for key, local_limit in policy_limits.items():
                value = policy.get(key)
                if not isinstance(value, int) or isinstance(value, bool) or value < 1 or value > local_limit:
                    raise BundleError(f"manifest policy {key} exceeds or omits the local verification limit")
            if len(entries) > policy["max_files"]:
                raise BundleError("manifest entry count exceeds its declared policy")
            for raw in entries:
                if not isinstance(raw, dict):
                    raise BundleError("manifest entry must be an object")
                destination = safe_destination(str(raw.get("path", "")))
                archive_path = str(raw.get("archive_path", ""))
                if archive_path != f"payload/{destination}":
                    raise BundleError(f"archive path mismatch for {destination}")
                if destination in destinations:
                    raise BundleError(f"duplicate manifest destination: {destination}")
                destinations.add(destination)
                expected.add(archive_path)
                info = archive.getinfo(archive_path)
                declared_size = raw.get("size")
                if not isinstance(declared_size, int) or isinstance(declared_size, bool) or declared_size < 0:
                    raise BundleError(f"invalid declared size: {destination}")
                if declared_size > policy["max_file_bytes"] or info.file_size > policy["max_file_bytes"]:
                    raise BundleError(f"entry exceeds its declared policy: {destination}")
                digest = hashlib.sha256()
                observed_size = 0
                with archive.open(info, "r") as source:
                    while chunk := source.read(1024 * 1024):
                        observed_size += len(chunk)
                        if observed_size > min(policy["max_file_bytes"], VERIFY_MAX_FILE_BYTES):
                            raise BundleError(f"expanded entry exceeds its verification limit: {destination}")
                        digest.update(chunk)
                if observed_size != declared_size:
                    errors.append(f"size mismatch: {destination}")
                if digest.hexdigest() != raw.get("sha256"):
                    errors.append(f"sha256 mismatch: {destination}")
                total += observed_size
                if total > min(policy["max_total_bytes"], VERIFY_MAX_TOTAL_BYTES):
                    raise BundleError("expanded payload exceeds its verification limit")
                descriptor_digests.append(sha256_bytes(canonical_json_bytes(raw)))
                checked += 1
            undeclared = sorted(set(names) - expected)
            missing = sorted(expected - set(names))
            if undeclared:
                errors.append(f"undeclared members: {undeclared}")
            if missing:
                errors.append(f"missing members: {missing}")
            if total != manifest.get("payload_bytes"):
                errors.append("payload byte total mismatch")
            if checked != manifest.get("entry_count"):
                errors.append("entry count mismatch")
            if merkle_root(descriptor_digests) != manifest.get("descriptor_merkle_root"):
                errors.append("descriptor Merkle root mismatch")
            for key, expected_value in {
                "deterministic": True,
                "network_required": False,
                "symlinks_allowed": False,
                "external_writes_allowed": False,
                "secret_scan_required": True,
            }.items():
                if policy.get(key) != expected_value:
                    errors.append(f"policy mismatch: {key}")
    except (OSError, zipfile.BadZipFile, BundleError, KeyError) as exc:
        errors.append(str(exc))
        manifest = None
    return {
        "schema_version": "org.searchright.review-bundle-verification-receipt.v1",
        "status": "failed" if errors else "passed",
        "bundle": str(bundle_path),
        "entries_checked": checked,
        "manifest": manifest,
        "errors": errors,
        "claim_boundary": "Verification establishes archive and payload integrity only.",
    }


def inspect_bundle(bundle_path: Path) -> dict[str, Any]:
    with zipfile.ZipFile(bundle_path, "r") as archive:
        return load_manifest(archive)


def self_test() -> dict[str, Any]:
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix="searchright-review-bundle-") as temp_value:
        temp = Path(temp_value)
        (temp / "protocol.json").write_text('{"review_id":"review-self-test"}\n', encoding="utf-8")
        (temp / "receipts.jsonl").write_text('{"receipt_id":"receipt-1"}\n', encoding="utf-8")
        plan = {
            "schema_version": "org.searchright.review-bundle-plan.v1",
            "bundle_id": "bundle-self-test",
            "review_id": "review-self-test",
            "source_epoch": "2026-08-08",
            "entries": [
                {"source": "protocol.json", "path": "protocol/protocol.json", "role": "review_protocol", "media_type": "application/json"},
                {"source": "receipts.jsonl", "path": "execution/receipts.jsonl", "role": "source_receipts", "media_type": "application/x-ndjson"},
            ],
        }
        plan_path = temp / "plan.json"
        plan_path.write_bytes(canonical_json_bytes(plan))
        first = temp / "first.srpack"
        second = temp / "second.srpack"
        pack(plan_path, first, max_files=10, max_total_bytes=1024 * 1024, max_file_bytes=1024 * 1024)
        pack(plan_path, second, max_files=10, max_total_bytes=1024 * 1024, max_file_bytes=1024 * 1024)
        if first.read_bytes() != second.read_bytes():
            errors.append("deterministic pack outputs differ")
        verification = verify(first)
        if verification["status"] != "passed":
            errors.extend(verification["errors"])
        tampered = temp / "tampered.srpack"
        with zipfile.ZipFile(first, "r") as source, zipfile.ZipFile(tampered, "w") as target:
            for info in source.infolist():
                data = source.read(info.filename)
                if info.filename == "payload/protocol/protocol.json":
                    data += b"tampered"
                target.writestr(info, data)
        if verify(tampered)["status"] != "failed":
            errors.append("tampered bundle was not rejected")
        secret = temp / ".env"
        secret.write_text("API_KEY=0123456789abcdef\n", encoding="utf-8")
        bad_plan = dict(plan)
        bad_plan["entries"] = [{"source": ".env", "path": "secret.txt", "role": "invalid"}]
        bad_plan_path = temp / "bad-plan.json"
        bad_plan_path.write_bytes(canonical_json_bytes(bad_plan))
        try:
            pack(bad_plan_path, temp / "bad.srpack", max_files=10, max_total_bytes=1024 * 1024, max_file_bytes=1024 * 1024)
            errors.append("secret-bearing input was not rejected")
        except BundleError:
            pass
        digest = sha256_bytes(first.read_bytes())
    return {
        "schema_version": "org.searchright.review-bundle-self-test-receipt.v1",
        "status": "failed" if errors else "passed",
        "deterministic_digest": digest,
        "tests": ["deterministic_bytes", "payload_verification", "tamper_rejection", "secret_rejection"],
        "errors": errors,
    }


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    commands = result.add_subparsers(dest="command", required=True)
    pack_command = commands.add_parser("pack", help="create a deterministic review bundle")
    pack_command.add_argument("--plan", type=Path, required=True)
    pack_command.add_argument("--output", type=Path, required=True)
    pack_command.add_argument("--max-files", type=int, default=DEFAULT_MAX_FILES)
    pack_command.add_argument("--max-total-bytes", type=int, default=DEFAULT_MAX_TOTAL_BYTES)
    pack_command.add_argument("--max-file-bytes", type=int, default=DEFAULT_MAX_FILE_BYTES)
    verify_command = commands.add_parser("verify", help="verify a review bundle")
    verify_command.add_argument("bundle", type=Path)
    inspect_command = commands.add_parser("inspect", help="print a bundle manifest")
    inspect_command.add_argument("bundle", type=Path)
    commands.add_parser("self-test", help="run deterministic and tamper-resistance tests")
    return result


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "pack":
            output = pack(
                args.plan,
                args.output,
                max_files=args.max_files,
                max_total_bytes=args.max_total_bytes,
                max_file_bytes=args.max_file_bytes,
            )
        elif args.command == "verify":
            output = verify(args.bundle)
        elif args.command == "inspect":
            output = inspect_bundle(args.bundle)
        else:
            output = self_test()
    except BundleError as exc:
        output = {
            "schema_version": "org.searchright.review-bundle-error.v1",
            "status": "failed",
            "errors": [str(exc)],
        }
    print(json.dumps(output, indent=2, sort_keys=True))
    return 0 if output.get("status", "passed") == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
