#!/usr/bin/env python3
"""Create and synchronise the Searchright GitHub Project v2.

Dry-run is the unconditional default. Apply requires ``--apply`` and
``SEARCHRIGHT_GITHUB_PROJECT_APPLY=1``. The synchroniser is additive and
convergent: it never deletes fields, views, items or projects, and remote state
cannot promote Conductor evidence.

Large first-run projections can be resumed with ``--resume-after`` and bounded
with ``--max-items``. Existing Project values are compared when the GitHub CLI
returns them; known-equal values are not rewritten.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import sys
from pathlib import Path
from typing import Any

from github_common import (
    ROOT,
    GitHubCommandError,
    repository_owner,
    require_clean_tree,
    require_gh,
    run,
    run_json,
    select_after,
    write_json_atomic,
)

PROJECT_PATH = ROOT / "conductor/github/project.json"
HIERARCHY_PATH = ROOT / "conductor/github/issue-hierarchy.json"


def collection(value: Any, key: str) -> list[dict[str, Any]]:
    if isinstance(value, dict) and isinstance(value.get(key), list):
        return [item for item in value[key] if isinstance(item, dict)]
    if isinstance(value, list):
        return [item for item in value if isinstance(item, dict)]
    return []


def find_project(owner: str, title: str) -> dict[str, Any] | None:
    payload = run_json(["gh", "project", "list", "--owner", owner, "--limit", "100", "--format", "json"])
    matches = [project for project in collection(payload, "projects") if project.get("title") == title]
    if len(matches) > 1:
        raise GitHubCommandError(f"multiple Projects named {title!r}; manual reconciliation required")
    return matches[0] if matches else None


def ensure_project(manifest: dict[str, Any], create: bool) -> dict[str, Any]:
    owner = manifest["owner"]
    project = find_project(owner, manifest["title"])
    if project is None:
        if not create:
            raise GitHubCommandError("Project does not exist; rerun with --create-project")
        run([
            "gh", "project", "create", "--owner", owner,
            "--title", manifest["title"],
            "--format", "json",
        ])
        project = find_project(owner, manifest["title"])
    if project is None:
        raise GitHubCommandError("Project creation could not be observed")
    number = str(project.get("number") or "")
    if not number:
        raise GitHubCommandError("Project number was missing from GitHub response")
    visibility = "PUBLIC" if manifest["visibility"] == "public" else "PRIVATE"
    readme = (ROOT / manifest["readme_path"]).read_text(encoding="utf-8")
    run([
        "gh", "project", "edit", number, "--owner", owner,
        "--title", manifest["title"],
        "--description", manifest["short_description"],
        "--readme", readme,
        "--visibility", visibility,
    ])
    if manifest.get("link_repository"):
        run([
            "gh", "project", "link", number, "--owner", owner,
            "--repo", manifest["repository"],
        ], allow_failure=True)
    detail = run_json(["gh", "project", "view", number, "--owner", owner, "--format", "json"])
    if not isinstance(detail, dict):
        raise GitHubCommandError("Project view did not return an object")
    return detail


def list_fields(project_id: str) -> dict[str, dict[str, Any]]:
    query = """query($id: ID!) { node(id: $id) { ... on ProjectV2 { fields(first: 100) { nodes { __typename ... on ProjectV2Field { id name dataType } ... on ProjectV2SingleSelectField { id name options { id name } } } } } } }"""
    payload = run_json([
        "gh", "api", "graphql", "-f", f"query={query}", "-F", f"id={project_id}",
    ])
    fields = (((payload or {}).get("data") or {}).get("node") or {}).get("fields", {}).get("nodes", [])
    return {item.get("name"): item for item in fields if isinstance(item, dict) and item.get("name")}


def field_data_type(field: dict[str, Any]) -> str:
    """Normalize GitHub's GraphQL field union to manifest/CLI data types."""
    typename = str(field.get("__typename") or "").upper()
    if typename == "PROJECTV2SINGLESELECTFIELD":
        return "SINGLE_SELECT"
    return str(field.get("dataType") or field.get("type") or "").upper()


