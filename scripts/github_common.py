#!/usr/bin/env python3
"""Shared, bounded GitHub CLI helpers for explicit control-plane mutations."""
from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
RETRYABLE_MARKERS = (
    "secondary rate limit",
    "rate limit exceeded",
    "http 429",
    "http 500",
    "http 502",
    "http 503",
    "http 504",
    "connection reset",
    "temporarily unavailable",
    "try again later",
)
_LAST_GH_CALL_AT = 0.0


class GitHubCommandError(RuntimeError):
    """A GitHub CLI command failed after bounded retries."""


def bounded_env_int(name: str, default: int, *, minimum: int, maximum: int) -> int:
    """Read one integer environment control while enforcing a safe bound."""
    raw = os.environ.get(name)
    if raw is None or not raw.strip():
        return default
    try:
        value = int(raw)
    except ValueError as exc:
        raise GitHubCommandError(f"{name} must be an integer") from exc
    if not minimum <= value <= maximum:
        raise GitHubCommandError(f"{name} must be between {minimum} and {maximum}")
    return value


def _throttle_if_needed(args: list[str]) -> None:
    """Apply a bounded inter-call interval to GitHub CLI operations.

    A small default interval helps avoid GitHub secondary-rate-limit bursts when
    synchronising hundreds of issues and Project items. It can be increased for
    constrained installations but cannot be disabled below zero or expanded
    without bound.
    """
    global _LAST_GH_CALL_AT
    if not args or args[0] != "gh":
        return
    interval_ms = bounded_env_int(
        "SEARCHRIGHT_GITHUB_MIN_INTERVAL_MS", 75, minimum=0, maximum=2_000
    )
    if interval_ms:
        remaining = (interval_ms / 1_000) - (time.monotonic() - _LAST_GH_CALL_AT)
        if remaining > 0:
            time.sleep(remaining)
    _LAST_GH_CALL_AT = time.monotonic()


def run(
    args: list[str],
    *,
    allow_failure: bool = False,
    input_text: str | None = None,
    env: dict[str, str] | None = None,
    retries: int | None = None,
) -> subprocess.CompletedProcess[str]:
    """Run one command with bounded retry for transient GitHub failures."""
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    retry_limit = (
        bounded_env_int("SEARCHRIGHT_GITHUB_MAX_RETRIES", 6, minimum=0, maximum=10)
        if retries is None
        else retries
    )
    if not 0 <= retry_limit <= 10:
        raise GitHubCommandError("retries must be between 0 and 10")
    retry_cap = bounded_env_int(
        "SEARCHRIGHT_GITHUB_RETRY_CAP_SECS", 60, minimum=1, maximum=120
    )
    attempt = 0
    while True:
        _throttle_if_needed(args)
        process = subprocess.run(
            args,
            cwd=ROOT,
            text=True,
            input=input_text,
            capture_output=True,
            check=False,
            env=merged_env,
        )
        if process.returncode == 0:
            return process
        combined = f"{process.stdout}\n{process.stderr}".lower()
        retryable = any(marker in combined for marker in RETRYABLE_MARKERS)
        if retryable and attempt < retry_limit:
            time.sleep(min(2**attempt, retry_cap))
            attempt += 1
            continue
        if allow_failure:
            return process
        raise GitHubCommandError(
            f"command failed ({process.returncode}): {' '.join(args)}\n"
            f"stdout:\n{process.stdout}\nstderr:\n{process.stderr}"
        )


def run_json(args: list[str], **kwargs: Any) -> Any:
    """Run a command and parse its JSON output."""
    process = run(args, **kwargs)
    try:
        return json.loads(process.stdout or "null")
    except json.JSONDecodeError as exc:
        raise GitHubCommandError(
            f"command did not emit JSON: {' '.join(args)}\n{process.stdout}"
        ) from exc


def require_clean_tree() -> None:
    """Require a clean worktree before any remote mutation."""
    if run(["git", "status", "--porcelain"]).stdout.strip():
        raise GitHubCommandError("remote apply requires a clean Git working tree")


def require_gh() -> None:
    """Require an authenticated GitHub CLI session."""
    run(["gh", "--version"])
    run(["gh", "auth", "status"])


def repository_owner(repository: str) -> str:
    """Return the owner from a validated owner/name repository."""
    parts = repository.split("/")
    if len(parts) != 2 or not all(parts):
        raise GitHubCommandError("repository must use exact owner/name form")
    return parts[0]


def write_json_atomic(path: Path, payload: Any) -> None:
    """Write a JSON checkpoint atomically so interrupted runs remain resumable."""
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for attempt in range(6):
        try:
            temporary.replace(path)
            return
        except PermissionError:
            # Windows filesystem filters (including antivirus and sync clients)
            # can briefly retain a handle after write_text closes the file.
            if os.name != "nt" or attempt == 5:
                raise
            time.sleep(0.05 * (2**attempt))


def select_after(
    items: list[dict[str, Any]],
    *,
    key_name: str,
    resume_after: str | None,
    maximum: int | None,
) -> list[dict[str, Any]]:
    """Return a stable resumable slice without changing canonical ordering."""
    start = 0
    if resume_after:
        keys = [str(item.get(key_name, "")) for item in items]
        try:
            start = keys.index(resume_after) + 1
        except ValueError as exc:
            raise GitHubCommandError(f"resume key {resume_after!r} is not canonical") from exc
    selected = items[start:]
    if maximum is not None:
        if maximum < 1:
            raise GitHubCommandError("maximum must be at least 1")
        selected = selected[:maximum]
    return selected
