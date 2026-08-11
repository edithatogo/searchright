#!/usr/bin/env python3
"""Read-only audit of the observed Searchright GitHub control plane.

The audit compares repository settings, canonical issues, native subissues,
Project fields/views/items and recognised custom-field values with the checked-in
manifests. It never mutates GitHub and never promotes Conductor evidence on its
own.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

from github_common import GitHubCommandError, ROOT, require_gh, run_json, write_json_atomic
from sync_github_issues import child_ids, existing_issues, remote_label_names
from sync_github_project import (
    collection,
    field_data_type,
    find_project,
    list_fields,
    observed_field_value,
    project_items,
    query_views,
    values_equal,
)

HIERARCHY_PATH = ROOT / "conductor/github/issue-hierarchy.json"
PROJECT_PATH = ROOT / "conductor/github/project.json"
SETTINGS_PATH = ROOT / "conductor/github/repository-settings.json"


def compare_repository(settings: dict[str, Any], errors: list[str], warnings: list[str]) -> dict[str, Any]:
    repository = settings["repository"]
    observed = run_json(["gh", "api", f"repos/{repository}"])
    if not isinstance(observed, dict):
        raise GitHubCommandError("repository endpoint did not return an object")
    exact = {
        "full_name": repository,
        "visibility": settings["visibility"],
        "description": settings["description"],
        "homepage": settings["homepage"],
        "has_issues": settings["features"]["issues"],
        "has_projects": settings["features"]["projects"],
        "has_discussions": settings["features"]["discussions"],
        "has_wiki": settings["features"]["wiki"],
        "allow_squash_merge": settings["merge_policy"]["squash"],
        "allow_rebase_merge": settings["merge_policy"]["rebase"],
        "allow_merge_commit": settings["merge_policy"]["merge_commit"],
        "delete_branch_on_merge": settings["merge_policy"]["delete_head_branch"],
        "allow_auto_merge": settings["merge_policy"]["allow_auto_merge"],
        "allow_update_branch": settings["merge_policy"]["allow_update_branch"],
    }
    for key, desired in exact.items():
        if key in observed and observed.get(key) != desired:
            errors.append(f"repository setting {key}={observed.get(key)!r}, expected {desired!r}")
        elif key not in observed:
            warnings.append(f"repository endpoint omitted {key}")
    topics = run_json(["gh", "api", f"repos/{repository}/topics"])
    observed_topics = set(topics.get("names", [])) if isinstance(topics, dict) else set()
    if observed_topics != set(settings["topics"]):
        errors.append(
            f"repository topics differ: observed={sorted(observed_topics)}, expected={sorted(settings['topics'])}"
        )
    environments = run_json([
        "gh", "api", "--paginate", "--slurp",
        f"repos/{repository}/environments?per_page=100",
    ])
    pages = environments if isinstance(environments, list) else [environments]
    if len(pages) == 1 and isinstance(pages[0], list):
        pages = pages[0]
    observed_environments = {
        str(item.get("name"))
        for page in pages
        for item in (page.get("environments", []) if isinstance(page, dict) else [])
        if isinstance(item, dict) and item.get("name")
    }
    missing_env = set(settings["environments"]) - observed_environments
    if missing_env:
        errors.append(f"missing protected environments {sorted(missing_env)}")
    rulesets = run_json(["gh", "api", f"repos/{repository}/rulesets?includes_parents=false"])
    matches = [
        item for item in rulesets
        if isinstance(item, dict) and item.get("name") == settings["ruleset"]["name"]
    ] if isinstance(rulesets, list) else []
    if len(matches) != 1:
        errors.append(f"expected one ruleset named {settings['ruleset']['name']!r}, observed {len(matches)}")
    security = observed.get("security_and_analysis")
    if not isinstance(security, dict):
        warnings.append("repository endpoint did not expose security_and_analysis")
    return {
        "url": observed.get("html_url"),
        "default_branch": observed.get("default_branch"),
        "visibility": observed.get("visibility"),
        "topics": sorted(observed_topics),
        "environments": sorted(observed_environments),
        "ruleset_matches": len(matches),
    }


def compare_issues(
    hierarchy: dict[str, Any],
    errors: list[str],
) -> tuple[dict[str, dict[str, Any]], dict[str, Any]]:
    repository = hierarchy["repository"]
    nodes = hierarchy["nodes"]
    observed = existing_issues(repository)
    canonical_keys = {node["key"] for node in nodes}
    missing = sorted(canonical_keys - set(observed))
    if missing:
        errors.append(f"missing {len(missing)} canonical issues, including {missing[:5]}")
    content_drift = 0
    label_drift = 0
    state_drift = 0
    for node in nodes:
        issue = observed.get(node["key"])
        if issue is None:
            continue
        body = (ROOT / node["body_path"]).read_text(encoding="utf-8")
        if issue.get("title") != node["title"] or str(issue.get("body") or "") != body:
            content_drift += 1
        if remote_label_names(issue) != set(node["labels"]):
            label_drift += 1
        if node["kind"] == "task" and str(issue.get("state", "")).lower() != node["desired_state"]:
            state_drift += 1
    if content_drift:
        errors.append(f"{content_drift} canonical issues have content drift")
    if label_drift:
        errors.append(f"{label_drift} canonical issues have label drift")
    if state_drift:
        errors.append(f"{state_drift} canonical task issues have state drift")

    expected_children: dict[str, list[str]] = defaultdict(list)
    for node in nodes:
        if node.get("parent_key"):
            expected_children[str(node["parent_key"])].append(str(node["key"]))
    missing_relationships: list[tuple[str, str]] = []
    relationships_observed = 0
    for parent_key, child_keys in expected_children.items():
        parent = observed.get(parent_key)
        if parent is None:
            continue
        actual = child_ids(repository, int(parent["number"]))
        for child_key in child_keys:
            child = observed.get(child_key)
            if child is None:
                continue
            if int(child["id"]) not in actual:
                missing_relationships.append((parent_key, child_key))
            else:
                relationships_observed += 1
    if missing_relationships:
        errors.append(
            f"missing {len(missing_relationships)} native subissue relationships, including {missing_relationships[:5]}"
        )
    return observed, {
        "canonical": len(nodes),
        "observed": len(canonical_keys & set(observed)),
        "content_drift": content_drift,
        "label_drift": label_drift,
        "task_state_drift": state_drift,
        "relationships_expected": len(nodes) - 1,
        "relationships_observed": relationships_observed,
    }


def compare_project(
    manifest: dict[str, Any],
    hierarchy: dict[str, Any],
    issues: dict[str, dict[str, Any]],
    errors: list[str],
    warnings: list[str],
) -> dict[str, Any]:
    project = find_project(manifest["owner"], manifest["title"])
    if project is None:
        errors.append(f"Project {manifest['title']!r} is missing")
        return {"observed": False}
    number = str(project.get("number") or "")
    project_id = str(project.get("id") or "")
    if not number or not project_id:
        errors.append("observed Project lacks number or node ID")
        return {"observed": True, "number": number}
    fields = list_fields(project_id)
    missing_fields = sorted({field["name"] for field in manifest["fields"]} - set(fields))
    if missing_fields:
        errors.append(f"Project is missing fields {missing_fields}")
    for requested in manifest["fields"]:
        current = fields.get(requested["name"])
        if current is None:
            continue
        actual_type = field_data_type(current)
        if actual_type and actual_type != requested["data_type"]:
            errors.append(
                f"Project field {requested['name']!r} type {actual_type}, expected {requested['data_type']}"
            )
        if requested["data_type"] == "SINGLE_SELECT":
            options = {option.get("name") for option in current.get("options", []) if isinstance(option, dict)}
            missing_options = set(requested["options"]) - options
            if missing_options:
                errors.append(f"Project field {requested['name']!r} lacks options {sorted(missing_options)}")
    views = {view.get("name"): view for view in query_views(project_id) if isinstance(view, dict)}
    for requested in manifest["views"]:
        current = views.get(requested["name"])
        if current is None:
            errors.append(f"Project view {requested['name']!r} is missing")
            continue
        if current.get("layout") != requested["layout"] or (current.get("filter") or "") != requested.get("filter", ""):
            errors.append(f"Project view {requested['name']!r} differs from the manifest")
    items = project_items(manifest["owner"], number)
    missing_items = []
    mismatched_values = 0
    unknown_values = 0
    for node in hierarchy["nodes"]:
        issue = issues.get(node["key"])
        if issue is None:
            continue
        item = items.get(str(issue["html_url"]))
        if item is None:
            missing_items.append(node["key"])
            continue
        for field_name, desired in node["project_fields"].items():
            known, observed = observed_field_value(item, field_name)
            if not known:
                unknown_values += 1
            elif not values_equal(observed, desired):
                mismatched_values += 1
    if missing_items:
        errors.append(f"Project is missing {len(missing_items)} canonical items, including {missing_items[:5]}")
    if mismatched_values:
        errors.append(f"{mismatched_values} recognised Project custom-field values differ")
    if unknown_values:
        warnings.append(
            f"GitHub CLI JSON shape did not expose {unknown_values} custom-field values; item membership was still verified"
        )
    return {
        "observed": True,
        "id": project_id,
        "number": int(number),
        "url": project.get("url"),
        "fields_observed": len(fields),
        "views_observed": len(views),
        "canonical_items_observed": len(hierarchy["nodes"]) - len(missing_items),
        "recognised_value_mismatches": mismatched_values,
        "unrecognised_value_shapes": unknown_values,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--receipt-path", type=Path)
    args = parser.parse_args()
    require_gh()
    hierarchy = json.loads(HIERARCHY_PATH.read_text(encoding="utf-8"))
    project = json.loads(PROJECT_PATH.read_text(encoding="utf-8"))
    settings = json.loads(SETTINGS_PATH.read_text(encoding="utf-8"))
    errors: list[str] = []
    warnings: list[str] = []
    repository = compare_repository(settings, errors, warnings)
    issues, issue_summary = compare_issues(hierarchy, errors)
    project_summary = compare_project(project, hierarchy, issues, errors, warnings)
    receipt = {
        "schema_version": "org.searchright.github-control-plane-audit.v1",
        "status": "failed" if errors else "passed",
        "repository": settings["repository"],
        "repository_observed": repository,
        "issue_hierarchy": issue_summary,
        "project": project_summary,
        "errors": errors,
        "warnings": warnings,
        "mutation_operations": 0,
        "claim_boundary": "This read-only audit establishes observed GitHub parity only; it does not establish product maturity.",
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    if args.receipt_path:
        path = args.receipt_path if args.receipt_path.is_absolute() else ROOT / args.receipt_path
        write_json_atomic(path, receipt)
    return 1 if errors else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GitHubCommandError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
