#!/usr/bin/env python3
"""Conservative network-free secret signature scan over Git history and the worktree."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXCLUDED_PARTS = {".git", "target", "dist", "__pycache__", ".pytest_cache"}
PATTERNS: dict[str, re.Pattern[str]] = {
    "private_key_pem": re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
    "aws_access_key": re.compile(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b"),
    "github_token": re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,})\b"),
    "slack_token": re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{20,}\b"),
    "openai_token": re.compile(r"\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}\b"),
    "google_api_key": re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b"),
}


def scan_text(scope: str, path: str, text: str) -> list[dict[str, object]]:
    findings: list[dict[str, object]] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        for name, pattern in PATTERNS.items():
            if pattern.search(line):
                findings.append({"scope": scope, "path": path, "line": line_number, "pattern": name})
    return findings


def worktree_findings() -> tuple[list[dict[str, object]], int]:
    findings: list[dict[str, object]] = []
    files = 0
    for path in sorted(ROOT.rglob("*")):
        relative = path.relative_to(ROOT)
        if not path.is_file() or any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        files += 1
        findings.extend(scan_text("worktree", relative.as_posix(), text))
    return findings, files


def history_findings() -> tuple[list[dict[str, object]], int]:
    """Scan every reachable Git tree with one batched grep operation.

    The earlier implementation spawned ``git show`` once per file per commit,
    which was complete but needlessly slow as the generated issue corpus grew.
    ``git grep`` asks Git to search the reachable trees directly, skips binary
    blobs, and preserves commit/path/line attribution.
    """

    process = subprocess.run(
        ["git", "rev-list", "--all"], cwd=ROOT, check=True, capture_output=True, text=True
    )
    commits = [line for line in process.stdout.splitlines() if line]
    if not commits:
        return [], 0

    combined = "|".join(f"(?:{pattern.pattern})" for pattern in PATTERNS.values())
    command = [
        "git",
        "grep",
        "-I",
        "-n",
        "-P",
        "-e",
        combined,
        *commits,
        "--",
        ".",
        ":(exclude)dist/**",
        ":(exclude)target/**",
        ":(exclude)**/__pycache__/**",
        ":(exclude)**/.pytest_cache/**",
    ]
    search = subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)
    if search.returncode not in {0, 1}:
        raise RuntimeError(f"git grep failed: {search.stderr.strip()}")

    findings: list[dict[str, object]] = []
    for record in search.stdout.splitlines():
        try:
            commit, path, line_number, text = record.split(":", 3)
        except ValueError:
            continue
        for name, pattern in PATTERNS.items():
            if pattern.search(text):
                findings.append(
                    {
                        "scope": f"commit:{commit}",
                        "path": path,
                        "line": int(line_number),
                        "pattern": name,
                    }
                )
    return findings, len(commits)


def main() -> int:
    working, files = worktree_findings()
    history, commits = history_findings()
    findings = working + history
    receipt = {
        "schema_version": "org.searchright.local-secret-scan.v1",
        "status": "passed" if not findings else "failed",
        "patterns": sorted(PATTERNS),
        "worktree_text_files_scanned": files,
        "git_commits_scanned": commits,
        "finding_count": len(findings),
        "findings": findings,
        "limitations": [
            "Signature scan only; CI still runs full-history Gitleaks for entropy and rule-set coverage.",
            "Encrypted, compressed and non-UTF-8 content is not semantically inspected."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0 if not findings else 1


if __name__ == "__main__":
    raise SystemExit(main())
