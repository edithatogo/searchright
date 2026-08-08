#!/usr/bin/env python3
"""Apply conservative security and reproducibility rules to GitHub workflows."""

from __future__ import annotations

import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"
CONSEQUENTIAL_WRITE_PERMISSIONS = {
    "actions",
    "checks",
    "contents",
    "deployments",
    "discussions",
    "issues",
    "packages",
    "pages",
    "pull-requests",
    "repository-projects",
    "statuses",
}


def has_write(permission_map: object) -> bool:
    if permission_map == "write-all":
        return True
    if not isinstance(permission_map, dict):
        return False
    return any(
        key in CONSEQUENTIAL_WRITE_PERMISSIONS and str(value).lower() == "write"
        for key, value in permission_map.items()
    )


def main() -> int:
    errors: list[str] = []
    jobs_checked = 0
    checkouts_checked = 0

    for path in sorted(WORKFLOWS.glob("*.yml")):
        text = path.read_text(encoding="utf-8")
        try:
            workflow = yaml.safe_load(text)
        except yaml.YAMLError as exc:
            errors.append(f"{path.name}: invalid YAML: {exc}")
            continue
        if not isinstance(workflow, dict):
            errors.append(f"{path.name}: workflow must be a mapping")
            continue
        if "pull_request_target" in text:
            errors.append(f"{path.name}: pull_request_target is prohibited")
        if workflow.get("permissions") == "write-all":
            errors.append(f"{path.name}: write-all permissions are prohibited")
        jobs = workflow.get("jobs")
        if not isinstance(jobs, dict) or not jobs:
            errors.append(f"{path.name}: workflow has no jobs")
            continue
        for job_name, job in jobs.items():
            jobs_checked += 1
            if not isinstance(job, dict):
                errors.append(f"{path.name}/{job_name}: job must be a mapping")
                continue
            if not isinstance(job.get("timeout-minutes"), int):
                errors.append(f"{path.name}/{job_name}: timeout-minutes is required")
            effective_permissions = job.get("permissions", workflow.get("permissions"))
            if has_write(effective_permissions):
                if not job.get("environment"):
                    errors.append(
                        f"{path.name}/{job_name}: write-capable job requires a protected environment"
                    )
                triggers = str(workflow.get(True, workflow.get("on", "")))
                condition = str(job.get("if", ""))
                if "pull_request" in triggers and "pull_request" not in condition:
                    errors.append(
                        f"{path.name}/{job_name}: write-capable job must be excluded from pull requests"
                    )
            steps = job.get("steps", [])
            if not isinstance(steps, list):
                errors.append(f"{path.name}/{job_name}: steps must be a list")
                continue
            for step in steps:
                if not isinstance(step, dict):
                    continue
                uses = str(step.get("uses", ""))
                if uses.startswith("actions/checkout@"):
                    checkouts_checked += 1
                    settings = step.get("with", {})
                    if not isinstance(settings, dict) or settings.get("persist-credentials") is not False:
                        errors.append(
                            f"{path.name}/{job_name}: checkout must set persist-credentials: false"
                        )
                run = step.get("run")
                if isinstance(run, str):
                    if re.search(r"\bcurl\b.*\|\s*(?:ba)?sh\b", run):
                        errors.append(f"{path.name}/{job_name}: curl-to-shell is prohibited")
                    if "cargo install " in run:
                        for line in run.splitlines():
                            if "cargo install " not in line:
                                continue
                            local_path_install = " --path " in line
                            if " --locked" not in line or (
                                not local_path_install and " --version " not in line
                            ):
                                errors.append(
                                    f"{path.name}/{job_name}: cargo install must be exact and locked"
                                )

    receipt = {
        "schema_version": "org.searchright.workflow-hardening-receipt.v1",
        "status": "failed" if errors else "passed",
        "workflows_checked": len(list(WORKFLOWS.glob("*.yml"))),
        "jobs_checked": jobs_checked,
        "checkouts_checked": checkouts_checked,
        "errors": errors,
        "limitations": [
            "Static workflow review only; GitHub environment protection, rulesets and runner state require remote evidence."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