def validate_field(requested: dict[str, Any], current: dict[str, Any]) -> None:
    name = requested["name"]
    actual_type = field_data_type(current)
    if actual_type and actual_type != requested["data_type"]:
        raise GitHubCommandError(
            f"Project field {name!r} has type {actual_type}, expected {requested['data_type']}"
        )
    if requested["data_type"] == "SINGLE_SELECT":
        actual_options = {
            option.get("name")
            for option in current.get("options", [])
            if isinstance(option, dict)
        }
        missing = set(requested["options"]) - actual_options
        if missing:
            raise GitHubCommandError(
                f"Project field {name!r} lacks options {sorted(missing)}; refusing destructive recreation"
            )


def ensure_fields(owner: str, number: str, project_id: str, manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    existing = list_fields(project_id)
    for requested in manifest["fields"]:
        if requested["name"] in existing:
            continue
        command = [
            "gh", "project", "field-create", number, "--owner", owner,
            "--name", requested["name"], "--data-type", requested["data_type"], "--format", "json",
        ]
        if requested["data_type"] == "SINGLE_SELECT":
            command.extend(["--single-select-options", ",".join(requested["options"])])
        run(command)
    observed = list_fields(project_id)
    for requested in manifest["fields"]:
        current = observed.get(requested["name"])
        if current is None:
            raise GitHubCommandError(f"Project field {requested['name']!r} could not be observed")
        validate_field(requested, current)
    return observed


def query_views(project_id: str) -> list[dict[str, Any]]:
    query = """query($id: ID!) { node(id: $id) { ... on ProjectV2 { views(first: 100) { nodes { id name layout filter } } } } }"""
    payload = run_json(["gh", "api", "graphql", "-f", f"query={query}", "-F", f"id={project_id}"])
    return (((payload or {}).get("data") or {}).get("node") or {}).get("views", {}).get("nodes", [])


def ensure_views(project_id: str, requested_views: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Add or update manifest-owned Project views without deleting remote views."""
    existing = {view.get("name"): view for view in query_views(project_id) if isinstance(view, dict)}
    create_mutation = """
      mutation($projectId: ID!, $name: String!, $layout: ProjectV2ViewLayout!) {
        createProjectV2View(input: {projectId: $projectId, name: $name, layout: $layout}) {
          projectV2View { id name layout filter }
        }
      }
    """
    update_mutation = """
      mutation($viewId: ID!, $name: String!, $layout: ProjectV2ViewLayout!, $filter: String!) {
        updateProjectV2View(input: {viewId: $viewId, name: $name, layout: $layout, filter: $filter}) {
          projectV2View { id name layout filter }
        }
      }
    """
    for requested in requested_views:
        current = existing.get(requested["name"])
        if current is None:
            payload = run_json([
                "gh", "api", "graphql",
                "-f", f"query={create_mutation}",
                "-F", f"projectId={project_id}",
                "-f", f"name={requested['name']}",
                "-f", f"layout={requested['layout']}",
            ])
            current = ((((payload or {}).get("data") or {}).get("createProjectV2View") or {}).get("projectV2View"))
            if not isinstance(current, dict):
                raise GitHubCommandError(f"could not create Project view {requested['name']!r}")
            existing[requested["name"]] = current
        desired_filter = requested.get("filter", "")
        if current.get("layout") != requested["layout"] or (current.get("filter") or "") != desired_filter:
            payload = run_json([
                "gh", "api", "graphql",
                "-f", f"query={update_mutation}",
                "-F", f"viewId={current['id']}",
                "-f", f"name={requested['name']}",
                "-f", f"layout={requested['layout']}",
                "-f", f"filter={desired_filter}",
            ])
            updated = ((((payload or {}).get("data") or {}).get("updateProjectV2View") or {}).get("projectV2View"))
            if not isinstance(updated, dict):
                raise GitHubCommandError(f"could not update Project view {requested['name']!r}")
            existing[requested["name"]] = updated
    return query_views(project_id)


def existing_issues(repository: str) -> dict[str, dict[str, Any]]:
    payload = run_json([
        "gh", "api", "--paginate", "--slurp",
        f"repos/{repository}/issues?state=all&per_page=100",
    ])
    pages = payload if isinstance(payload, list) else []
    if pages and isinstance(pages[0], list):
        items = [item for page in pages for item in page if isinstance(item, dict)]
    else:
        items = [item for item in pages if isinstance(item, dict)]
    result: dict[str, dict[str, Any]] = {}
    duplicates: set[str] = set()
    for issue in items:
        if issue.get("pull_request"):
            continue
        body = issue.get("body") or ""
        for line in body.splitlines():
            if line.startswith("<!-- searchright-issue-key: ") and line.endswith(" -->"):
                key = line.removeprefix("<!-- searchright-issue-key: ").removesuffix(" -->")
                if key in result:
                    duplicates.add(key)
                result[key] = issue
                break
    if duplicates:
        raise GitHubCommandError(
            f"multiple remote issues contain canonical keys {sorted(duplicates)}; manual reconciliation required"
        )
    return result


def project_items(owner: str, number: str) -> dict[str, dict[str, Any]]:
    payload = run_json([
        "gh", "project", "item-list", number, "--owner", owner,
        "--limit", "1000", "--format", "json",
    ])
    result: dict[str, dict[str, Any]] = {}
    duplicates: set[str] = set()
    for item in collection(payload, "items"):
        content = item.get("content") if isinstance(item.get("content"), dict) else {}
        url = content.get("url") or item.get("url")
        if not url:
            continue
        url = str(url)
        if url in result:
            duplicates.add(url)
        result[url] = item
    if duplicates:
        raise GitHubCommandError(
            f"multiple Project items point to the same issue URLs {sorted(duplicates)[:5]}"
        )
    return result


def option_id(field: dict[str, Any], value: str) -> str:
    for option in field.get("options", []):
        if isinstance(option, dict) and option.get("name") == value and option.get("id"):
            return str(option["id"])
    raise GitHubCommandError(f"Project field {field.get('name')!r} has no option {value!r}")


def field_value_literal(field: dict[str, Any], value: Any) -> str:
    data_type = field_data_type(field)
    if data_type == "SINGLE_SELECT":
        return f"singleSelectOptionId: {json.dumps(option_id(field, str(value)))}"
    if data_type == "NUMBER":
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise GitHubCommandError(f"Project field {field.get('name')!r} requires a number")
        return f"number: {json.dumps(value)}"
    if data_type == "DATE":
        return f"date: {json.dumps(str(value))}"
    return f"text: {json.dumps(str(value))}"


def set_fields(project_id: str, item_id: str, changes: list[tuple[dict[str, Any], Any]]) -> None:
    """Update one item's changed fields in a single atomic GraphQL request."""
    if not changes:
        return
    mutations = []
    for index, (field, value) in enumerate(changes):
        mutations.append(
            f"f{index}: updateProjectV2ItemFieldValue(input: {{"
            f"projectId: {json.dumps(project_id)}, itemId: {json.dumps(item_id)}, "
            f"fieldId: {json.dumps(str(field['id']))}, value: {{{field_value_literal(field, value)}}}"
            "}) { projectV2Item { id } }"
        )
    query = "mutation { " + " ".join(mutations) + " }"
    run_json(["gh", "api", "graphql", "-f", f"query={query}"])


def key_variants(name: str) -> list[str]:
    words = [word for word in re.split(r"[^A-Za-z0-9]+", name) if word]
    camel = words[0].lower() + "".join(word[:1].upper() + word[1:] for word in words[1:]) if words else name
    variants = [
        name,
        name.lower(),
        camel,
        "_".join(word.lower() for word in words),
        "-".join(word.lower() for word in words),
    ]
    return list(dict.fromkeys(variants))


def normalise_value(value: Any) -> Any:
    if isinstance(value, dict):
        for key in ("name", "text", "number", "date", "value"):
            if key in value:
                return normalise_value(value[key])
        return value
    if isinstance(value, float) and value.is_integer():
        return int(value)
    return value


def observed_field_value(item: dict[str, Any], field_name: str) -> tuple[bool, Any]:
    """Read one dynamic Project field when the CLI exposes it.

    ``gh project item-list --format json`` uses dynamic field keys. Different
    CLI releases have represented them as exact names, normalised keys, a
    ``fields`` object or a field-value list. This reader recognises those
    documented/evolved shapes. Unknown shapes return ``(False, None)`` so the
    synchroniser writes rather than incorrectly assuming equality.
    """
    item_keys = {str(key).casefold(): key for key in item}
    for key in key_variants(field_name):
        observed_key = item_keys.get(key.casefold())
        if observed_key is not None:
            return True, normalise_value(item[observed_key])
    fields = item.get("fields")
    if isinstance(fields, dict):
        field_keys = {str(key).casefold(): key for key in fields}
        for key in key_variants(field_name):
            observed_key = field_keys.get(key.casefold())
            if observed_key is not None:
                return True, normalise_value(fields[observed_key])
    for list_key in ("fieldValues", "field_values", "field-values"):
        values = item.get(list_key)
        if not isinstance(values, list):
            continue
        for entry in values:
            if not isinstance(entry, dict):
                continue
            field = entry.get("field") if isinstance(entry.get("field"), dict) else {}
            name = entry.get("fieldName") or entry.get("field_name") or field.get("name")
            if name == field_name:
                return True, normalise_value(entry)
    return False, None


def values_equal(observed: Any, desired: Any) -> bool:
    observed = normalise_value(observed)
    desired = normalise_value(desired)
    if isinstance(desired, int) and isinstance(observed, str):
        try:
            observed = int(observed)
        except ValueError:
            pass
    return observed == desired


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--create-project", action="store_true")
    parser.add_argument("--resume-after", help="Continue after this canonical issue key")
    parser.add_argument("--max-items", type=int, help="Bound this run to a canonical item count")
    parser.add_argument("--verbose-plan", action="store_true")
    parser.add_argument("--checkpoint-path", type=Path)
    parser.add_argument("--receipt-path", type=Path)
    args = parser.parse_args()

    manifest = json.loads(PROJECT_PATH.read_text(encoding="utf-8"))
    hierarchy = json.loads(HIERARCHY_PATH.read_text(encoding="utf-8"))
    nodes = hierarchy["nodes"]
    selected = select_after(
        nodes,
        key_name="key",
        resume_after=args.resume_after,
        maximum=args.max_items,
    )
    final_selected_key = selected[-1]["key"] if selected else args.resume_after
    final_index = next(
        (index for index, node in enumerate(nodes) if node["key"] == final_selected_key),
        -1,
    )
    remaining = max(0, len(nodes) - final_index - 1)
    apply = args.apply and os.environ.get("SEARCHRIGHT_GITHUB_PROJECT_APPLY") == "1"
    plan: dict[str, Any] = {
        "schema_version": "org.searchright.github-project-sync-plan.v1",
        "mode": "apply" if apply else "dry_run",
        "repository": manifest["repository"],
        "project": manifest["title"],
        "create_project": bool(args.create_project),
        "fields": len(manifest["fields"]),
        "views": len(manifest["views"]),
        "canonical_items": len(nodes),
        "selected_items": len(selected),
        "resume_after": args.resume_after,
        "next_resume_after": final_selected_key if remaining else None,
        "remaining_after_run": remaining,
        "delete_operations": 0,
        "claim_boundary": "A Project is a coordination projection and cannot promote Conductor evidence.",
    }
    if args.verbose_plan:
        plan["selected_keys"] = [node["key"] for node in selected]
    if not apply:
        print(json.dumps(plan, indent=2, sort_keys=True))
        return 0

    if manifest["repository"] != hierarchy["repository"]:
        raise GitHubCommandError("Project and issue hierarchy repository differ")
    if manifest["owner"] != repository_owner(manifest["repository"]):
        raise GitHubCommandError("Project owner differs from repository owner")
    require_clean_tree()
    require_gh()
    run(["gh", "repo", "view", manifest["repository"], "--json", "nameWithOwner"])

    detail = ensure_project(manifest, args.create_project)
    project_number = str(detail.get("number") or "")
    project_id = str(detail.get("id") or "")
    if not project_number or not project_id:
        raise GitHubCommandError("Project number/id missing after creation or lookup")
    fields = ensure_fields(manifest["owner"], project_number, project_id, manifest)
    views = ensure_views(project_id, manifest["views"])

    issues = existing_issues(manifest["repository"])
    missing = sorted({node["key"] for node in nodes} - set(issues))
    if missing:
        raise GitHubCommandError(
            f"Project sync requires issue hierarchy first; missing {len(missing)} issues, including {missing[:5]}"
        )
    items = project_items(manifest["owner"], project_number)
    synced: list[dict[str, Any]] = []
    checkpoint_path = None
    if args.checkpoint_path:
        checkpoint_path = args.checkpoint_path if args.checkpoint_path.is_absolute() else ROOT / args.checkpoint_path
    sync_date = dt.datetime.now(tz=dt.UTC).date().isoformat()
    field_updates = 0
    field_skips = 0
    for node in selected:
        issue = issues[node["key"]]
        url = str(issue["html_url"])
        item = items.get(url)
        action = "existing"
        if item is None:
            payload = run_json([
                "gh", "project", "item-add", project_number, "--owner", manifest["owner"],
                "--url", url, "--format", "json",
            ])
            item = payload if isinstance(payload, dict) else None
            action = "added"
            if item is None or not item.get("id"):
                items = project_items(manifest["owner"], project_number)
                item = items.get(url)
        if item is None or not item.get("id"):
            raise GitHubCommandError(f"Project item for {node['key']} could not be observed")
        item_id = str(item["id"])
        desired_fields = dict(node["project_fields"])
        desired_fields["Last sync"] = sync_date
        changed_fields: list[str] = []
        skipped_fields: list[str] = []
        pending_changes: list[tuple[dict[str, Any], Any]] = []
        for field_name, value in desired_fields.items():
            if field_name not in fields:
                raise GitHubCommandError(f"Project field {field_name!r} is absent")
            known, observed = observed_field_value(item, field_name)
            if known and values_equal(observed, value):
                skipped_fields.append(field_name)
                field_skips += 1
                continue
            changed_fields.append(field_name)
            pending_changes.append((fields[field_name], value))
        set_fields(project_id, item_id, pending_changes)
        field_updates += len(pending_changes)
        synced.append({
            "key": node["key"],
            "issue_url": url,
            "item_id": item_id,
            "action": action,
            "changed_fields": changed_fields,
            "known_equal_fields": skipped_fields,
        })
        if checkpoint_path:
            write_json_atomic(checkpoint_path, {
                "schema_version": "org.searchright.github-project-sync-checkpoint.v1",
                "stage": "items",
                "repository": manifest["repository"],
                "project_number": int(project_number),
                "last_item_key": node["key"],
                "items_completed_in_run": len(synced),
                "selected_items": len(selected),
                "field_updates": field_updates,
                "known_equal_field_skips": field_skips,
                "safe_recovery": f"rerun with --resume-after {node['key']} or rerun the whole convergent sync",
            })

    receipt = {
        "schema_version": "org.searchright.github-project-sync-receipt.v1",
        "mode": "apply",
        "repository": manifest["repository"],
        "canonical_items": len(nodes),
        "selected_items": len(selected),
        "resume_after": args.resume_after,
        "next_resume_after": final_selected_key if remaining else None,
        "remaining_after_run": remaining,
        "project": {
            "id": project_id,
            "number": int(project_number),
            "url": detail.get("url"),
            "title": detail.get("title"),
        },
        "fields_observed": sorted(fields),
        "views_observed": sorted(
            view.get("name") for view in views if isinstance(view, dict) and view.get("name")
        ),
        "field_updates": field_updates,
        "known_equal_field_skips": field_skips,
        "items": synced,
        "delete_operations": 0,
        "claim_boundary": "Remote Project state was observed during this run; it does not promote Conductor evidence.",
    }
    text = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    print(text, end="")
    if args.receipt_path:
        path = args.receipt_path if args.receipt_path.is_absolute() else ROOT / args.receipt_path
        write_json_atomic(path, receipt)
    if checkpoint_path:
        write_json_atomic(checkpoint_path, {
            "schema_version": "org.searchright.github-project-sync-checkpoint.v1",
            "stage": "complete",
            "repository": manifest["repository"],
            "project_number": int(project_number),
            "items_completed_in_run": len(synced),
            "remaining_after_run": remaining,
            "next_resume_after": final_selected_key if remaining else None,
            "field_updates": field_updates,
            "known_equal_field_skips": field_skips,
        })
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GitHubCommandError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
