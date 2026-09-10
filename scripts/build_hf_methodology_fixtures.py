#!/usr/bin/env python3
"""Build and verify the rights-clear Hugging Face methodology-fixture package."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import tempfile
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PACKET = ROOT / "registry/huggingface/searchright-methodology-fixtures"
ALLOWLIST = PACKET / "source-allowlist.json"
CARD = PACKET / "README.md"
CANONICAL_MANIFEST = PACKET / "dataset-manifest.json"
DEFAULT_OUTPUT = ROOT / "target/huggingface/searchright-methodology-fixtures"
MAX_FILE_BYTES = 2 * 1024 * 1024

SECRET_PATTERNS = {
    "private_key": re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    "github_token": re.compile(rb"gh[pousr]_[A-Za-z0-9]{20,}"),
    "huggingface_token": re.compile(rb"hf_[A-Za-z0-9]{20,}"),
    "aws_access_key": re.compile(rb"AKIA[0-9A-Z]{16}"),
    "bearer_token": re.compile(rb"(?i)authorization\s*:\s*bearer\s+\S+"),
    "email": re.compile(rb"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b"),
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")


def safe_relative(value: str) -> PurePosixPath:
    path = PurePosixPath(value)
    if path.is_absolute() or not path.parts or any(part in {"", ".", ".."} for part in path.parts):
        raise ValueError(f"unsafe relative path: {value!r}")
    return path


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def scan_payload(path: str, data: bytes) -> None:
    for name, pattern in SECRET_PATTERNS.items():
        if pattern.search(data):
            raise ValueError(f"{path} contains prohibited {name} material")


def validate_source(path: Path, source_path: str, media_type: str) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"allowlisted source is not a regular file: {source_path}")
    data = path.read_bytes()
    if len(data) > MAX_FILE_BYTES:
        raise ValueError(f"allowlisted source exceeds {MAX_FILE_BYTES} bytes: {source_path}")
    scan_payload(source_path, data)
    if media_type == "application/json":
        try:
            json.loads(data.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ValueError(f"invalid JSON source {source_path}: {error}") from error
    else:
        try:
            data.decode("utf-8")
        except UnicodeDecodeError as error:
            raise ValueError(f"non-UTF-8 text source {source_path}") from error
    return data


def allowlist() -> dict[str, Any]:
    value = load_json(ALLOWLIST)
    if value.get("schema_version") != "org.searchright.huggingface-source-allowlist.v1":
        raise ValueError("unsupported Hugging Face source allowlist schema")
    if value.get("dataset_id") != "edithatogo/searchright-methodology-fixtures":
        raise ValueError("unexpected Hugging Face dataset id")
    files = value.get("files")
    if not isinstance(files, list) or not files:
        raise ValueError("source allowlist must contain files")
    sources: set[str] = set()
    targets: set[str] = set()
    for row in files:
        if not isinstance(row, dict):
            raise ValueError("allowlist file entries must be objects")
        source = str(row.get("source_path", ""))
        target = str(row.get("target_path", ""))
        safe_relative(source)
        safe_relative(target)
        lowered = f"{source}/{target}".lower()
        if "sealed" in lowered or ".git" in PurePosixPath(target).parts:
            raise ValueError(f"sealed or Git material is prohibited: {source} -> {target}")
        if source in sources or target in targets:
            raise ValueError(f"duplicate allowlist source or target: {source} -> {target}")
        sources.add(source)
        targets.add(target)
        if not row.get("media_type") or not row.get("rights_basis"):
            raise ValueError(f"allowlist entry lacks media type or rights basis: {source}")
    return value


def validate_methodology_contract(targets: set[str]) -> None:
    methodology = load_json(ROOT / "benchmarks/methodology/manifest.json")
    leakage = methodology.get("leakage_controls", {})
    if leakage.get("sealed_labels_committed") is not False:
        raise ValueError("methodology manifest does not deny committed sealed labels")
    if leakage.get("sealed_labels_available_to_agents") is not False:
        raise ValueError("methodology manifest does not deny agent access to sealed labels")
    for task in methodology.get("tasks", []):
        fixture = task.get("fixture")
        if "validation" in task.get("partitions", []) and fixture not in targets:
            raise ValueError(f"validation fixture is absent from package: {fixture}")
        if task.get("id") == "screening_prioritisation" and "advisory" not in str(
            task.get("claim_boundary", "")
        ).lower():
            raise ValueError("screening benchmark omits the advisory-authority boundary")

    query_index = load_json(ROOT / "contracts/query-corpus/index.json")
    for fixture in query_index.get("fixtures", []):
        path = fixture.get("path")
        if path not in targets:
            raise ValueError(f"native-query fixture is absent from package: {path}")


def render_manifest() -> dict[str, Any]:
    policy = allowlist()
    entries: list[dict[str, Any]] = []

    card_data = validate_source(CARD, str(CARD.relative_to(ROOT)), "text/markdown")
    entries.append(
        {
            "source_path": str(CARD.relative_to(ROOT)),
            "path": "README.md",
            "sha256": sha256(card_data),
            "bytes": len(card_data),
            "media_type": "text/markdown",
            "rights_basis": "Searchright-authored Hugging Face dataset card.",
        }
    )

    targets: set[str] = {"README.md"}
    for row in policy["files"]:
        source = str(row["source_path"])
        target = str(row["target_path"])
        data = validate_source(ROOT / source, source, str(row["media_type"]))
        entries.append(
            {
                "source_path": source,
                "path": target,
                "sha256": sha256(data),
                "bytes": len(data),
                "media_type": row["media_type"],
                "rights_basis": row["rights_basis"],
            }
        )
        targets.add(target)

    validate_methodology_contract(targets)
    entries.sort(key=lambda row: row["path"])
    root_material = [[row["path"], row["sha256"], row["bytes"]] for row in entries]
    return {
        "schema_version": "org.searchright.huggingface-dataset-manifest.v1",
        "dataset_id": policy["dataset_id"],
        "repo_type": "dataset",
        "source_repository": "edithatogo/searchright",
        "source_epoch": "2026-08-08",
        "licence": policy["licence"],
        "status": "source_package_prepared_not_published",
        "files": entries,
        "dataset_root_sha256": sha256(canonical_json(root_material)),
        "excluded": policy["excluded"],
        "claim_boundary": (
            "This manifest proves only deterministic source packaging for the exact listed files. "
            "It does not prove Hub publication, external validation, methodological performance, "
            "live-provider behaviour or independent acceptance."
        ),
    }


def build(output: Path) -> dict[str, Any]:
    manifest = render_manifest()
    if output.exists():
        if output.is_symlink() or not output.is_dir():
            raise ValueError(f"unsafe output path: {output}")
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=False)

    for entry in manifest["files"]:
        target = output.joinpath(*safe_relative(entry["path"]).parts)
        target.parent.mkdir(parents=True, exist_ok=True)
        source = ROOT / entry["source_path"]
        target.write_bytes(source.read_bytes())
    (output / "dataset-manifest.json").write_bytes(canonical_json(manifest))
    verify_output(output, manifest)
    return manifest


def verify_output(output: Path, expected_manifest: dict[str, Any] | None = None) -> dict[str, Any]:
    if output.is_symlink() or not output.is_dir():
        raise ValueError("dataset output is not a regular directory")
    manifest_path = output / "dataset-manifest.json"
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise ValueError("dataset manifest is missing or symbolic")
    manifest = load_json(manifest_path)
    if expected_manifest is not None and manifest != expected_manifest:
        raise ValueError("built dataset manifest differs from rendered manifest")

    expected_paths = {entry["path"] for entry in manifest["files"]} | {"dataset-manifest.json"}
    actual_paths: set[str] = set()
    for path in sorted(output.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"dataset output contains a symlink: {path}")
        if path.is_file():
            relative = path.relative_to(output).as_posix()
            safe_relative(relative)
            actual_paths.add(relative)
    if actual_paths != expected_paths:
        raise ValueError(
            f"dataset output paths differ: missing={sorted(expected_paths-actual_paths)}, "
            f"extra={sorted(actual_paths-expected_paths)}"
        )

    root_material: list[list[Any]] = []
    for entry in sorted(manifest["files"], key=lambda row: row["path"]):
        data = (output / entry["path"]).read_bytes()
        scan_payload(entry["path"], data)
        if sha256(data) != entry["sha256"] or len(data) != entry["bytes"]:
            raise ValueError(f"dataset file integrity mismatch: {entry['path']}")
        root_material.append([entry["path"], entry["sha256"], entry["bytes"]])
    if sha256(canonical_json(root_material)) != manifest.get("dataset_root_sha256"):
        raise ValueError("dataset root digest mismatch")
    if any("sealed" in path.lower() for path in actual_paths):
        raise ValueError("sealed material entered the dataset output")
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--write-manifest", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    manifest = render_manifest()
    rendered = canonical_json(manifest)
    stale = not CANONICAL_MANIFEST.is_file() or CANONICAL_MANIFEST.read_bytes() != rendered
    if args.write_manifest:
        CANONICAL_MANIFEST.parent.mkdir(parents=True, exist_ok=True)
        CANONICAL_MANIFEST.write_bytes(rendered)
        stale = False
    if args.check and stale:
        raise SystemExit(
            "registry/huggingface/searchright-methodology-fixtures/dataset-manifest.json "
            "is stale; run with --write-manifest"
        )

    built = build(args.output)
    receipt = {
        "schema_version": "org.searchright.huggingface-package-receipt.v1",
        "status": "passed",
        "dataset_id": built["dataset_id"],
        "files": len(built["files"]) + 1,
        "dataset_root_sha256": built["dataset_root_sha256"],
        "canonical_manifest_stale": stale,
        "output": str(args.output),
        "limitations": [
            "No Hugging Face repository creation, upload, revision or external validation is established."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
