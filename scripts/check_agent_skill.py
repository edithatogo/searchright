#!/usr/bin/env python3
"""Validate the systematic-search skill package and deterministic safety cases."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
SKILL_ROOT = ROOT / "skills" / "systematic-search"
SKILL = SKILL_ROOT / "SKILL.md"
WORKFLOW = SKILL_ROOT / "workflows" / "systematic-review.yaml"
SCENARIOS = SKILL_ROOT / "evaluations" / "authority-scenarios.json"
CALLER = SKILL_ROOT / "integrations" / "academic-research-skills" / "SKILL.md"
PACKET = ROOT / "registry" / "skills" / "systematic-search" / "manifest.json"
RECEIPT = ROOT / "verification" / "receipts" / "systematic-search-skill.json"

EXPECTED_STAGES = [
    "scope",
    "strategy",
    "press",
    "execute",
    "deduplicate",
    "screen",
    "report",
]
EXPECTED_ROLES = [
    "question-framer",
    "information-specialist",
    "press-reviewer",
    "execution-operator",
    "dedup-adjudicator",
    "screening-assistant",
    "reporting-auditor",
]
REQUIRED_REFERENCES = [
    "authority.md",
    "failure-modes.md",
    "handoffs.md",
    "methodology.md",
    "tool-map.md",
]


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path.relative_to(ROOT)} must contain a JSON object")
    return value


def frontmatter(path: Path) -> tuple[dict[str, Any], str]:
    text = path.read_text(encoding="utf-8")
    match = re.match(r"\A---\n(.*?)\n---\n(.*)\Z", text, re.DOTALL)
    if match is None:
        raise ValueError(f"{path.relative_to(ROOT)} requires YAML front matter")
    metadata = yaml.safe_load(match.group(1))
    if not isinstance(metadata, dict):
        raise ValueError(f"{path.relative_to(ROOT)} front matter must be a mapping")
    return metadata, match.group(2)


def package_digest() -> tuple[str, int]:
    digest = hashlib.sha256()
    paths = sorted(path for path in SKILL_ROOT.rglob("*") if path.is_file())
    for path in paths:
        relative = path.relative_to(ROOT).as_posix().encode("utf-8")
        content = path.read_bytes()
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(len(content).to_bytes(8, "big"))
        digest.update(content)
    return digest.hexdigest(), len(paths)


def validate(*, check_receipt: bool = True) -> tuple[list[str], dict[str, Any]]:
    errors: list[str] = []
    try:
        metadata, skill_text = frontmatter(SKILL)
    except (OSError, ValueError, yaml.YAMLError) as error:
        return [str(error)], {}

    if metadata.get("name") != "systematic-search":
        errors.append("skill name must be systematic-search")
    description = str(metadata.get("description", ""))
    for phrase in ("Do not use", "final eligibility", "licensed database"):
        if phrase.lower() not in description.lower():
            errors.append(f"skill description lacks non-trigger boundary {phrase!r}")
    for heading in ("## Trigger boundary", "## Non-negotiable rules", "## Failure handling"):
        if heading not in skill_text:
            errors.append(f"skill is missing {heading}")
    for reference in REQUIRED_REFERENCES:
        if not (SKILL_ROOT / "references" / reference).is_file():
            errors.append(f"missing skill reference {reference}")
        if reference not in skill_text:
            errors.append(f"skill entrypoint does not route to {reference}")
    for phrase in ("telemetry disabled", "credentials", "full text", "explicit allowlist"):
        if phrase.lower() not in skill_text.lower():
            errors.append(f"portable skill lacks data boundary phrase {phrase!r}")

    methodology = re.sub(
        r"\s+",
        " ",
        (SKILL_ROOT / "references" / "methodology.md").read_text(encoding="utf-8"),
    )
    for phrase in (
        "translation of the question",
        "Boolean and proximity operators",
        "subject headings",
        "text words",
        "spelling, syntax and line numbers",
        "limits and filters",
        "reporting completeness, not a conduct certificate",
    ):
        if phrase.lower() not in methodology.lower():
            errors.append(f"methodology reference lacks required PRESS/reporting boundary {phrase!r}")

    workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
    if not isinstance(workflow, dict):
        errors.append("workflow must be a mapping")
        workflow = {}
    if workflow.get("schema_version") != "org.searchright.skill-workflow.v1":
        errors.append("unexpected skill workflow schema version")
    if workflow.get("handoff_contract") != "org.searchright.agent-handoff.v1":
        errors.append("workflow must bind the versioned agent handoff contract")
    if workflow.get("handoff_policy") != "approved_artifact_references_only":
        errors.append("workflow handoffs must use approved artefact references only")
    checkpoints = workflow.get("human_checkpoints", [])
    for checkpoint in (
        "review_plan_approval",
        "strategy_and_press_approval",
        "live_execution_approval",
        "deduplication_apply",
        "screening_conflict_resolution",
        "final_reporting_review",
    ):
        if checkpoint not in checkpoints:
            errors.append(f"workflow lacks human checkpoint {checkpoint}")
    stages = workflow.get("stages", [])
    stage_ids = [stage.get("id") for stage in stages if isinstance(stage, dict)]
    roles = [stage.get("agent") for stage in stages if isinstance(stage, dict)]
    if stage_ids != EXPECTED_STAGES:
        errors.append(f"workflow stages must be ordered as {EXPECTED_STAGES}")
    if roles != EXPECTED_ROLES:
        errors.append(f"workflow roles must be ordered as {EXPECTED_ROLES}")
    stage_by_id = {
        stage.get("id"): stage for stage in stages if isinstance(stage, dict) and stage.get("id")
    }
    press = stage_by_id.get("press", {})
    execute = stage_by_id.get("execute", {})
    screen = stage_by_id.get("screen", {})
    deduplicate = stage_by_id.get("deduplicate", {})
    if press.get("isolation") != "independent_context":
        errors.append("PRESS stage must use independent context")
    if "live_execution_approval" not in execute.get("requires", []):
        errors.append("execution stage must require live execution approval")
    if screen.get("authority") != "advisory_only":
        errors.append("screening assistant must remain advisory only")
    if "deduplication_apply" not in deduplicate.get("requires", []):
        errors.append("deduplication stage must require the human apply checkpoint")
    for index, role in enumerate(EXPECTED_ROLES[:-1]):
        stage = stages[index] if index < len(stages) and isinstance(stages[index], dict) else {}
        if stage.get("handoff_to") != EXPECTED_ROLES[index + 1]:
            errors.append(f"workflow handoff from {role} is missing or misdirected")

    for role in EXPECTED_ROLES:
        role_path = SKILL_ROOT / "agents" / f"{role}.md"
        if not role_path.is_file():
            errors.append(f"missing role card {role}")
            continue
        role_text = role_path.read_text(encoding="utf-8")
        for heading in ("## Role", "## Required inputs", "## Output", "## Stop conditions"):
            if heading not in role_text:
                errors.append(f"role card {role} lacks {heading}")

    scenarios = load_json(SCENARIOS)
    if scenarios.get("schema_version") != "org.searchright.agent-scenario-suite.v1":
        errors.append("unexpected authority scenario schema version")
    if scenarios.get("status") != "deterministic_fixture_ready_external_host_model_evaluation_pending":
        errors.append("scenario status must retain the external host/model evidence boundary")
    cases = scenarios.get("cases", [])
    identifiers = [case.get("id") for case in cases if isinstance(case, dict)]
    if len(identifiers) < 8 or len(identifiers) != len(set(identifiers)):
        errors.append("authority scenarios must contain at least eight uniquely identified cases")
    operations = {
        case.get("request", {}).get("operation")
        for case in cases
        if isinstance(case, dict) and isinstance(case.get("request"), dict)
    }
    for operation in ("live_execution", "final_exclusion", "protocol_amendment", "registry_publication"):
        if operation not in operations:
            errors.append(f"authority scenarios do not cover {operation}")
    injection_cases = [
        case
        for case in cases
        if isinstance(case, dict)
        and "ignore" in str(case.get("request", {}).get("untrusted_content", "")).lower()
    ]
    if not injection_cases or any(case.get("expected", {}).get("allowed") is not False for case in injection_cases):
        errors.append("prompt-injection scenarios must be present and denied")

    caller_metadata, caller_text = frontmatter(CALLER)
    if caller_metadata.get("metadata", {}).get("status") != "prepared_not_applied":
        errors.append("academic-research-skills caller must remain prepared_not_applied")
    caller_description = str(caller_metadata.get("description", ""))
    caller_boundary_text = re.sub(r"\s+", " ", caller_description + " " + caller_text)
    for phrase in (
        "does not implement providers",
        "untrusted data",
        "explicit approval",
        "human-only",
        "not live-provider",
        "disable automated tool invocation",
    ):
        if phrase.lower() not in caller_boundary_text.lower():
            errors.append(f"thin caller lacks boundary phrase {phrase!r}")
    if re.search(r"https?://[^\s`]+/(?:search|api|query)", caller_text, re.IGNORECASE):
        errors.append("thin caller must not embed provider endpoints")

    packet = load_json(PACKET)
    digest, files = package_digest()
    if packet.get("schema_version") != "org.searchright.skill-registry-packet.v1":
        errors.append("unexpected skill registry packet schema version")
    if packet.get("status") != "prepared_not_submitted" or packet.get("submission_authorized") is not False:
        errors.append("skill registry packet must remain prepared_not_submitted and unauthorized")
    if packet.get("entrypoint") != "skills/systematic-search/SKILL.md":
        errors.append("skill registry packet has the wrong entrypoint")
    if packet.get("package_sha256") != digest:
        errors.append("skill registry packet digest is stale; run check_agent_skill.py --write")

    receipt = {
        "schema_version": "org.searchright.agent-skill-verification-receipt.v1",
        "status": "failed" if errors else "passed",
        "skill": "systematic-search",
        "package_files": files,
        "package_sha256": digest,
        "workflow_stages": len(stage_ids),
        "role_cards": len(EXPECTED_ROLES),
        "authority_scenarios": len(cases),
        "downstream_integration": "prepared_not_applied",
        "registry_packet": "prepared_not_submitted",
        "errors": errors,
        "limitations": [
            "Static package checks and deterministic authority fixtures do not establish model behaviour or host compatibility.",
            "No companion repository, live provider, registry, screening decision or publication system was mutated.",
            "Human information-specialist calibration and downstream consumer validation remain separate evidence."
        ],
    }
    if check_receipt:
        try:
            checked_in_receipt = load_json(RECEIPT)
        except (OSError, ValueError, json.JSONDecodeError) as error:
            errors.append(str(error))
        else:
            if checked_in_receipt != receipt:
                errors.append("systematic-search verification receipt is stale; run check_agent_skill.py --write")
        receipt["errors"] = errors
        receipt["status"] = "failed" if errors else "passed"
    return errors, receipt


def write_generated() -> None:
    packet = load_json(PACKET)
    digest, _ = package_digest()
    packet["package_sha256"] = digest
    PACKET.write_text(json.dumps(packet, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    errors, receipt = validate(check_receipt=False)
    if errors:
        raise ValueError("; ".join(errors))
    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        try:
            write_generated()
        except (OSError, ValueError, json.JSONDecodeError, yaml.YAMLError) as error:
            print(json.dumps({"status": "failed", "errors": [str(error)]}, indent=2))
            return 1
    try:
        errors, receipt = validate()
    except (OSError, ValueError, json.JSONDecodeError, yaml.YAMLError) as error:
        errors = [str(error)]
        receipt = {"status": "failed", "errors": errors}
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
