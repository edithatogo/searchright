#!/usr/bin/env python3
"""Validate canonical context, hazards, capabilities and claim boundaries."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load(path: str) -> dict:
    value = json.loads((ROOT / path).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def local_path_exists(value: str) -> bool:
    return value.startswith("external://") or (ROOT / value).exists()


def main() -> int:
    errors: list[str] = []
    manifest = load("context/manifest.json")
    claims = load("context/claim-boundaries.json")
    decisions = load("context/decision-ledger.json")
    capabilities = load("context/capability-matrix.json")
    hazards = load("context/hazard-log.json")
    evidence = load("context/evidence-ledger.json")

    ceilings = {
        manifest.get("evidence_ceiling"),
        claims.get("current_evidence_ceiling"),
        evidence.get("repository_level"),
    }
    if ceilings != {"source_verified"}:
        errors.append(f"context evidence ceilings disagree: {sorted(str(x) for x in ceilings)}")

    required = manifest.get("required_context", [])
    required_paths = [item.get("path") for item in required if isinstance(item, dict)]
    if len(required_paths) != len(set(required_paths)) or any(not path for path in required_paths):
        errors.append("required context paths must be unique and non-empty")
    for path in required_paths:
        if not (ROOT / str(path)).exists():
            errors.append(f"missing context path {path}")
    load_order = manifest.get("load_order", [])
    if not isinstance(load_order, list) or len(load_order) != len(set(load_order)):
        errors.append("context load_order must be a unique list")
    for path in load_order if isinstance(load_order, list) else []:
        if not (ROOT / str(path)).exists():
            errors.append(f"load_order references missing path {path}")

    decision_items = decisions.get("decisions", [])
    decision_ids = [item.get("id") for item in decision_items if isinstance(item, dict)]
    decision_id_set = {identifier for identifier in decision_ids if identifier}
    if len(decision_ids) != len(decision_id_set) or any(not identifier for identifier in decision_ids):
        errors.append("decision IDs must be unique and non-empty")
    for decision in decision_items:
        if not isinstance(decision, dict):
            errors.append("decision entries must be objects")
            continue
        identifier = decision.get("id")
        if not decision.get("decision") or not decision.get("rationale") or not decision.get("status"):
            errors.append(f"decision {identifier} lacks decision/rationale/status")
        evidence_paths = decision.get("evidence")
        if not isinstance(evidence_paths, list) or not evidence_paths:
            errors.append(f"decision {identifier} lacks evidence")
        else:
            for path in evidence_paths:
                if not isinstance(path, str) or not local_path_exists(path):
                    errors.append(f"decision {identifier} references missing evidence {path}")
        for superseded in decision.get("supersedes", []):
            if superseded not in decision_id_set:
                errors.append(f"decision {identifier} supersedes unknown decision {superseded}")

    hazard_items = hazards.get("hazards", [])
    hazard_ids = [item.get("id") for item in hazard_items if isinstance(item, dict)]
    if len(hazard_ids) != len(set(hazard_ids)) or any(not identifier for identifier in hazard_ids):
        errors.append("hazard IDs must be unique and non-empty")
    for hazard in hazard_items:
        if not isinstance(hazard, dict):
            errors.append("hazard entries must be objects")
            continue
        if not hazard.get("controls") or not hazard.get("evidence") or not hazard.get("residual_status"):
            errors.append(f"hazard {hazard.get('id')} lacks controls/evidence/status")
        for path in hazard.get("evidence", []):
            if not isinstance(path, str) or not local_path_exists(path):
                errors.append(f"hazard {hazard.get('id')} references missing evidence {path}")

    if capabilities.get("default_policy") != "deny":
        errors.append("capability default must be deny")
    component_names: list[str] = []
    for component in capabilities.get("components", []):
        if not isinstance(component, dict):
            errors.append("capability component entries must be objects")
            continue
        component_names.append(str(component.get("component", "")))
        for field in ("network", "external_writes", "telemetry", "final_exclusion"):
            if component.get(field) is not False:
                errors.append(f"{component.get('component')} default {field} must be false")
    if len(component_names) != len(set(component_names)) or any(not name for name in component_names):
        errors.append("capability component names must be unique and non-empty")

    forbidden = claims.get("forbidden_without_additional_evidence", [])
    if not isinstance(forbidden, list) or not forbidden:
        errors.append("claim boundaries require forbidden claims")
    for item in forbidden if isinstance(forbidden, list) else []:
        if not isinstance(item, dict) or not item.get("claim") or not item.get("required_evidence"):
            errors.append("every forbidden claim requires claim and required_evidence")
    if not claims.get("allowed_claims") or not claims.get("language_rules"):
        errors.append("claim boundaries require allowed claims and language rules")

    domains = evidence.get("domains", [])
    domain_names = [item.get("domain") for item in domains if isinstance(item, dict)]
    if len(domain_names) != len(set(domain_names)) or any(not name for name in domain_names):
        errors.append("evidence domains must be unique and non-empty")
    for domain in domains:
        if not isinstance(domain, dict) or not domain.get("level") or "blockers" not in domain:
            errors.append(f"invalid evidence domain {domain}")
            continue
        receipt = domain.get("receipt")
        if receipt is not None and (not isinstance(receipt, str) or not (ROOT / receipt).exists()):
            errors.append(f"evidence domain {domain.get('domain')} references missing receipt {receipt}")

    for filename in ("AGENTS.md", "CLAUDE.md", "GEMINI.md", ".github/copilot-instructions.md"):
        text = (ROOT / filename).read_text(encoding="utf-8")
        if "CONTEXT.md" not in text:
            errors.append(f"{filename} must reference CONTEXT.md")

    receipt = {
        "schema_version": "org.searchright.context-integrity-receipt.v1",
        "status": "failed" if errors else "passed",
        "required_context": len(required_paths),
        "decisions": len(decision_ids),
        "hazards": len(hazard_ids),
        "capability_components": len(component_names),
        "claim_boundaries": len(forbidden) if isinstance(forbidden, list) else 0,
        "evidence_domains": len(domain_names),
        "errors": errors,
        "limitations": [
            "Static context consistency only; human interpretation and runtime behaviour remain separate evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
