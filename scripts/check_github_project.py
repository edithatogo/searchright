#!/usr/bin/env python3
"""Validate the declarative GitHub Project v2 and repository control plane."""
from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / "conductor/github/project.json"
HIERARCHY = ROOT / "conductor/github/issue-hierarchy.json"
SETTINGS = ROOT / "conductor/github/repository-settings.json"
ALLOWED_TYPES = {"SINGLE_SELECT", "TEXT", "NUMBER", "DATE"}
ALLOWED_LAYOUTS = {"BOARD_LAYOUT", "ROADMAP_LAYOUT", "TABLE_LAYOUT"}
REQUIRED_FIELDS = {
    "Delivery status",
    "Work kind",
    "Horizon",
    "Evidence level",
    "Implementation state",
    "MoSCoW",
    "External gate",
    "Track ID",
    "Phase",
    "Task",
    "Conductor key",
    "Conductor path",
    "Last sync",
}


def main() -> int:
    errors: list[str] = []
    project = json.loads(PROJECT.read_text(encoding="utf-8"))
    hierarchy = json.loads(HIERARCHY.read_text(encoding="utf-8"))
    settings = json.loads(SETTINGS.read_text(encoding="utf-8"))

    if project.get("schema_version") != "org.searchright.github-project.v1":
        errors.append("unexpected project schema version")
    if project.get("repository") != hierarchy.get("repository") or project.get("repository") != settings.get("repository"):
        errors.append("repository differs across project, hierarchy and settings manifests")
    if project.get("apply_permitted") is not False or settings.get("apply_permitted") is not False:
        errors.append("source manifests must not authorise remote apply")
    if project.get("project_number") is not None:
        errors.append("remote project number must remain external evidence")
    readme = ROOT / str(project.get("readme_path", ""))
    if not readme.is_file():
        errors.append("project readme path is missing")
    if project.get("visibility") not in {"public", "private"}:
        errors.append("project visibility must be public or private")

    fields = project.get("fields", [])
    names = [field.get("name") for field in fields if isinstance(field, dict)]
    if len(names) != len(set(names)) or any(not name for name in names):
        errors.append("project field names must be unique and non-empty")
    if set(names) != REQUIRED_FIELDS:
        errors.append(f"project fields differ: missing={sorted(REQUIRED_FIELDS-set(names))}, extra={sorted(set(names)-REQUIRED_FIELDS)}")
    field_by_name = {field["name"]: field for field in fields if isinstance(field, dict) and field.get("name")}
    for name, field in field_by_name.items():
        data_type = field.get("data_type")
        if data_type not in ALLOWED_TYPES:
            errors.append(f"field {name} has invalid data type {data_type}")
        options = field.get("options", [])
        if data_type == "SINGLE_SELECT":
            if not isinstance(options, list) or not options or len(options) != len(set(options)):
                errors.append(f"single-select field {name} requires unique options")
        elif options:
            errors.append(f"non-select field {name} must not declare options")

    views = project.get("views", [])
    view_names = [view.get("name") for view in views if isinstance(view, dict)]
    if len(view_names) != len(set(view_names)) or any(not name for name in view_names):
        errors.append("project view names must be unique and non-empty")
    if not {"Delivery board", "Roadmap", "Evidence blockers", "MVP", "Open tasks", "Implementation gaps"}.issubset(set(view_names)):
        errors.append("required project views are missing")
    for view in views:
        if not isinstance(view, dict) or view.get("layout") not in ALLOWED_LAYOUTS or not isinstance(view.get("filter"), str):
            errors.append(f"invalid project view {view}")

    sync = project.get("sync", {})
    if sync.get("hierarchy_path") != "conductor/github/issue-hierarchy.json":
        errors.append("project hierarchy path is not canonical")
    if sync.get("identity_field") != "Conductor key":
        errors.append("project identity must use the stable Conductor key")
    for policy in ("delete_policy", "archive_policy"):
        if "never" not in str(sync.get(policy, "")):
            errors.append(f"{policy} must default to never")
    if sync.get("promotion_policy") != "remote_state_cannot_promote_evidence":
        errors.append("remote Project state must not promote evidence")
    if sync.get("checkpoint_policy") != "ignored_atomic_resumable":
        errors.append("Project sync must use ignored atomic resumable checkpoints")
    if sync.get("partial_run_policy") != "canonical_order_resume_after":
        errors.append("Project partial runs must preserve canonical ordering")
    if sync.get("receipt_directory") != ".searchright/receipts":
        errors.append("observed control-plane receipts must stay in the ignored state directory")
    audit_path = ROOT / str(sync.get("remote_audit_path", ""))
    if not audit_path.is_file():
        errors.append("read-only remote control-plane audit is missing")
    interval = sync.get("minimum_interval_ms")
    if not isinstance(interval, int) or not 0 <= interval <= 2000:
        errors.append("Project minimum interval must be a bounded integer")

    node_fields = Counter()
    node_keys: set[str] = set()
    child_counts: Counter[str] = Counter()
    for node in hierarchy.get("nodes", []):
        key = node.get("key")
        if not key or key in node_keys:
            errors.append(f"duplicate or empty hierarchy node key {key}")
        node_keys.add(str(key))
        parent = node.get("parent_key")
        if parent:
            child_counts[str(parent)] += 1
        projected = node.get("project_fields", {})
        if not isinstance(projected, dict):
            errors.append(f"node {key} lacks project field projection")
            continue
        if set(projected) - set(field_by_name):
            errors.append(f"node {key} projects unknown fields {sorted(set(projected)-set(field_by_name))}")
        if projected.get("Conductor key") != key:
            errors.append(f"node {key} has a mismatched Conductor key")
        for name, value in projected.items():
            node_fields[name] += 1
            field = field_by_name.get(name, {})
            data_type = field.get("data_type")
            if data_type == "SINGLE_SELECT" and value not in field.get("options", []):
                errors.append(f"node {key} value {value!r} is not an option for {name}")
            if data_type == "NUMBER" and not isinstance(value, int):
                errors.append(f"node {key} value for {name} must be an integer")
            if data_type == "TEXT" and not isinstance(value, str):
                errors.append(f"node {key} value for {name} must be text")
    over_limit = {key: count for key, count in child_counts.items() if count > 100}
    if over_limit:
        errors.append(f"native subissue child limit exceeded: {over_limit}")

    ruleset = settings.get("ruleset", {})
    if ruleset.get("enforcement") != "active" or ruleset.get("include") != ["~DEFAULT_BRANCH"]:
        errors.append("main ruleset must be active and target the default branch")
    if not ruleset.get("required_status_checks"):
        errors.append("main ruleset must declare required status checks")
    if not settings.get("environments") or "github-project-write" not in settings.get("environments", []):
        errors.append("github-project-write environment is required")

    receipt = {
        "schema_version": "org.searchright.github-project-validation-receipt.v1",
        "status": "failed" if errors else "passed",
        "project_fields": len(fields),
        "project_views": len(views),
        "project_items": len(hierarchy.get("nodes", [])),
        "hierarchy_depth": 4,
        "maximum_children_observed": max(child_counts.values(), default=0),
        "node_field_counts": dict(sorted(node_fields.items())),
        "errors": errors,
        "limitations": [
            "Static validation only; no remote repository, Project, field, view or item is claimed."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
