#!/usr/bin/env python3
"""Enforce an explicit, minimal and evidence-gated Cargo publication surface."""
from __future__ import annotations

import json
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "release/public-packages.json"


def package_dependencies(cargo: dict[str, Any]) -> set[str]:
    result: set[str] = set()
    for section in ("dependencies", "build-dependencies"):
        for name, value in cargo.get(section, {}).items():
            if isinstance(value, dict) and "path" in value:
                result.add(value.get("package", name))
    return result


def main() -> int:
    errors: list[str] = []
    policy = json.loads(MANIFEST.read_text(encoding="utf-8"))
    candidates = {item["name"]: item for item in policy["packages"]}
    if len(candidates) != len(policy["packages"]):
        errors.append("public package candidate names must be unique")
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"]["channel"]
    if policy.get("development_toolchain") != toolchain:
        errors.append("public package policy development_toolchain differs from rust-toolchain.toml")
    shared_msrv = str(policy.get("shared_layer_msrv", ""))

    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    members = workspace["workspace"]["members"]
    packages: dict[str, dict[str, Any]] = {}
    cargo_documents: dict[str, dict[str, Any]] = {}
    for member in members:
        cargo_path = ROOT / member / "Cargo.toml"
        if not cargo_path.is_file():
            errors.append(f"missing {member}/Cargo.toml")
            continue
        cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
        data = cargo["package"]
        name = data["name"]
        if name in packages:
            errors.append(f"duplicate workspace package name {name}")
        packages[name] = data
        cargo_documents[name] = cargo
        publish = data.get("publish", True)
        if publish is not False and name not in candidates:
            errors.append(f"unlisted package {name} is publishable")
        if name in candidates and candidates[name]["publish_ready"] is not True and publish is not False:
            errors.append(f"candidate {name} must remain publish=false until publish_ready")

    missing = set(candidates) - set(packages)
    if missing:
        errors.append(f"public package policy names missing workspace packages: {sorted(missing)}")

    for name, candidate in candidates.items():
        if not candidate.get("required_before_publish"):
            errors.append(f"candidate {name} lacks required_before_publish gates")
        if candidate.get("publication_status") not in {"blocked", "prepared", "ready"}:
            errors.append(f"candidate {name} has invalid publication_status")
        if name in {"evidence-search-contracts", "evidence-search-core"}:
            observed = str(packages.get(name, {}).get("rust-version", ""))
            if observed != shared_msrv:
                errors.append(f"shared candidate {name} must declare rust-version {shared_msrv}, found {observed}")
        if candidate.get("publish_ready") is True:
            if candidate.get("publication_status") != "ready":
                errors.append(f"publish-ready candidate {name} must have publication_status ready")
            receipts = candidate.get("evidence_receipts", [])
            if not receipts:
                errors.append(f"publish-ready candidate {name} lacks evidence receipts")
            for receipt in receipts:
                if not (ROOT / receipt).is_file():
                    errors.append(f"publish-ready candidate {name} references missing receipt {receipt}")
            internal = package_dependencies(cargo_documents.get(name, {})) - set(candidates)
            if internal:
                errors.append(f"publish-ready candidate {name} depends on non-public workspace packages: {sorted(internal)}")

    receipt = {
        "schema_version": "org.searchright.public-package-policy-receipt.v1",
        "status": "failed" if errors else "passed",
        "development_toolchain": toolchain,
        "shared_layer_msrv": shared_msrv,
        "workspace_packages": len(packages),
        "public_candidates": len(candidates),
        "publish_ready": sum(1 for item in candidates.values() if item["publish_ready"]),
        "errors": errors,
        "claim_boundary": "Passing this gate keeps publication default-deny; it does not establish compiler, API, SemVer, licence, registry or release evidence.",
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
