from __future__ import annotations

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_coverage_policy", ROOT / "scripts" / "check_coverage_policy.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def current_inputs() -> tuple[dict[str, object], str, str]:
    policy = json.loads((ROOT / "verification" / "coverage-policy.json").read_text())
    workflow = (ROOT / ".github" / "workflows" / "coverage.yml").read_text()
    codecov = (ROOT / "codecov.yml").read_text()
    return policy, workflow, codecov


def test_current_policy_is_internally_consistent() -> None:
    policy, workflow, codecov = current_inputs()
    assert MODULE.validate_policy(policy, workflow, codecov) == []


def test_policy_rejects_maturity_target_at_or_below_90() -> None:
    policy, workflow, codecov = current_inputs()
    policy["maturity"]["target_line_percent"] = 90
    assert "Track 16 maturity target must remain greater than 90 percent" in MODULE.validate_policy(
        policy, workflow, codecov
    )


def test_policy_rejects_regression_allowance() -> None:
    policy, workflow, codecov = current_inputs()
    policy["admission"]["allowed_regression_percent"] = 0.1
    assert "admission policy must not allow regression" in MODULE.validate_policy(
        policy, workflow, codecov
    )


def test_lcov_totals_are_aggregated(tmp_path: Path) -> None:
    report = tmp_path / "lcov.info"
    report.write_text("TN:\nSF:a.rs\nLF:4\nLH:3\nend_of_record\nSF:b.rs\nLF:6\nLH:4\nend_of_record\n")
    assert MODULE.read_lcov_totals(report) == (7, 10)


def test_lcov_rejects_empty_report(tmp_path: Path) -> None:
    report = tmp_path / "lcov.info"
    report.write_text("TN:\n")
    try:
        MODULE.read_lcov_totals(report)
    except ValueError as exc:
        assert "valid positive LF/LH totals" in str(exc)
    else:
        raise AssertionError("empty LCOV report was accepted")
