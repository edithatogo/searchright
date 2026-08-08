#!/usr/bin/env python3
"""Idempotently synchronise generated issues and native nested subissues.

Conductor remains canonical. Dry-run is unconditional unless both ``--apply``
and ``SEARCHRIGHT_GITHUB_APPLY=1`` are present. Task open/closed state is
synchronised only when ``--sync-task-state`` and
``SEARCHRIGHT_GITHUB_TASK_STATE_APPLY=1`` are also present. The script never
deletes issues, labels, or hierarchy nodes.

Large first-run projections can be resumed with ``--resume-after`` and bounded
with ``--max-nodes``. Existing issue content, labels and relationships are read
before mutation, so a repeated run is convergent rather than write-amplifying.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

from github_common import (
    GitHubCommandError,
    ROOT,
    require_clean_tree,
    require_gh,
    run,
    run_json,
    select_after,
    write_json_atomic,
)

HIERARCHY = ROOT / "conductor/github/issue-hierarchy.json"
LABELS = ROOT / "conductor/github/labels.json"


def flatten_pages(payload: Any) -> list[dict[str, Any]]:
    if not isinstance(payload, list):
        return []
    if payload and isinstance(payload[0], list):
        return [item for page in payload for item in page if isinstance(item, dict)]
    return [item for item in payload if isinstance(item, dict)]


def issue_key(body: str) -> str | None:
    for line in body.splitlines():
        if line.startswith("<!-- searchright-issue-key: ") and line.endswith(" -->"):
            return line.removeprefix("<!-- searchright-issue-key: ").removesuffix(" -->")
    return None


def existing_issues(repository: str) -> dict[str, dict[str, Any]]:
    payload = run_json([
        "gh", "api", "--paginate", "--slurp",
        f"repos/{repository}/issues?state=all&per_page=100",
    ])
    result: dict[str, dict[str, Any]] = {}
    duplicates: set[str] = set()
    for issue in flatten_pages(payload):
        if issue.get("pull_request"):
            continue
        key = issue_key(str(issue.get("body") or ""))
        if not key:
            continue
        if key in result:
            duplicates.add(key)
        result[key] = issue
    if duplicates:
        raise GitHubCommandError(
            f"multiple remote issues contain canonical keys {sorted(duplicates)}; manual reconciliation required"
        )
    return result


def ensure_labels(repository: str, labels: list[dict[str, Any]]) -> None:
    for label in labels:
        run([
            "gh", "label", "create", label["name"], "--repo", repository,
            "--color", label["color"], "--description", label["description"], "--force",
        ])


def create_issue(repository: str, node: dict[str, Any]) -> dict[str, Any]:
    body = (ROOT / node["body_path"]).read_text(encoding="utf-8")
    payload = {"title": node["title"], "body": body, "labels": node["labels"]}
    result = run_json([
        "gh", "api", "-X", "POST", f"repos/{repository}/issues", "--input", "-",
    ], input_text=json.dumps(payload))
    if not isinstance(result, dict) or not result.get("number") or not result.get("id"):
        raise GitHubCommandError(f"issue creation for {node['key']} returned no issue identity")
    return result


def remote_label_names(issue: dict[str, Any]) -> set[str]:
    values: set[str] = set()
    for label in issue.get("labels", []):
        if isinstance(label, dict) and label.get("name"):
            values.add(str(label["name"]))
        elif isinstance(label, str):
            values.add(label)
    return values


def update_issue_if_needed(
    repository: str,
    issue: dict[str, Any],
    node: dict[str, Any],
) -> tuple[dict[str, Any], str, list[str]]:
    body = (ROOT / node["body_path"]).read_text(encoding="utf-8")
    content_changed = issue.get("title") != node["title"] or str(issue.get("body") or "") != body
    labels_changed = remote_label_names(issue) != set(node["labels"])
    changes: list[str] = []
    if content_changed:
        run_json([
            "gh", "api", "-X", "PATCH", f"repos/{repository}/issues/{issue['number']}", "--input", "-",
        ], input_text=json.dumps({"title": node["title"], "body": body}))
        changes.append("content")
    if labels_changed:
        run_json([
            "gh", "api", "-X", "PUT", f"repos/{repository}/issues/{issue['number']}/labels", "--input", "-",
        ], input_text=json.dumps({"labels": node["labels"]}))
        changes.append("labels")
    if not changes:
        return issue, "unchanged", []
    result = run_json(["gh", "api", f"repos/{repository}/issues/{issue['number']}"])
    if not isinstance(result, dict):
        raise GitHubCommandError(f"issue update for {node['key']} returned no object")
    return result, "updated", changes


def child_ids(repository: str, parent_number: int) -> set[int]:
    payload = run_json([
        "gh", "api", "--paginate", "--slurp",
        f"repos/{repository}/issues/{parent_number}/sub_issues?per_page=100",
    ])
    return {int(item["id"]) for item in flatten_pages(payload) if item.get("id") is not None}


def attach_child(
    repository: str,
    parent: dict[str, Any],
    child: dict[str, Any],
    cache: dict[int, set[int]],
) -> str:
    parent_number = int(parent["number"])
    current = cache.setdefault(parent_number, child_ids(repository, parent_number))
    child_id = int(child["id"])
    if child_id in current:
        return "existing"
    payload = run_json([
        "gh", "api", "-X", "POST",
        f"repos/{repository}/issues/{parent_number}/sub_issues",
        "--input", "-",
    ], input_text=json.dumps({"sub_issue_id": child_id}))
    if not isinstance(payload, dict):
        raise GitHubCommandError(
            f"native subissue attachment {parent_number} -> {child['number']} returned no object"
        )
    current.add(child_id)
    return "attached"


def sync_task_state(repository: str, issue: dict[str, Any], desired: str) -> str:
    actual = str(issue.get("state") or "").lower()
    if actual == desired:
        return "unchanged"
    run_json([
        "gh", "api", "-X", "PATCH", f"repos/{repository}/issues/{issue['number']}", "--input", "-",
    ], input_text=json.dumps({"state": desired}))
    return f"{actual or 'unknown'}->{desired}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default="edithatogo/searchright")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--sync-task-state", action="store_true")
    parser.add_argument("--resume-after", help="Continue after this canonical issue key")
    parser.add_argument("--max-nodes", type=int, help="Bound this run to a canonical node count")
    parser.add_argument("--verbose-plan", action="store_true")
    parser.add_argument("--checkpoint-path", type=Path)
    parser.add_argument("--receipt-path", type=Path)
    args = parser.parse_args()

    data = json.loads(HIERARCHY.read_text(encoding="utf-8"))
    nodes = data["nodes"]
    selected = select_after(
        nodes,
        key_name="key",
        resume_after=args.resume_after,
        maximum=args.max_nodes,
    )
    selected_keys = {node["key"] for node in selected}
    final_selected_key = selected[-1]["key"] if selected else args.resume_after
    final_index = next(
        (index for index, node in enumerate(nodes) if node["key"] == final_selected_key),
        -1,
    )
    remaining = max(0, len(nodes) - final_index - 1)
    apply = args.apply and os.environ.get("SEARCHRIGHT_GITHUB_APPLY") == "1"
    state_apply = (
        apply
        and args.sync_task_state
        and os.environ.get("SEARCHRIGHT_GITHUB_TASK_STATE_APPLY") == "1"
    )
    plan: dict[str, Any] = {
        "schema_version": "org.searchright.github-issue-sync-plan.v2",
        "repository": args.repo,
        "mode": "apply" if apply else "dry_run",
        "canonical_issues": len(nodes),
        "selected_issues": len(selected),
        "selected_relationships": sum(1 for node in selected if node.get("parent_key")),
        "selected_task_states": sum(1 for node in selected if node["kind"] == "task"),
        "task_state_apply": state_apply,
        "resume_after": args.resume_after,
        "next_resume_after": final_selected_key if remaining else None,
        "remaining_after_run": remaining,
        "delete_operations": 0,
    }
    if args.verbose_plan:
        plan["operations"] = [
            {
                "action": "upsert_issue",
                "key": node["key"],
                "kind": node["kind"],
                "parent_key": node["parent_key"],
                "desired_state": node["desired_state"] if node["kind"] == "task" else "open",
            }
            for node in selected
        ]
    if not apply:
        print(json.dumps(plan, indent=2, sort_keys=True))
        return 0

    if args.repo != data["repository"]:
        raise GitHubCommandError("apply repository must match the generated hierarchy")
    require_clean_tree()
    require_gh()
    run(["gh", "repo", "view", args.repo, "--json", "nameWithOwner"])
    ensure_labels(args.repo, json.loads(LABELS.read_text(encoding="utf-8"))["labels"])

    existing = existing_issues(args.repo)
    by_key: dict[str, dict[str, Any]] = dict(existing)
    remote: list[dict[str, Any]] = []
    checkpoint_path = None
    if args.checkpoint_path:
        checkpoint_path = args.checkpoint_path if args.checkpoint_path.is_absolute() else ROOT / args.checkpoint_path
    for node in selected:
        previous = existing.get(node["key"])
        changes: list[str] = []
        if previous is None:
            issue = create_issue(args.repo, node)
            action = "created"
            changes = ["content", "labels"]
        else:
            issue, action, changes = update_issue_if_needed(args.repo, previous, node)
        state_action = "not_requested"
        if node["kind"] == "task" and state_apply:
            state_action = sync_task_state(args.repo, issue, node["desired_state"])
            if state_action != "unchanged":
                issue = run_json(["gh", "api", f"repos/{args.repo}/issues/{issue['number']}"])
        by_key[node["key"]] = issue
        remote.append({
            "key": node["key"],
            "kind": node["kind"],
            "number": issue["number"],
            "id": issue["id"],
            "action": action,
            "changes": changes,
            "state_action": state_action,
            "state": issue.get("state"),
            "url": issue["html_url"],
        })
        if checkpoint_path:
            write_json_atomic(checkpoint_path, {
                "schema_version": "org.searchright.github-issue-sync-checkpoint.v1",
                "stage": "issues",
                "repository": args.repo,
                "last_issue_key": node["key"],
                "issues_completed_in_run": len(remote),
                "selected_issues": len(selected),
                "safe_recovery": "rerun the same command; unchanged issues are not rewritten",
            })

    relationships: list[dict[str, Any]] = []
    child_cache: dict[int, set[int]] = {}
    for node in selected:
        if not node["parent_key"]:
            continue
        parent = by_key.get(node["parent_key"])
        child = by_key.get(node["key"])
        if parent is None or child is None:
            raise GitHubCommandError(
                f"cannot attach {node['key']}: parent or child is absent remotely; resume from an earlier key"
            )
        action = attach_child(args.repo, parent, child, child_cache)
        relationships.append({
            "parent_key": node["parent_key"],
            "child_key": node["key"],
            "parent_number": parent["number"],
            "child_number": child["number"],
            "action": action,
        })
        if checkpoint_path:
            write_json_atomic(checkpoint_path, {
                "schema_version": "org.searchright.github-issue-sync-checkpoint.v1",
                "stage": "relationships",
                "repository": args.repo,
                "last_relationship_child_key": node["key"],
                "issues_completed_in_run": len(remote),
                "relationships_completed_in_run": len(relationships),
                "safe_recovery": "rerun the same command; existing native subissues are detected",
            })

    receipt = {
        "schema_version": "org.searchright.github-issue-sync-receipt.v2",
        "repository": args.repo,
        "mode": "apply",
        "canonical_issues": len(nodes),
        "selected_issues": len(selected),
        "selected_keys": sorted(selected_keys),
        "task_state_apply": state_apply,
        "resume_after": args.resume_after,
        "next_resume_after": final_selected_key if remaining else None,
        "remaining_after_run": remaining,
        "issues": remote,
        "relationships": relationships,
        "delete_operations": 0,
        "claim_boundary": "Remote issue state was observed during this explicit run; Conductor and evidence receipts remain canonical.",
    }
    text = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    print(text, end="")
    if args.receipt_path:
        path = args.receipt_path if args.receipt_path.is_absolute() else ROOT / args.receipt_path
        write_json_atomic(path, receipt)
    if checkpoint_path:
        write_json_atomic(checkpoint_path, {
            "schema_version": "org.searchright.github-issue-sync-checkpoint.v1",
            "stage": "complete",
            "repository": args.repo,
            "issues_completed_in_run": len(remote),
            "relationships_completed_in_run": len(relationships),
            "remaining_after_run": remaining,
            "next_resume_after": final_selected_key if remaining else None,
        })
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GitHubCommandError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
