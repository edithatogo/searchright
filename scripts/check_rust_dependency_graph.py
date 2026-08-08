#!/usr/bin/env python3
"""Source-level Rust workspace dependency graph checks for toolchain-poor environments."""

from __future__ import annotations

import json
import re
import sys
import tomllib
from collections import defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    errors: list[str] = []
    root = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    members = [ROOT / value for value in root["workspace"]["members"]]
    workspace_version = str(root["workspace"]["package"]["version"])
    expected_internal_version = f"={workspace_version}"
    packages: dict[str, Path] = {}
    manifests: dict[str, dict] = {}
    for member in members:
        manifest = tomllib.loads((member / "Cargo.toml").read_text(encoding="utf-8"))
        name = manifest["package"]["name"]
        packages[name] = member
        manifests[name] = manifest

    graph: dict[str, set[str]] = {name: set() for name in packages}
    source_references: dict[str, set[str]] = defaultdict(set)
    crate_identifiers = {name: name.replace("-", "_") for name in packages}

    for package, manifest in manifests.items():
        declared: set[str] = set()
        for section in ("dependencies", "dev-dependencies", "build-dependencies"):
            for dependency, value in manifest.get(section, {}).items():
                canonical = dependency
                if isinstance(value, dict) and isinstance(value.get("package"), str):
                    canonical = value["package"]
                if canonical in packages:
                    declared.add(canonical)
                    graph[package].add(canonical)
                    if isinstance(value, dict) and "path" in value:
                        resolved = (packages[package] / value["path"]).resolve()
                        if resolved != packages[canonical].resolve():
                            errors.append(f"{package}: path for {canonical} resolves to {resolved}")
                        if value.get("version") != expected_internal_version:
                            errors.append(
                                f"{package}: internal dependency {canonical} must pin "
                                f"version {expected_internal_version}"
                            )
                        consumer_publishable = manifests[package]["package"].get("publish") is not False
                        dependency_publishable = manifests[canonical]["package"].get("publish") is not False
                        if consumer_publishable and not dependency_publishable:
                            errors.append(
                                f"{package}: publishable package depends on non-publishable {canonical}"
                            )
                if isinstance(value, dict) and "git" in value:
                    revision = value.get("rev")
                    if not isinstance(revision, str) or len(revision) != 40 or not all(
                        character in "0123456789abcdef" for character in revision
                    ):
                        errors.append(f"{package}: Git dependency {dependency} requires an exact revision")
                    if manifest["package"].get("publish") is not False:
                        errors.append(
                            f"{package}: package with a Git dependency must remain publish=false "
                            "until a registry-backed dependency is available"
                        )
        text = "\n".join(path.read_text(encoding="utf-8") for path in sorted(packages[package].rglob("*.rs")))
        for other, identifier in crate_identifiers.items():
            if other == package:
                continue
            if re.search(rf"(?m)(?:^|[^A-Za-z0-9_]){re.escape(identifier)}::", text):
                source_references[package].add(other)
                if other not in declared:
                    errors.append(f"{package}: references internal crate {other} without declaring it")

    indegree = {name: 0 for name in graph}
    reverse: dict[str, set[str]] = defaultdict(set)
    for package, dependencies in graph.items():
        for dependency in dependencies:
            indegree[package] += 1
            reverse[dependency].add(package)
    queue = deque(sorted(name for name, degree in indegree.items() if degree == 0))
    ordered: list[str] = []
    while queue:
        name = queue.popleft()
        ordered.append(name)
        for consumer in sorted(reverse[name]):
            indegree[consumer] -= 1
            if indegree[consumer] == 0:
                queue.append(consumer)
    if len(ordered) != len(graph):
        errors.append(f"workspace internal dependency cycle detected: {sorted(name for name, degree in indegree.items() if degree)}")

    receipt = {
        "schema_version": "org.searchright.rust-dependency-graph-receipt.v1",
        "status": "failed" if errors else "passed",
        "workspace_packages": len(packages),
        "internal_edges": sum(len(value) for value in graph.values()),
        "source_references": sum(len(value) for value in source_references.values()),
        "publishable_packages": sum(
            manifest["package"].get("publish") is not False for manifest in manifests.values()
        ),
        "non_publishable_packages": sum(
            manifest["package"].get("publish") is False for manifest in manifests.values()
        ),
        "topological_order": ordered,
        "errors": errors,
        "limitations": [
            "Lexical source-reference analysis only; Cargo feature resolution and compilation were not executed.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
