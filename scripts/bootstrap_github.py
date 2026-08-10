#!/usr/bin/env python3
"""Create/wire the Searchright remote repository, issues and Project v2.

The command is dry-run-first and non-destructive. Apply requires --apply plus
SEARCHRIGHT_GITHUB_BOOTSTRAP_APPLY=1. It creates or verifies the remote, pushes
main, applies manifest-owned settings, creates protected environments, upserts a
main-branch ruleset, synchronises the native issue hierarchy, and creates and
populates the Project. It never deletes a repository, issue, Project or field.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any

from github_common import GitHubCommandError, ROOT, require_clean_tree, require_gh, run, run_json

SETTINGS_PATH = ROOT / "conductor/github/repository-settings.json"
PROJECT_PATH = ROOT / "conductor/github/project.json"
HIERARCHY_PATH = ROOT / "conductor/github/issue-hierarchy.json"


def github_remote_matches(url: str, repository: str) -> bool:
    normalised = url.strip().removesuffix(".git").rstrip("/")
    return normalised in {
        f"https://github.com/{repository}",
        f"git@github.com:{repository}",
        f"ssh://git@github.com/{repository}",
    }


def repository_exists(repository: str) -> bool:
    return run(["gh", "repo", "view", repository, "--json", "nameWithOwner"], allow_failure=True).returncode == 0


def ensure_remote(repository: str, visibility: str, description: str) -> dict[str, Any]:
    """Create or verify the remote without ever rewriting an unrelated origin."""
    remote = run(["git", "remote", "get-url", "origin"], allow_failure=True)
    if remote.returncode == 0 and not github_remote_matches(remote.stdout, repository):
        raise GitHubCommandError(
            f"origin points to {remote.stdout.strip()!r}, not {repository}; refusing to rewrite it"
        )
    existed = repository_exists(repository)
    if not existed:
        command = [
            "gh", "repo", "create", repository,
            "--description", description,
        ]
        command.append("--public" if visibility == "public" else "--private")
        run(command)
        action = "created"
    else:
        action = "verified"
    expected = f"https://github.com/{repository}.git"
    if remote.returncode != 0:
        run(["git", "remote", "add", "origin", expected])
    run(["git", "push", "--set-upstream", "origin", "main"])
    action = f"{action}_and_pushed"
    detail = run_json(["gh", "repo", "view", repository, "--json", "nameWithOwner,url,visibility,defaultBranchRef"])
    return {"action": action, "detail": detail}


def apply_repository_settings(settings: dict[str, Any]) -> dict[str, Any]:
    repository = settings["repository"]
    features = settings["features"]
    merge = settings["merge_policy"]
    payload = {
        "description": settings["description"],
        "homepage": settings["homepage"],
        "has_issues": features["issues"],
        "has_projects": features["projects"],
        "has_discussions": features["discussions"],
        "has_wiki": features["wiki"],
        "allow_squash_merge": merge["squash"],
        "allow_rebase_merge": merge["rebase"],
        "allow_merge_commit": merge["merge_commit"],
        "delete_branch_on_merge": merge["delete_head_branch"],
        "allow_auto_merge": merge["allow_auto_merge"],
        "allow_update_branch": merge["allow_update_branch"],
    }
    run_json(["gh", "api", "-X", "PATCH", f"repos/{repository}", "--input", "-"], input_text=json.dumps(payload))
    run_json(["gh", "api", "-X", "PUT", f"repos/{repository}/topics", "--input", "-"], input_text=json.dumps({"names": settings["topics"]}))
    outcomes: dict[str, str] = {}
    security_endpoints = {
        "vulnerability_alerts": ("PUT", f"repos/{repository}/vulnerability-alerts"),
        "automated_security_fixes": ("PUT", f"repos/{repository}/automated-security-fixes"),
        "private_vulnerability_reporting": ("PUT", f"repos/{repository}/private-vulnerability-reporting"),
    }
    for name, (method, endpoint) in security_endpoints.items():
        process = run(["gh", "api", "-X", method, endpoint], allow_failure=True)
        outcomes[name] = "applied" if process.returncode == 0 else "unsupported_or_permission_blocked"
    analysis_payload = {
        "security_and_analysis": {
            "secret_scanning": {"status": "enabled"},
            "secret_scanning_push_protection": {"status": "enabled"},
        }
    }
    process = run([
        "gh", "api", "-X", "PATCH", f"repos/{repository}", "--input", "-"
    ], input_text=json.dumps(analysis_payload), allow_failure=True)
    outcomes["secret_scanning"] = "applied" if process.returncode == 0 else "unsupported_or_permission_blocked"
    return outcomes


def ensure_environments(repository: str, names: list[str]) -> list[str]:
    observed: list[str] = []
    for name in names:
        run_json(["gh", "api", "-X", "PUT", f"repos/{repository}/environments/{name}", "--input", "-"], input_text="{}")
        observed.append(name)
    return observed


def ruleset_payload(settings: dict[str, Any]) -> dict[str, Any]:
    declared = settings["ruleset"]
    rules: list[dict[str, Any]] = []
    if declared.get("deletion") is False:
        rules.append({"type": "deletion"})
    if declared.get("non_fast_forward") is False:
        rules.append({"type": "non_fast_forward"})
    if declared.get("required_linear_history"):
        rules.append({"type": "required_linear_history"})
    checks = declared.get("required_status_checks", [])
    if checks:
        rules.append({
            "type": "required_status_checks",
            "parameters": {
                "required_status_checks": [{"context": context} for context in checks],
                "strict_required_status_checks_policy": true_value(),
                "do_not_enforce_on_create": true_value(),
            },
        })
    if declared.get("required_signed_commits"):
        rules.append({"type": "required_signatures"})
    return {
        "name": declared["name"],
        "target": declared["target"],
        "enforcement": declared["enforcement"],
        "bypass_actors": [],
        "conditions": {"ref_name": {"include": declared["include"], "exclude": []}},
        "rules": rules,
    }


def true_value() -> bool:
    """Keep JSON booleans obvious in the ruleset construction."""
    return True


def ensure_ruleset(settings: dict[str, Any]) -> dict[str, Any]:
    repository = settings["repository"]
    payload = ruleset_payload(settings)
    current = run_json(["gh", "api", f"repos/{repository}/rulesets?includes_parents=false"])
    matches = [item for item in current if isinstance(item, dict) and item.get("name") == payload["name"]] if isinstance(current, list) else []
    if len(matches) > 1:
        raise GitHubCommandError(f"multiple rulesets named {payload['name']!r}; manual reconciliation required")
    if matches:
        ruleset_id = matches[0]["id"]
        result = run_json(["gh", "api", "-X", "PUT", f"repos/{repository}/rulesets/{ruleset_id}", "--input", "-"], input_text=json.dumps(payload))
        action = "updated"
    else:
        result = run_json(["gh", "api", "-X", "POST", f"repos/{repository}/rulesets", "--input", "-"], input_text=json.dumps(payload))
        action = "created"
    return {"action": action, "id": result.get("id"), "name": result.get("name")}


def set_repository_variables(repository: str, project_title: str) -> list[str]:
    variables = {
        "SEARCHRIGHT_CANONICAL_REPOSITORY": repository,
        "SEARCHRIGHT_PROJECT_TITLE": project_title,
    }
    for name, value in variables.items():
        run(["gh", "variable", "set", name, "--repo", repository, "--body", value])
    return sorted(variables)


def maybe_set_project_secret(repository: str) -> str:
    value = os.environ.get("SEARCHRIGHT_PROJECT_TOKEN_VALUE")
    if not value:
        return "manual_secret_required"
    run([
        "gh", "secret", "set", "SEARCHRIGHT_PROJECT_TOKEN", "--repo", repository,
    ], input_text=value)
    return "configured"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--create-project", action="store_true")
    parser.add_argument("--sync-task-state", action="store_true")
    parser.add_argument("--receipt-path", type=Path)
    args = parser.parse_args()

    settings = json.loads(SETTINGS_PATH.read_text(encoding="utf-8"))
    project = json.loads(PROJECT_PATH.read_text(encoding="utf-8"))
    hierarchy = json.loads(HIERARCHY_PATH.read_text(encoding="utf-8"))
    apply = args.apply and os.environ.get("SEARCHRIGHT_GITHUB_BOOTSTRAP_APPLY") == "1"
    plan = {
        "schema_version": "org.searchright.github-bootstrap-plan.v1",
        "mode": "apply" if apply else "dry_run",
        "repository": settings["repository"],
        "project": project["title"],
        "issues": len(hierarchy["nodes"]),
        "environments": settings["environments"],
        "create_project": bool(args.create_project),
        "sync_task_state": bool(args.sync_task_state),
        "operations": [
            "create_or_verify_remote",
            "push_main",
            "apply_repository_settings",
            "enable_security_controls_where_supported",
            "create_protected_environments",
            "upsert_main_ruleset",
            "synchronise_issue_hierarchy",
            "create_and_synchronise_project",
        ],
        "delete_operations": 0,
    }
    if not apply:
        print(json.dumps(plan, indent=2, sort_keys=True))
        return 0

    if settings["repository"] != project["repository"] or project["repository"] != hierarchy["repository"]:
        raise GitHubCommandError("repository differs across control-plane manifests")
    require_clean_tree()
    require_gh()
    authenticated_login = run(["gh", "api", "user", "--jq", ".login"]).stdout.strip()
    if project.get("owner_type") == "user" and authenticated_login.lower() != str(project["owner"]).lower():
        raise GitHubCommandError(
            f"authenticated GitHub user {authenticated_login!r} does not own the declared user Project {project['owner']!r}"
        )
    project_scope = run([
        "gh", "project", "list", "--owner", project["owner"], "--limit", "1", "--format", "json",
    ], allow_failure=True)
    if project_scope.returncode != 0:
        raise GitHubCommandError(
            "GitHub Project access failed; refresh the token with the project scope before applying the control plane"
        )
    branch = run(["git", "branch", "--show-current"]).stdout.strip()
    if branch != "main":
        raise GitHubCommandError(f"bootstrap requires main branch, found {branch!r}")
    remote = ensure_remote(settings["repository"], settings["visibility"], settings["description"])
    security = apply_repository_settings(settings)
    environments = ensure_environments(settings["repository"], settings["environments"])
    ruleset = ensure_ruleset(settings)
    variables = set_repository_variables(settings["repository"], project["title"])
    secret_status = maybe_set_project_secret(settings["repository"])

    issue_env = {"SEARCHRIGHT_GITHUB_APPLY": "1"}
    local_receipts = ROOT / ".searchright" / "receipts"
    local_receipts.mkdir(parents=True, exist_ok=True)
    issue_command = [
        sys.executable, "scripts/sync_github_issues.py", "--repo", settings["repository"], "--apply",
        "--checkpoint-path", str(local_receipts / "github-issue-sync-checkpoint.json"),
        "--receipt-path", str(local_receipts / "github-issue-sync.json"),
    ]
    if args.sync_task_state:
        issue_command.append("--sync-task-state")
        issue_env["SEARCHRIGHT_GITHUB_TASK_STATE_APPLY"] = "1"
    issue_process = run(issue_command, env=issue_env)
    issue_receipt = json.loads(issue_process.stdout)

    project_process = run([
        sys.executable, "scripts/sync_github_project.py", "--apply",
        "--checkpoint-path", str(local_receipts / "github-project-sync-checkpoint.json"),
        "--receipt-path", str(local_receipts / "github-project-sync.json"),
        *( ["--create-project"] if args.create_project else [] ),
    ], env={"SEARCHRIGHT_GITHUB_PROJECT_APPLY": "1"})
    project_receipt = json.loads(project_process.stdout)

    receipt = {
        "schema_version": "org.searchright.github-bootstrap-receipt.v1",
        "mode": "apply",
        "repository": settings["repository"],
        "remote": remote,
        "security_controls": security,
        "environments": environments,
        "ruleset": ruleset,
        "variables": variables,
        "project_token_secret": secret_status,
        "issue_sync": {
            "issues": len(issue_receipt.get("issues", [])),
            "relationships": len(issue_receipt.get("relationships", [])),
            "task_state_apply": issue_receipt.get("task_state_apply"),
        },
        "project_sync": {
            "project": project_receipt.get("project"),
            "items": len(project_receipt.get("items", [])),
            "fields": len(project_receipt.get("fields_observed", [])),
            "views": len(project_receipt.get("views_observed", [])),
        },
        "local_ignored_receipt_directory": str(local_receipts.relative_to(ROOT)),
        "delete_operations": 0,
        "claim_boundary": "This receipt records observed GitHub control-plane mutations, not product maturity or higher evidence.",
    }
    text = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    print(text, end="")
    if args.receipt_path:
        path = args.receipt_path if args.receipt_path.is_absolute() else ROOT / args.receipt_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GitHubCommandError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
