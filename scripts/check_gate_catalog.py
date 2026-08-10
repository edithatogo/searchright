#!/usr/bin/env python3
"""Generate or verify the executable gate catalogue.

The catalogue makes an explicit distinction between a command being present in
CI and the maximum evidence that command can establish. It is generated from
the static harness plus assertion-level traceability, while the classification
rules in this file remain reviewable policy rather than inferred claims.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import shlex
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "verification" / "gate-catalog.json"
TRACEABILITY_ROOT = ROOT / "conductor" / "tracks"

SOURCE_REPRODUCIBILITY = {
    "scripts/check_packaging_reproducibility.py",
    "scripts/generate_source_hash_manifest.py",
    "scripts/generate_source_sbom.py",
}
CONTROL_PLANE = {
    "scripts/audit_github_control_plane.py",
    "scripts/bootstrap_github.py",
    "scripts/check_github_issue_hierarchy.py",
    "scripts/check_github_project.py",
    "scripts/render_github_issues.py",
    "scripts/sync_github_issues.py",
    "scripts/sync_github_project.py",
}
SAFETY = {
    "scripts/check_default_deny.py",
    "scripts/check_licence_firewall.py",
    "scripts/check_secrets.py",
    "scripts/check_workflow_hardening.py",
}
REFERENCE_MODELS = {
    "scripts/check_vertical_slice.py",
    "scripts/reduce_review_events.py",
    "scripts/review_bundle.py",
    "scripts/recovery_rehearsal.py",
    "scripts/check_redaction_policy.py",
}


def normalise_command(value: str | list[str]) -> str:
    """Return a stable command string independent of the Python executable path."""
    parts = shlex.split(value) if isinstance(value, str) else [str(item) for item in value]
    if parts and Path(parts[0]).name.startswith("python"):
        parts[0] = "python"
    return " ".join(shlex.quote(part) if any(ch.isspace() for ch in part) else part for part in parts)


def script_path(command: str) -> str:
    parts = shlex.split(command)
    for part in parts:
        if part.startswith("scripts/"):
            return part
    raise ValueError(f"gate command has no scripts/ path: {command}")


def gate_slug(command: str) -> str:
    parts = shlex.split(command)
    script = Path(script_path(command)).stem.replace("_", "-")
    qualifiers = [
        part.strip("-").replace("_", "-")
        for part in parts[2:]
        if part.startswith("--") or part in {"self-test"}
    ]
    suffix = "-" + "-".join(qualifiers) if qualifiers else ""
    return f"SR-GATE-{script.upper()}{suffix.upper()}"


def category_for(path: str) -> str:
    if path in SOURCE_REPRODUCIBILITY:
        return "source_reproducibility"
    if path in CONTROL_PLANE:
        return "control_plane_dry_run"
    if path in SAFETY:
        return "safety_policy"
    if path in REFERENCE_MODELS:
        return "deterministic_reference_model"
    stem = Path(path).stem
    if "schema" in stem or stem == "validate_repository":
        return "contract_validation"
    if "traceability" in stem or "roadmap" in stem or "maturity" in stem or "evidence" in stem:
        return "claim_evidence"
    if "integration" in stem or "release_train" in stem or "companion" in stem or "ecosystem" in stem:
        return "ecosystem_compatibility"
    if "provider" in stem or "query" in stem or "search" in stem:
        return "search_method"
    if "context" in stem:
        return "context_integrity"
    if "rust" in stem or "toolchain" in stem or "public_package" in stem:
        return "source_structure"
    if "sdk" in stem or "cli_mcp" in stem:
        return "interface_parity"
    if "release" in stem or "registry" in stem:
        return "release_preparation"
    return "repository_policy"


def load_harness_commands() -> list[str]:
    spec = importlib.util.spec_from_file_location(
        "searchright_static_harness", ROOT / "scripts" / "run_static_harness.py"
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load static harness")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return [normalise_command(command) for command in module.COMMANDS]


def assertion_coverage() -> tuple[dict[str, list[str]], set[str]]:
    by_command: dict[str, list[str]] = defaultdict(list)
    all_commands: set[str] = set()
    for path in sorted(TRACEABILITY_ROOT.glob("*/traceability.json")):
        value = json.loads(path.read_text(encoding="utf-8"))
        for assertion in value.get("assertions", []):
            assertion_id = assertion.get("assertion_id")
            for raw in assertion.get("deterministic_tests", []):
                command = normalise_command(raw)
                all_commands.add(command)
                if assertion_id:
                    by_command[command].append(assertion_id)
    return by_command, all_commands


def render() -> dict[str, Any]:
    harness_commands = load_harness_commands()
    covered, traceability_commands = assertion_coverage()
    ordered = harness_commands + sorted(traceability_commands - set(harness_commands))
    gates = []
    for command in ordered:
        path = script_path(command)
        harness_gate = command in harness_commands
        gates.append(
            {
                "gate_id": gate_slug(command),
                "command": command,
                "script": path,
                "category": category_for(path),
                "harness_gate": harness_gate,
                "network": False,
                "external_writes": False,
                "compiler_required": False,
                "evidence_ceiling": (
                    "source_reproducible" if path in SOURCE_REPRODUCIBILITY else "source_verified"
                ),
                "covered_assertions": sorted(set(covered.get(command, []))),
                "claim_boundary": (
                    "Passing this gate establishes only the declared source-level property; "
                    "it does not establish compilation, live-provider behaviour, remote state, "
                    "methodological adequacy or external acceptance."
                ),
            }
        )
    return {
        "schema_version": "org.searchright.gate-catalog.v1",
        "generated_from": [
            "scripts/run_static_harness.py",
            "conductor/tracks/*/traceability.json",
            "scripts/check_gate_catalog.py",
        ],
        "default_capabilities": {
            "network": False,
            "external_writes": False,
            "compiler_required": False,
        },
        "gates": gates,
    }


def validate(value: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    gates = value.get("gates")
    if not isinstance(gates, list) or not gates:
        return ["gate catalogue must contain a non-empty gates array"]
    ids: set[str] = set()
    commands: set[str] = set()
    for gate in gates:
        if not isinstance(gate, dict):
            errors.append("gate entries must be objects")
            continue
        identifier = gate.get("gate_id")
        command = gate.get("command")
        if not isinstance(identifier, str) or not identifier.startswith("SR-GATE-"):
            errors.append(f"invalid gate id {identifier!r}")
        elif identifier in ids:
            errors.append(f"duplicate gate id {identifier}")
        ids.add(str(identifier))
        if not isinstance(command, str) or not command.startswith("python scripts/"):
            errors.append(f"invalid gate command {command!r}")
        elif command in commands:
            errors.append(f"duplicate gate command {command}")
        commands.add(str(command))
        for capability in ("network", "external_writes", "compiler_required"):
            if gate.get(capability) is not False:
                errors.append(f"{identifier} must keep static capability {capability}=false")
        if gate.get("evidence_ceiling") not in {"source_verified", "source_reproducible"}:
            errors.append(f"{identifier} has invalid evidence ceiling")
        if not isinstance(gate.get("covered_assertions"), list):
            errors.append(f"{identifier}.covered_assertions must be an array")
        path = ROOT / str(gate.get("script", ""))
        if not path.is_file():
            errors.append(f"{identifier} references missing script {gate.get('script')}")
    harness = set(load_harness_commands())
    marked_harness = {gate["command"] for gate in gates if gate.get("harness_gate")}
    if harness != marked_harness:
        errors.append(
            f"harness catalogue mismatch: missing={sorted(harness-marked_harness)}, "
            f"extra={sorted(marked_harness-harness)}"
        )
    _, traceability = assertion_coverage()
    if not traceability.issubset(commands):
        errors.append(f"traceability commands absent from gate catalogue: {sorted(traceability-commands)}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()

    expected_value = render()
    expected = json.dumps(expected_value, indent=2, sort_keys=True) + "\n"
    stale = not CATALOG.is_file() or CATALOG.read_text(encoding="utf-8") != expected
    if args.write:
        CATALOG.parent.mkdir(parents=True, exist_ok=True)
        CATALOG.write_text(expected, encoding="utf-8")
        stale = False
    errors = validate(expected_value)
    if args.check and stale:
        errors.append("verification/gate-catalog.json is stale; run check_gate_catalog.py --write")
    receipt = {
        "schema_version": "org.searchright.gate-catalog-receipt.v1",
        "status": "failed" if errors else "passed",
        "mode": "write" if args.write else "check" if args.check else "inspect",
        "gates": len(expected_value["gates"]),
        "harness_gates": sum(1 for gate in expected_value["gates"] if gate["harness_gate"]),
        "assertions_with_gate_coverage": len(
            {assertion for gate in expected_value["gates"] for assertion in gate["covered_assertions"]}
        ),
        "stale": stale,
        "errors": errors,
        "limitations": [
            "The catalogue constrains source-level claims; it does not upgrade evidence beyond each gate's declared ceiling."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
