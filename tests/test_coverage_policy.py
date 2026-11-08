from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

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


class CoveragePolicyTests(unittest.TestCase):
    def test_current_policy_is_internally_consistent(self) -> None:
        policy, workflow, codecov = current_inputs()
        self.assertEqual(MODULE.validate_policy(policy, workflow, codecov), [])

    def test_policy_rejects_maturity_target_at_or_below_90(self) -> None:
        policy, workflow, codecov = current_inputs()
        policy["maturity"]["target_line_percent"] = 90
        self.assertIn(
            "Track 16 maturity target must remain greater than 90 percent",
            MODULE.validate_policy(policy, workflow, codecov),
        )

    def test_policy_rejects_regression_allowance(self) -> None:
        policy, workflow, codecov = current_inputs()
        policy["admission"]["allowed_regression_percent"] = 0.1
        self.assertIn(
            "admission policy must not allow regression",
            MODULE.validate_policy(policy, workflow, codecov),
        )

    def test_lcov_totals_are_aggregated(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = Path(tmp_dir) / "lcov.info"
            report.write_text("TN:\nSF:a.rs\nLF:4\nLH:3\nend_of_record\nSF:b.rs\nLF:6\nLH:4\nend_of_record\n")
            self.assertEqual(MODULE.read_lcov_totals(report), (7, 10))

    def test_lcov_rejects_empty_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = Path(tmp_dir) / "lcov.info"
            report.write_text("TN:\n")
            with self.assertRaises(ValueError) as cm:
                MODULE.read_lcov_totals(report)
            self.assertIn("valid positive LF/LH totals", str(cm.exception))


if __name__ == "__main__":
    unittest.main()
