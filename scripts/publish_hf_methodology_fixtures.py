#!/usr/bin/env python3
"""Publish the exact methodology-fixture package to Hugging Face, fail closed."""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
STATUS_PATH = ROOT / "registry/huggingface/searchright-methodology-fixtures/status.json"
BUILDER_SPEC = importlib.util.spec_from_file_location(
    "build_hf_methodology_fixtures",
    ROOT / "scripts/build_hf_methodology_fixtures.py",
)
assert BUILDER_SPEC is not None and BUILDER_SPEC.loader is not None
BUILDER = importlib.util.module_from_spec(BUILDER_SPEC)
BUILDER_SPEC.loader.exec_module(BUILDER)


def clean_worktree() -> bool:
    result = subprocess.run(
        ["git", "status", "--porcelain=v1", "--untracked-files=normal"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode == 0 and not result.stdout.strip()


def write_receipt(path: Path, receipt: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def package_paths(package_dir: Path) -> list[str]:
    return sorted(
        path.relative_to(package_dir).as_posix()
        for path in package_dir.rglob("*")
        if path.is_file()
    )


def publish(
    *,
    package_dir: Path,
    repo_id: str,
    visibility: str,
    receipt_path: Path,
    apply: bool,
) -> dict[str, Any]:
    status = json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    expected_repo = status["dataset_id"]
    if repo_id != expected_repo:
        raise ValueError(f"repo id must exactly match {expected_repo}")
    if visibility not in {"public", "private"}:
        raise ValueError("visibility must be public or private")

    manifest = BUILDER.verify_output(package_dir)
    files = package_paths(package_dir)
    expected_files = sorted(
        [entry["path"] for entry in manifest["files"]] + ["dataset-manifest.json"]
    )
    if files != expected_files:
        raise ValueError("package file set differs from the verified manifest")

    base_receipt: dict[str, Any] = {
        "schema_version": "org.searchright.huggingface-publication-receipt.v1",
        "dataset_id": repo_id,
        "repo_type": "dataset",
        "visibility": visibility,
        "dataset_root_sha256": manifest["dataset_root_sha256"],
        "files": len(files),
        "apply_requested": apply,
        "remote_revision": None,
        "parent_revision": None,
        "claim_boundary": (
            "A dry run establishes package readiness only. An applied receipt records a Hub commit "
            "but does not establish external validation, discoverability, methodological performance "
            "or registry acceptance."
        ),
    }

    if not apply:
        base_receipt["status"] = "dry_run_passed"
        write_receipt(receipt_path, base_receipt)
        return base_receipt

    if os.environ.get("SEARCHRIGHT_HF_APPLY") != "1":
        raise PermissionError("SEARCHRIGHT_HF_APPLY=1 is required for publication")
    token = os.environ.get("HF_TOKEN")
    if not token:
        raise PermissionError("HF_TOKEN is required for publication")
    if not clean_worktree():
        raise RuntimeError("publication requires a clean Git worktree")

    from huggingface_hub import CommitOperationAdd, HfApi
    from huggingface_hub.errors import RepositoryNotFoundError

    api = HfApi(token=token)
    parent: str | None = None
    desired_private = visibility == "private"
    try:
        existing = api.dataset_info(repo_id=repo_id)
    except RepositoryNotFoundError:
        api.create_repo(
            repo_id=repo_id,
            repo_type="dataset",
            private=desired_private,
            exist_ok=False,
        )
    else:
        parent = existing.sha
        observed_private = getattr(existing, "private", None)
        if observed_private is not None and bool(observed_private) != desired_private:
            raise RuntimeError(
                "remote visibility differs from the requested visibility; an explicit owner change is required"
            )
        remote_files = set(api.list_repo_files(repo_id=repo_id, repo_type="dataset"))
        permitted = set(files) | {".gitattributes"}
        extras = sorted(remote_files - permitted)
        if extras:
            raise RuntimeError(
                "remote dataset contains non-package files; explicit owner cleanup is required: "
                + ", ".join(extras)
            )

    operations = [
        CommitOperationAdd(path_in_repo=path, path_or_fileobj=package_dir / path)
        for path in files
    ]
    commit = api.create_commit(
        repo_id=repo_id,
        repo_type="dataset",
        operations=operations,
        commit_message=(
            "Publish Searchright methodology fixtures " + manifest["dataset_root_sha256"][:16]
        ),
        parent_commit=parent,
    )
    observed = api.dataset_info(repo_id=repo_id)
    remote_files = set(api.list_repo_files(repo_id=repo_id, repo_type="dataset"))
    missing = sorted(set(files) - remote_files)
    extras = sorted(remote_files - (set(files) | {".gitattributes"}))
    if missing or extras:
        raise RuntimeError(f"remote file-set mismatch: missing={missing}, extras={extras}")

    base_receipt.update(
        {
            "status": "published_commit_observed",
            "parent_revision": parent,
            "remote_revision": observed.sha,
            "commit_url": str(commit.commit_url),
        }
    )
    write_receipt(receipt_path, base_receipt)
    return base_receipt


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--package-dir", type=Path, required=True)
    parser.add_argument(
        "--repo-id", default="edithatogo/searchright-methodology-fixtures"
    )
    parser.add_argument("--visibility", choices=("public", "private"), default="public")
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    receipt = publish(
        package_dir=args.package_dir,
        repo_id=args.repo_id,
        visibility=args.visibility,
        receipt_path=args.receipt,
        apply=args.apply,
    )
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
