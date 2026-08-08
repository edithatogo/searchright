#!/usr/bin/env python3
"""Check pinned integration revisions without ever changing them."""

from __future__ import annotations

import argparse
import json
import os
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOCKS = ROOT / "integration" / "locks.json"


def live_head(repository: str, branch: str) -> str:
    request = urllib.request.Request(
        f"https://api.github.com/repos/{repository}/branches/{branch}",
        headers={
            "Accept": "application/vnd.github+json",
            "User-Agent": "searchright-integration-drift/0.1",
            **(
                {"Authorization": f"Bearer {os.environ['GITHUB_TOKEN']}"}
                if os.environ.get("GITHUB_TOKEN")
                else {}
            ),
        },
    )
    with urllib.request.urlopen(request, timeout=20) as response:  # noqa: S310 - fixed HTTPS host
        payload = json.load(response)
    return str(payload["commit"]["sha"])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--live", action="store_true")
    parser.add_argument("--allow-drift", action="store_true")
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()

    locks = json.loads(LOCKS.read_text(encoding="utf-8"))
    errors: list[str] = []
    results: list[dict[str, object]] = []
    if locks.get("automatic_updates") is not False:
        errors.append("integration lock policy must disable automatic updates")

    for item in locks.get("repositories", []):
        repository = str(item.get("repository", ""))
        revision = str(item.get("revision", ""))
        branch = str(item.get("default_branch", ""))
        passport_path = ROOT / str(item.get("passport", ""))
        if len(revision) != 40 or not all(character in "0123456789abcdef" for character in revision):
            errors.append(f"invalid revision pin for {repository}")
        if not passport_path.is_file():
            errors.append(f"missing passport for {repository}: {passport_path}")
            continue
        passport = json.loads(passport_path.read_text(encoding="utf-8"))
        if passport.get("repository") != repository or passport.get("revision") != revision:
            errors.append(f"passport/lock mismatch for {repository}")
        result: dict[str, object] = {
            "repository": repository,
            "branch": branch,
            "pinned_revision": revision,
            "mode": "live" if args.live else "offline",
            "status": "pin_consistent",
        }
        if args.live:
            try:
                head = live_head(repository, branch)
            except (urllib.error.URLError, TimeoutError, KeyError, ValueError) as exc:
                errors.append(f"could not read {repository}@{branch}: {exc}")
                result["status"] = "unavailable"
            else:
                result["observed_revision"] = head
                if head != revision:
                    result["status"] = "drift_detected"
                    if not args.allow_drift:
                        errors.append(f"integration drift detected for {repository}: {revision} -> {head}")
        results.append(result)

    receipt = {
        "schema_version": "org.searchright.integration-drift-receipt.v1",
        "status": "failed" if errors else "passed",
        "mode": "live_read_only" if args.live else "offline",
        "automatic_updates": False,
        "integrations": results,
        "errors": errors,
        "claim_boundary": "Drift detection is read-only and never changes a revision, dependency, issue, pull request or public claim.",
    }
    text = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    print(text, end="")
    if args.receipt:
        path = args.receipt if args.receipt.is_absolute() else ROOT / args.receipt
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
