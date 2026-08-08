#!/usr/bin/env python3
"""Generate a reproducible CycloneDX source-component inventory.

This is deliberately labelled a source inventory when Cargo.lock is absent. It
must not be represented as a fully resolved binary SBOM.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
import uuid
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "verification" / "sbom" / "source-components.cdx.json"
NAMESPACE = uuid.UUID("1374d540-6dd8-55aa-8a46-65cae50992aa")


def component_ref(kind: str, name: str, version: str) -> str:
    return f"{kind}:{name}@{version}"


def generate() -> dict[str, Any]:
    root = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    workspace = root["workspace"]
    package_defaults = workspace["package"]
    version = package_defaults["version"]
    components: list[dict[str, Any]] = []
    dependencies: list[dict[str, Any]] = []
    package_refs: dict[str, str] = {}

    for member_value in sorted(workspace["members"]):
        member = ROOT / member_value
        manifest = tomllib.loads((member / "Cargo.toml").read_text(encoding="utf-8"))
        package = manifest["package"]
        name = package["name"]
        package_version = package.get("version", version)
        if isinstance(package_version, dict):
            package_version = version
        bom_ref = component_ref("cargo", name, str(package_version))
        package_refs[name] = bom_ref
        components.append(
            {
                "type": "library" if "lib.rs" in {path.name for path in (member / "src").glob("*.rs")} else "application",
                "bom-ref": bom_ref,
                "name": name,
                "version": str(package_version),
                "purl": f"pkg:cargo/{name}@{package_version}",
                "licenses": [{"expression": package.get("license", package_defaults["license"])}],
                "properties": [
                    {"name": "searchright:path", "value": member_value},
                    {"name": "searchright:component-kind", "value": "workspace-crate"},
                ],
            }
        )

    external_versions: dict[str, str] = {}
    for name, value in sorted(workspace.get("dependencies", {}).items()):
        if isinstance(value, str):
            external_versions[name] = value
        elif isinstance(value, dict) and isinstance(value.get("version"), str):
            external_versions[name] = value["version"]
    for name, requirement in external_versions.items():
        bom_ref = component_ref("cargo-requirement", name, requirement)
        components.append(
            {
                "type": "library",
                "bom-ref": bom_ref,
                "name": name,
                "version": requirement,
                "purl": f"pkg:cargo/{name}@{requirement}",
                "scope": "required",
                "properties": [
                    {"name": "searchright:resolution", "value": "version-requirement-not-resolved"},
                ],
            }
        )

    for member_value in sorted(workspace["members"]):
        manifest = tomllib.loads((ROOT / member_value / "Cargo.toml").read_text(encoding="utf-8"))
        package_name = manifest["package"]["name"]
        refs: set[str] = set()
        for section in ("dependencies", "dev-dependencies", "build-dependencies"):
            for name, value in manifest.get(section, {}).items():
                canonical = value.get("package", name) if isinstance(value, dict) else name
                if canonical in package_refs:
                    refs.add(package_refs[canonical])
                elif canonical in external_versions:
                    refs.add(component_ref("cargo-requirement", canonical, external_versions[canonical]))
        dependencies.append({"ref": package_refs[package_name], "dependsOn": sorted(refs)})

    action_pattern = re.compile(r"uses:\s*([^@\s]+)@([0-9a-f]{40})")
    for workflow in sorted((ROOT / ".github" / "workflows").glob("*.yml")):
        for action, digest in action_pattern.findall(workflow.read_text(encoding="utf-8")):
            ref = component_ref("github-action", action, digest)
            if any(component.get("bom-ref") == ref for component in components):
                continue
            components.append(
                {
                    "type": "application",
                    "bom-ref": ref,
                    "name": action,
                    "version": digest,
                    "properties": [
                        {"name": "searchright:component-kind", "value": "github-action"},
                    ],
                }
            )

    schema_catalog = json.loads((ROOT / "contracts" / "schema-catalog.json").read_text(encoding="utf-8"))
    standard_catalog = json.loads((ROOT / "contracts" / "standards" / "index.json").read_text(encoding="utf-8"))
    lockfile_present = (ROOT / "Cargo.lock").is_file()
    serial = uuid.uuid5(NAMESPACE, json.dumps(sorted(component["bom-ref"] for component in components)))
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "serialNumber": f"urn:uuid:{serial}",
        "version": 1,
        "metadata": {
            "component": {
                "type": "application",
                "name": "searchright",
                "version": version,
                "purl": f"pkg:github/edithatogo/searchright@{version}",
            },
            "properties": [
                {"name": "searchright:evidence-level", "value": "source-inventory" if not lockfile_present else "lockfile-resolved-source-inventory"},
                {"name": "searchright:cargo-lock-present", "value": str(lockfile_present).lower()},
                {"name": "searchright:schema-count", "value": str(len(schema_catalog["entries"]))},
                {"name": "searchright:standard-pack-count", "value": str(len(standard_catalog["packs"]))},
                {"name": "searchright:claim-boundary", "value": "not-a-binary-sbom-without-compiler-and-release-artifact"},
            ],
        },
        "components": sorted(components, key=lambda value: value["bom-ref"]),
        "dependencies": sorted(dependencies, key=lambda value: value["ref"]),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    content = json.dumps(generate(), indent=2, sort_keys=True) + "\n"
    if args.check:
        if not OUTPUT.is_file():
            print(f"missing {OUTPUT.relative_to(ROOT)}", file=sys.stderr)
            return 1
        if OUTPUT.read_text(encoding="utf-8") != content:
            print(f"stale {OUTPUT.relative_to(ROOT)}", file=sys.stderr)
            return 1
        print(json.dumps({"status": "passed", "path": str(OUTPUT.relative_to(ROOT))}, indent=2))
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(content, encoding="utf-8")
    print(OUTPUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
