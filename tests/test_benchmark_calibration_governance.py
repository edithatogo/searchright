"""Unit tests for Track 17 benchmark governance, calibration harness, and dataset provenance."""
from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_module(name: str, rel_path: str):
    path = ROOT / rel_path
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


GOVERNANCE = load_module("check_benchmark_governance", "scripts/check_benchmark_governance.py")
CALIBRATION = load_module("check_calibration_harness", "scripts/check_calibration_harness.py")
PROVENANCE = load_module("check_dataset_provenance", "scripts/check_dataset_provenance.py")
HF_CONTRACTS = load_module("check_hf_dataset_eval_contracts", "scripts/check_hf_dataset_eval_contracts.py")


class BenchmarkCalibrationGovernanceTests(unittest.TestCase):
    def test_benchmark_governance_passes_on_checked_in_repo(self) -> None:
        receipt = GOVERNANCE.check_benchmark_governance()
        self.assertEqual(receipt["status"], "passed")
        self.assertEqual(receipt["errors"], [])
        self.assertGreaterEqual(receipt["tasks_checked"], 6)
        self.assertGreaterEqual(receipt["sealed_cases_checked"], 4)

    def test_forbidden_key_detection(self) -> None:
        self.assertTrue(GOVERNANCE.contains_forbidden_key({"nested": {"gold": [1, 2]}}))
        self.assertTrue(GOVERNANCE.contains_forbidden_key({"labels": ["pos", "neg"]}))
        self.assertFalse(GOVERNANCE.contains_forbidden_key({"case_ids": ["c1", "c2"], "partition": "sealed_test"}))

    def test_calibration_harness_passes_on_checked_in_repo(self) -> None:
        receipt = CALIBRATION.check_calibration_harness()
        self.assertEqual(receipt["status"], "passed")
        self.assertEqual(receipt["errors"], [])
        self.assertEqual(receipt["strata_verified"], 7)
        self.assertEqual(receipt["math_checks_passed"], 5)

    def test_calibration_harness_math_helpers(self) -> None:
        # WSS calculation: 100 total, 20 reviewed, target recall 0.95 -> 0.95 - 0.20 = 0.75
        wss = CALIBRATION.compute_wss(100, 20, 0.95)
        self.assertIsNotNone(wss)
        self.assertAlmostEqual(wss, 0.75, places=6)

        # Invalid WSS inputs
        self.assertIsNone(CALIBRATION.compute_wss(0, 10, 0.95))
        self.assertIsNone(CALIBRATION.compute_wss(100, 10, 1.5))

        # Brier score
        perfect = [(1.0, True), (0.0, False)]
        self.assertAlmostEqual(CALIBRATION.compute_brier_score(perfect), 0.0, places=6)
        inverted = [(1.0, False), (0.0, True)]
        self.assertAlmostEqual(CALIBRATION.compute_brier_score(inverted), 1.0, places=6)
        self.assertIsNone(CALIBRATION.compute_brier_score([]))

        # Expected Calibration Error (ECE)
        self.assertAlmostEqual(CALIBRATION.compute_ece(perfect, num_bins=10), 0.0, places=6)
        self.assertAlmostEqual(CALIBRATION.compute_ece(inverted, num_bins=10), 1.0, places=6)
        self.assertIsNone(CALIBRATION.compute_ece([], num_bins=10))

    def test_dataset_provenance_passes_on_checked_in_repo(self) -> None:
        receipt = PROVENANCE.check_dataset_provenance()
        self.assertEqual(receipt["status"], "passed")
        self.assertEqual(receipt["errors"], [])
        self.assertEqual(receipt["passports_checked"], 1)
        self.assertEqual(receipt["upstreams_checked"], 1)

    def test_hf_dataset_eval_contracts_passes_on_checked_in_repo(self) -> None:
        receipt = HF_CONTRACTS.check_hf_dataset_eval_contracts()
        self.assertEqual(receipt["status"], "passed")
        self.assertEqual(receipt["errors"], [])
        self.assertEqual(receipt["contracts_checked"], 2)
        self.assertEqual(receipt["partitions_checked"], 3)

    def test_main_entry_points_return_zero(self) -> None:
        self.assertEqual(GOVERNANCE.main(), 0)
        self.assertEqual(CALIBRATION.main(), 0)
        self.assertEqual(PROVENANCE.main(), 0)
        self.assertEqual(HF_CONTRACTS.main(), 0)


if __name__ == "__main__":
    unittest.main()
