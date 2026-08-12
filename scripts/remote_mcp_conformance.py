#!/usr/bin/env python3
"""Emit fixture-level receipts for remote MCP access-control tests."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_AUTH_RECEIPT = ROOT / "verification" / "receipts" / "remote-mcp-auth-conformance.json"
DEFAULT_TENANCY_RECEIPT = ROOT / "verification" / "receipts" / "remote-mcp-tenancy-adversarial.json"
CARGO_COMMAND = [
    "rustup",
    "run",
    "1.97.1-x86_64-pc-windows-gnu",
    "cargo",
    "test",
    "-p",
    "searchright-access",
    "-p",
    "searchright-mcp",
    "--all-targets",
    "--all-features",
    "--locked",
]
JSON_TEST_ARGS = ["--", "--format", "json", "-Z", "unstable-options"]
TEXT_TEST_ARGS: list[str] = []
TEST_LINE = re.compile(r"^test (?P<name>\S+) \.\.\. (?P<status>ok|FAILED|ignored)$")
SUMMARY_LINE = re.compile(
    r"test result: (?P<result>ok|FAILED)\. (?P<passed>\d+) passed; "
    r"(?P<failed>\d+) failed; (?P<ignored>\d+) ignored"
)


@dataclass(frozen=True)
class Control:
    """One control family and the Rust tests expected to prove it."""

    key: str
    label: str
    required_tests: tuple[str, ...]
    cases: tuple[str, ...]


AUTH_CONTROLS = (
    Control(
        key="issuer",
        label="issuer trust",
        required_tests=(
            "issuer_region_and_token_age_are_bound_to_remote_policy",
            "rotating_jwks_verifies_current_key_and_rejects_removed_key",
        ),
        cases=("untrusted issuer is blocked", "removed signing key is blocked"),
    ),
    Control(
        key="principal",
        label="principal kind",
        required_tests=("final_eligibility_decision_by_non_human_is_denied",),
        cases=("non-human final eligibility decision is blocked",),
    ),
    Control(
        key="replay",
        label="token freshness and replay",
        required_tests=(
            "issuer_region_and_token_age_are_bound_to_remote_policy",
            "replayed_request_id_is_denied_on_second_use",
            "authenticated_streamable_http_initializes_and_replay_is_denied",
        ),
        cases=("stale token is blocked", "replayed bound request is blocked"),
    ),
    Control(
        key="approval",
        label="approval requirements",
        required_tests=(
            "external_write_without_human_approval_is_denied",
            "final_eligibility_decision_by_non_human_is_denied",
        ),
        cases=(
            "external write without human approval is blocked",
            "final decision path requires a human principal and approval",
        ),
    ),
    Control(
        key="transport",
        label="authenticated Streamable HTTP",
        required_tests=("authenticated_streamable_http_initializes_and_replay_is_denied",),
        cases=("signed request initializes over the loopback transport",),
    ),
    Control(
        key="audit",
        label="redacted audit correlation",
        required_tests=("audit_events_are_redacted_and_correlated",),
        cases=("audit event correlates the request without raw tenant, principal or token",),
    ),
    Control(
        key="request_budget",
        label="request timeout",
        required_tests=("authenticated_request_budget_times_out_fail_closed",),
        cases=("pending authenticated request is cancelled at its wall-clock budget",),
    ),
)
TENANCY_CONTROLS = (
    Control(
        key="tenant",
        label="tenant isolation",
        required_tests=("cross_tenant_request_is_denied",),
        cases=("cross-tenant request is blocked",),
    ),
    Control(
        key="region",
        label="region residency",
        required_tests=(
            "disallowed_region_is_denied",
            "issuer_region_and_token_age_are_bound_to_remote_policy",
        ),
        cases=("disallowed region is blocked",),
    ),
    Control(
        key="scope",
        label="scope restriction",
        required_tests=("unrequested_scope_is_denied",),
        cases=("unrequested tenant administration scope is blocked",),
    ),
    Control(
        key="rate",
        label="rate budget",
        required_tests=("request_replay_rate_and_concurrency_fail_closed",),
        cases=("request at the configured per-minute boundary is blocked",),
    ),
    Control(
        key="concurrency",
        label="concurrency budget",
        required_tests=("request_replay_rate_and_concurrency_fail_closed",),
        cases=("active task count at the configured concurrency boundary is blocked",),
    ),
)
ACCUMULATION_TEST = "multiple_control_violations_accumulate_blockers"
COMPLIANT_TEST = "fully_compliant_request_is_permitted"
AUTH_LIMITATIONS = [
    "Fixture-level Rust tests exercise a loopback authenticated Streamable HTTP transport; they do not prove a hosted production deployment.",
    "No hosted or multi-tenant production deployment was exercised.",
    "No real OAuth/OIDC integration against a live identity provider was exercised.",
    "No rollback rehearsal against a real deployment was performed.",
    "No independent security or privacy review was performed.",
]
TENANCY_LIMITATIONS = [
    "Fixture-level Rust tests exercise loopback transport and in-process tenant controls only; they do not prove a hosted production deployment.",
    "No hosted or multi-tenant production deployment was exercised.",
    "No real OAuth/OIDC integration against a live identity provider was exercised.",
    "No rollback rehearsal against a real deployment was performed.",
    "No independent security or privacy review was performed.",
]


@dataclass
class CargoRun:
    """Captured access-test execution."""

    command: list[str]
    returncode: int
    stdout: str
    stderr: str
    parser: str
    tests: dict[str, str]
    summary: dict[str, int | str]
    errors: list[str]


def short_test_name(name: str) -> str:
    """Return the function name from a Rust test path."""
    return name.rsplit("::", 1)[-1]


def run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    """Run Cargo from the repository root while capturing output."""
    environment = {**os.environ, "CARGO_TERM_COLOR": "never"}
    return subprocess.run(  # noqa: S603 - command is fixed by this harness
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        env=environment,
        check=False,
    )


def parse_json_lines(output: str) -> tuple[dict[str, str], dict[str, int | str], list[str]]:
    """Parse libtest JSON events when the toolchain supports them."""
    tests: dict[str, str] = {}
    summary: dict[str, int | str] = {}
    errors: list[str] = []
    for line in output.splitlines():
        if not line.startswith("{"):
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError as exc:
            errors.append(f"could not parse JSON test event: {exc}")
            continue
        if not isinstance(payload, dict):
            continue
        event_type = payload.get("type")
        if event_type == "test":
            name = payload.get("name")
            event = payload.get("event")
            if isinstance(name, str) and isinstance(event, str) and event in {"ok", "failed", "ignored"}:
                tests[short_test_name(name)] = "FAILED" if event == "failed" else event
        elif event_type == "suite":
            event = payload.get("event")
            if event == "ok":
                summary["result"] = "ok"
            elif event == "failed":
                summary["result"] = "FAILED"
            for key in ("passed", "failed", "ignored"):
                value = payload.get(key)
                if isinstance(value, int):
                    summary[key] = value
    return tests, summary, errors


def parse_text_output(output: str) -> tuple[dict[str, str], dict[str, int | str], list[str]]:
    """Parse standard libtest text output."""
    tests: dict[str, str] = {}
    summary: dict[str, int | str] = {}
    summary_total = -1
    for line in output.splitlines():
        stripped = line.strip()
        test_match = TEST_LINE.match(stripped)
        if test_match:
            name = short_test_name(test_match.group("name"))
            tests[name] = test_match.group("status")
            continue
        summary_match = SUMMARY_LINE.search(stripped)
        if summary_match:
            candidate = {
                "result": summary_match.group("result"),
                "passed": int(summary_match.group("passed")),
                "failed": int(summary_match.group("failed")),
                "ignored": int(summary_match.group("ignored")),
            }
            total = sum(candidate[key] for key in ("passed", "failed", "ignored"))
            if total > summary_total:
                summary = candidate
                summary_total = total
    return tests, summary, []


def run_access_tests() -> CargoRun:
    """Run access tests, preferring structured libtest output."""
    json_command = [*CARGO_COMMAND, *JSON_TEST_ARGS]
    json_run = run_command(json_command)
    json_output = json_run.stdout + "\n" + json_run.stderr
    tests, summary, parse_errors = parse_json_lines(json_output)
    if tests and summary:
        return CargoRun(
            command=json_command,
            returncode=json_run.returncode,
            stdout=json_run.stdout,
            stderr=json_run.stderr,
            parser="libtest-json",
            tests=tests,
            summary=summary,
            errors=parse_errors,
        )

    text_command = [*CARGO_COMMAND, *TEXT_TEST_ARGS]
    text_run = run_command(text_command)
    text_output = text_run.stdout + "\n" + text_run.stderr
    tests, summary, parse_errors = parse_text_output(text_output)
    errors = parse_errors
    if not tests:
        errors.append("no Rust test result lines were observed")
    if not summary:
        errors.append("no Rust test summary line was observed")
    return CargoRun(
        command=text_command,
        returncode=text_run.returncode,
        stdout=text_run.stdout,
        stderr=text_run.stderr,
        parser="libtest-text",
        tests=tests,
        summary=summary,
        errors=errors,
    )


def first_meaningful_line(output: str) -> str:
    """Select a compact diagnostic from command output."""
    for line in output.splitlines():
        stripped = line.strip()
        if stripped:
            return stripped[:300]
    return "no diagnostic text emitted"


def evaluate_control(control: Control, tests: dict[str, str]) -> dict[str, Any]:
    """Build one per-control receipt entry."""
    observed = []
    missing = []
    failed = []
    for test in control.required_tests:
        status = tests.get(test)
        if status is None:
            missing.append(test)
        else:
            observed.append(test)
            if status != "ok":
                failed.append(test)
    if missing:
        status = "not_proven"
    elif failed:
        status = "failed"
    else:
        status = "passed"
    return {
        "control": control.key,
        "label": control.label,
        "status": status,
        "tests": observed,
        "missing_tests": missing,
        "cases": list(control.cases),
    }


def receipt_status(entries: list[dict[str, Any]], extra_errors: list[str]) -> str:
    """Return the aggregate status for a receipt."""
    if extra_errors or any(entry["status"] == "failed" for entry in entries):
        return "failed"
    if any(entry["status"] != "passed" for entry in entries):
        return "not_proven"
    return "passed"


def test_record(name: str, tests: dict[str, str]) -> dict[str, Any]:
    """Represent a supporting test without overstating absence."""
    status = tests.get(name)
    return {
        "name": name,
        "observed": status is not None,
        "status": "not_proven" if status is None else status,
    }


def command_record(run: CargoRun) -> dict[str, Any]:
    """Summarise the command used as evidence."""
    return {
        "command": run.command,
        "parser": run.parser,
        "returncode": run.returncode,
        "summary": run.summary,
        "tests_observed": len(run.tests),
    }


def source_binding() -> dict[str, Any]:
    """Bind the observation to the exact Git revision and tree state."""
    revision = run_command(["git", "rev-parse", "HEAD"])
    status = run_command(["git", "status", "--porcelain"])
    return {
        "revision": revision.stdout.strip() if revision.returncode == 0 else None,
        "working_tree_clean": status.returncode == 0 and not status.stdout.strip(),
    }


def cargo_errors(run: CargoRun) -> list[str]:
    """Collect command-level failures without duplicating normal fallback notes."""
    errors = list(run.errors)
    if run.returncode != 0:
        diagnostic = first_meaningful_line(run.stderr) or first_meaningful_line(run.stdout)
        errors.append(f"access-control cargo test command failed with exit code {run.returncode}: {diagnostic}")
    failed_tests = sorted(name for name, status in run.tests.items() if status == "FAILED")
    for name in failed_tests:
        errors.append(f"Rust test failed: {name}")
    return errors


def build_auth_receipt(run: CargoRun) -> dict[str, Any]:
    """Build the authentication-facing receipt."""
    controls = [evaluate_control(control, run.tests) for control in AUTH_CONTROLS]
    errors = cargo_errors(run)
    for control in controls:
        if control["status"] == "not_proven":
            errors.append(f"control not proven: {control['control']}")
    status = receipt_status(controls, errors)
    return {
        "schema_version": "org.searchright.remote-mcp-auth-conformance-receipt.v1",
        "status": status,
        "evidence_level": "fixture_proven" if status == "passed" else "source_verified",
        "evidence_scope": "Rust access-policy and loopback authenticated Streamable HTTP fixture tests",
        "lp_track": "34",
        "roadmap_item": "LP-003",
        "command": command_record(run),
        "source": source_binding(),
        "controls_checked": len(controls),
        "controls_passed": sum(1 for control in controls if control["status"] == "passed"),
        "controls_not_proven": sum(1 for control in controls if control["status"] == "not_proven"),
        "controls_failed": sum(1 for control in controls if control["status"] == "failed"),
        "controls": controls,
        "supporting_tests": [test_record(COMPLIANT_TEST, run.tests)],
        "errors": errors,
        "limitations": AUTH_LIMITATIONS,
    }


def build_tenancy_receipt(run: CargoRun) -> dict[str, Any]:
    """Build the tenancy-facing receipt."""
    controls = [evaluate_control(control, run.tests) for control in TENANCY_CONTROLS]
    errors = cargo_errors(run)
    for control in controls:
        if control["status"] == "not_proven":
            errors.append(f"control not proven: {control['control']}")
    accumulation = test_record(ACCUMULATION_TEST, run.tests)
    if accumulation["status"] == "not_proven":
        errors.append("multiple-violation accumulation case is not proven")
    status = receipt_status(controls, errors)
    if status == "passed" and accumulation["status"] != "ok":
        status = "not_proven" if accumulation["status"] == "not_proven" else "failed"
    return {
        "schema_version": "org.searchright.remote-mcp-tenancy-adversarial-receipt.v1",
        "status": status,
        "evidence_level": "fixture_proven" if status == "passed" else "source_verified",
        "evidence_scope": "Rust access-policy and loopback authenticated Streamable HTTP fixture tests",
        "lp_track": "34",
        "roadmap_item": "LP-003",
        "command": command_record(run),
        "source": source_binding(),
        "controls_checked": len(controls),
        "controls_passed": sum(1 for control in controls if control["status"] == "passed"),
        "controls_not_proven": sum(1 for control in controls if control["status"] == "not_proven"),
        "controls_failed": sum(1 for control in controls if control["status"] == "failed"),
        "controls": controls,
        "adversarial_cases": {
            "boundary_cases": [
                "rate budget is denied at the configured per-minute boundary",
                "concurrency budget is denied at the configured active-task boundary",
            ],
            "multiple_violations_accumulate": accumulation,
            "compliant_baseline": test_record(COMPLIANT_TEST, run.tests),
        },
        "errors": errors,
        "limitations": TENANCY_LIMITATIONS,
    }


def resolve_receipt_paths(paths: list[Path] | None) -> tuple[Path | None, Path | None]:
    """Resolve optional receipt destinations for one or both receipts."""
    if not paths:
        return DEFAULT_AUTH_RECEIPT, DEFAULT_TENANCY_RECEIPT
    auth_path: Path | None = None
    tenancy_path: Path | None = None
    for path in paths:
        resolved = path if path.is_absolute() else ROOT / path
        if resolved.suffix == "":
            auth_path = resolved / DEFAULT_AUTH_RECEIPT.name
            tenancy_path = resolved / DEFAULT_TENANCY_RECEIPT.name
        elif "tenancy" in resolved.name:
            tenancy_path = resolved
        elif "auth" in resolved.name:
            auth_path = resolved
        else:
            auth_path = resolved
    return auth_path, tenancy_path


def write_receipt(path: Path | None, receipt: dict[str, Any]) -> None:
    """Write one receipt if a path was requested."""
    if path is None:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--receipt",
        type=Path,
        action="append",
        help="Receipt file or directory. Repeat for auth and tenancy files.",
    )
    args = parser.parse_args()

    run = run_access_tests()
    auth_receipt = build_auth_receipt(run)
    tenancy_receipt = build_tenancy_receipt(run)
    auth_path, tenancy_path = resolve_receipt_paths(args.receipt)
    write_receipt(auth_path, auth_receipt)
    write_receipt(tenancy_path, tenancy_receipt)
    output = {
        "auth_receipt": str(auth_path) if auth_path else None,
        "tenancy_receipt": str(tenancy_path) if tenancy_path else None,
        "auth_status": auth_receipt["status"],
        "tenancy_status": tenancy_receipt["status"],
        "tests_observed": len(run.tests),
        "parser": run.parser,
        "errors": auth_receipt["errors"] + tenancy_receipt["errors"],
    }
    print(json.dumps(output, indent=2, sort_keys=True))
    return 0 if auth_receipt["status"] == "passed" and tenancy_receipt["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
