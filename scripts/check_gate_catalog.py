#!/usr/bin/env python3
"""Generate or verify the executable gate catalogue.

The catalogue distinguishes repository scripts from native compiler and tooling
commands. It records the maximum evidence each command could establish if it is
actually executed, without promoting a command merely because it is listed in
traceability.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import shlex
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
RUST_PROGRAMS = {"cargo", "rustc", "rustdoc", "rustfmt", "clippy-driver"}
NODE_PROGRAMS = {"node", "npm", "npx", "pnpm", "yarn"}
PYTHON_PROGRAMS = {"python", "python3", "py", "pypy", "pypy3"}
SHELL_CONTROL_TOKENS = {"&&", "||", ";", "|", "&", ">", ">>", "<", "<<"}
ALLOWED_KINDS = {
    "repository_script",
    "rust_toolchain",
    "python_module",
    "node_toolchain",
    "external_tool",
}
ALLOWED_CEILINGS = {"source_verified", "source_reproducible", "compiler_verified"}


def normalise_command(value: str | list[str]) -> str:
    """Return a stable command string independent of the Python executable path."""
    parts = shlex.split(value) if isinstance(value, str) else [str(item) for item in value]
    if parts and Path(parts[0]).name.startswith("python"):
        parts[0] = "python"
    return " ".join(
        shlex.quote(part) if any(ch.isspace() for ch in part) else part for part in parts
    )


def command_parts(command: str) -> list[str]:
    parts = shlex.split(command)
    if not parts:
        raise ValueError("gate command is empty")
    if any(part in SHELL_CONTROL_TOKENS for part in parts):
        raise ValueError(f"compound shell gate commands are not allowed: {command}")
    return parts


def script_path(command: str) -> str | None:
    """Return the repository script path when the command invokes one."""
    for part in command_parts(command):
        if part.startswith("scripts/"):
            return part
    return None


def command_kind(command: str, path: str | None) -> str:
    if path is not None:
        return "repository_script"
    program = Path(command_parts(command)[0]).name.lower()
    if program in RUST_PROGRAMS:
        return "rust_toolchain"
    if program in PYTHON_PROGRAMS:
        return "python_module"
    if program in NODE_PROGRAMS:
        return "node_toolchain"
    return "external_tool"


def gate_slug(command: str) -> str:
    parts = command_parts(command)
    path = script_path(command)
    if path is not None:
        script = Path(path).stem.replace("_", "-")
        qualifiers = [
            part.strip("-").replace("_", "-")
            for part in parts[2:]
            if part.startswith("--") or part in {"self-test"}
        ]
        suffix = "-" + "-".join(qualifiers) if qualifiers else ""
        return f"SR-GATE-{script.upper()}{suffix.upper()}"
    program = re.sub(r"[^A-Za-z0-9]+", "-", Path(parts[0]).name).strip("-")
    subcommand = ""
    if len(parts) > 1 and not parts[1].startswith("-"):
        subcommand = "-" + re.sub(r"[^A-Za-z0-9]+", "-", parts[1]).strip("-")
    digest = hashlib.sha256(command.encode("utf-8")).hexdigest()[:10]
    return f"SR-GATE-{program}{subcommand}-{digest}".upper()


def category_for(path: str | None, kind: str) -> str:
    if kind == "rust_toolchain":
        return "compiler_validation"
    if kind == "python_module":
        return "python_runtime_validation"
    if kind == "node_toolchain":
        return "node_runtime_validation"
    if kind == "external_tool":
        return "external_tool_validation"
    assert path is not None
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


def capabilities(command: str, kind: str, path: str | None) -> dict[str, Any]:
    parts = command_parts(command)
    if kind == "repository_script":
        return {
            "network": False,
            "external_writes": False,
            "compiler_required": False,
            "evidence_ceiling": (
                "source_reproducible" if path in SOURCE_REPRODUCIBILITY else "source_verified"
            ),
        }
    if kind == "rust_toolchain":
        offline = "--offline" in parts or "--frozen" in parts
        return {
            "network": not offline,
            "external_writes": False,
            "compiler_required": True,
            "evidence_ceiling": "compiler_verified",
        }
    if kind == "node_toolchain":
        offline = "--offline" in parts or "--prefer-offline" in parts
        return {
            "network": not offline,
            "external_writes": False,
            "compiler_required": False,
            "evidence_ceiling": "source_verified",
        }
    return {
        "network": False,
        "external_writes": False,
        "compiler_required": False,
        "evidence_ceiling": "source_verified",
    }


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
    gates: list[dict[str, Any]] = []
    for command in ordered:
        path = script_path(command)
        kind = command_kind(command, path)
        profile = capabilities(command, kind, path)
        gate = {
            "gate_id": gate_slug(command),
            "command": command,
            "command_kind": kind,
            "program": Path(command_parts(command)[0]).name,
            "script": path,
            "category": category_for(path, kind),
            "harness_gate": command in harness_commands,
            **profile,
            "covered_assertions": sorted(set(covered.get(command, []))),
            "claim_boundary": (
                "Catalogue presence does not establish execution. A passing receipt may establish "
                "only this gate's declared evidence ceiling; it does not establish live-provider "
                "behaviour, remote state, methodological adequacy or external acceptance."
            ),
        }
        gates.append(gate)
    return {
        "schema_version": "org.searchright.gate-catalog.v2",
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
    if value.get("schema_version") != "org.searchright.gate-catalog.v2":
        errors.append("unsupported gate catalogue schema_version")
    gates = value.get("gates")
    if not isinstance(gates, list) or not gates:
        return errors + ["gate catalogue must contain a non-empty gates array"]
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
        if not isinstance(command, str) or not command.strip():
            errors.append(f"invalid gate command {command!r}")
            continue
        if command in commands:
            errors.append(f"duplicate gate command {command}")
        commands.add(command)
        try:
            parts = command_parts(command)
            expected_path = script_path(command)
            expected_kind = command_kind(command, expected_path)
        except ValueError as error:
            errors.append(str(error))
            continue
        if gate.get("command_kind") not in ALLOWED_KINDS:
            errors.append(f"{identifier} has invalid command_kind")
        elif gate.get("command_kind") != expected_kind:
            errors.append(f"{identifier} command_kind differs from command")
        if gate.get("program") != Path(parts[0]).name:
            errors.append(f"{identifier} program differs from command")
        if gate.get("script") != expected_path:
            errors.append(f"{identifier} script differs from command")
        if expected_path is not None and not (ROOT / expected_path).is_file():
            errors.append(f"{identifier} references missing script {expected_path}")
        expected_profile = capabilities(command, expected_kind, expected_path)
        for capability in ("network", "external_writes", "compiler_required"):
            if type(gate.get(capability)) is not bool:
                errors.append(f"{identifier}.{capability} must be boolean")
            elif gate.get(capability) != expected_profile[capability]:
                errors.append(f"{identifier}.{capability} differs from command profile")
        if gate.get("evidence_ceiling") not in ALLOWED_CEILINGS:
            errors.append(f"{identifier} has invalid evidence ceiling")
        elif gate.get("evidence_ceiling") != expected_profile["evidence_ceiling"]:
            errors.append(f"{identifier} evidence ceiling differs from command profile")
        if gate.get("harness_gate") and expected_kind != "repository_script":
            errors.append(f"{identifier} native command cannot be a static-harness gate")
        if not isinstance(gate.get("covered_assertions"), list):
            errors.append(f"{identifier}.covered_assertions must be an array")
    harness = set(load_harness_commands())
    marked_harness = {gate["command"] for gate in gates if gate.get("harness_gate")}
    if harness != marked_harness:
        errors.append(
            f"harness catalogue mismatch: missing={sorted(harness-marked_harness)}, "
            f"extra={sorted(marked_harness-harness)}"
        )
    _, traceability = assertion_coverage()
    if not traceability.issubset(commands):
        errors.append(
            f"traceability commands absent from gate catalogue: {sorted(traceability-commands)}"
        )
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
        "schema_version": "org.searchright.gate-catalog-receipt.v2",
        "status": "failed" if errors else "passed",
        "mode": "write" if args.write else "check" if args.check else "inspect",
        "gates": len(expected_value["gates"]),
        "harness_gates": sum(1 for gate in expected_value["gates"] if gate["harness_gate"]),
        "native_gates": sum(
            1 for gate in expected_value["gates"] if gate["command_kind"] != "repository_script"
        ),
        "assertions_with_gate_coverage": len(
            {assertion for gate in expected_value["gates"] for assertion in gate["covered_assertions"]}
        ),
        "stale": stale,
        "errors": errors,
        "limitations": [
            "The catalogue records maximum evidence ceilings; catalogue presence is not execution evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
