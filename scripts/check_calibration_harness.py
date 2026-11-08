#!/usr/bin/env python3
"""Validate calibration harness contracts, protocol strata, and mathematical invariants."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = ROOT / "docs/evaluation/benchmark-and-calibration-protocol.md"
MANIFEST = ROOT / "benchmarks/methodology/manifest.json"

REQUIRED_STRATA = [
    "Contract conformance",
    "Dialect fidelity",
    "Retrieval sensitivity",
    "Deduplication",
    "Prioritisation",
    "Human calibration",
    "Operational robustness",
]

REQUIRED_PRIORITISATION_METRICS = [
    "recall_at_95_percent",
    "work_saved_over_sampling",
    "calibration_error",
    "abstention_rate",
]


def compute_wss(total: int, reviewed: int, target_recall: float) -> float | None:
    """Calculate Work Saved over Sampling at target recall."""
    if total <= 0 or not (0.0 <= target_recall <= 1.0):
        return None
    return target_recall - (min(reviewed, total) / total)


def compute_brier_score(predictions: list[tuple[float, bool]]) -> float | None:
    """Calculate Brier score for probability forecasts."""
    if not predictions:
        return None
    sq_err = sum((prob - (1.0 if outcome else 0.0)) ** 2 for prob, outcome in predictions)
    return sq_err / len(predictions)


def compute_ece(predictions: list[tuple[float, bool]], num_bins: int = 10) -> float | None:
    """Calculate Expected Calibration Error with equal-width binning."""
    if not predictions or num_bins <= 0:
        return None
    bin_width = 1.0 / num_bins
    bins: list[list[tuple[float, bool]]] = [[] for _ in range(num_bins)]
    for prob, outcome in predictions:
        if not (0.0 <= prob <= 1.0):
            return None
        idx = min(int(prob / bin_width), num_bins - 1)
        bins[idx].append((prob, outcome))

    total = len(predictions)
    ece = 0.0
    for b in bins:
        if not b:
            continue
        avg_conf = sum(p for p, _ in b) / len(b)
        avg_acc = sum(1.0 for _, y in b if y) / len(b)
        ece += (len(b) / total) * abs(avg_acc - avg_conf)
    return ece


def check_calibration_harness() -> dict[str, Any]:
    """Execute calibration harness protocol and invariant checks."""
    errors: list[str] = []
    strata_found = 0

    if not PROTOCOL.is_file():
        errors.append("missing benchmark and calibration protocol document")
    else:
        text = PROTOCOL.read_text(encoding="utf-8")
        for stratum in REQUIRED_STRATA:
            if stratum.lower() in text.lower():
                strata_found += 1
            else:
                errors.append(f"protocol missing required evaluation stratum: {stratum}")
        if "leakage controls" not in text.lower():
            errors.append("protocol missing explicit leakage controls section")
        if "release gate" not in text.lower():
            errors.append("protocol missing release gate declaration")

    if not MANIFEST.is_file():
        errors.append("missing methodology benchmark manifest")
    else:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        tasks = {t.get("id"): t for t in manifest.get("tasks", []) if isinstance(t, dict)}
        prioritisation = tasks.get("screening_prioritisation")
        if not prioritisation:
            errors.append("missing screening_prioritisation task in methodology manifest")
        else:
            metrics = prioritisation.get("metrics", [])
            for required_metric in REQUIRED_PRIORITISATION_METRICS:
                if required_metric not in metrics:
                    errors.append(f"screening_prioritisation task missing metric {required_metric}")
            claim = str(prioritisation.get("claim_boundary", "")).lower()
            if "cannot make final exclusions" not in claim and "advisory" not in claim:
                errors.append("screening_prioritisation claim boundary must state ranking is advisory")

    # Mathematical invariant self-tests
    math_checks = 0
    wss_test = compute_wss(100, 20, 0.95)
    if wss_test is None or abs(wss_test - 0.75) > 1e-9:
        errors.append(f"WSS invariant failed: expected 0.75, got {wss_test}")
    else:
        math_checks += 1

    perfect_calibration = [(1.0, True)] * 20 + [(0.0, False)] * 20
    brier_perfect = compute_brier_score(perfect_calibration)
    ece_perfect = compute_ece(perfect_calibration, num_bins=10)
    if brier_perfect is None or abs(brier_perfect) > 1e-9:
        errors.append(f"Brier score perfect invariant failed: got {brier_perfect}")
    else:
        math_checks += 1

    if ece_perfect is None or abs(ece_perfect) > 1e-9:
        errors.append(f"ECE perfect invariant failed: got {ece_perfect}")
    else:
        math_checks += 1

    inverted_calibration = [(1.0, False)] * 20 + [(0.0, True)] * 20
    brier_inverted = compute_brier_score(inverted_calibration)
    ece_inverted = compute_ece(inverted_calibration, num_bins=10)
    if brier_inverted is None or abs(brier_inverted - 1.0) > 1e-9:
        errors.append(f"Brier score inverted invariant failed: got {brier_inverted}")
    else:
        math_checks += 1

    if ece_inverted is None or abs(ece_inverted - 1.0) > 1e-9:
        errors.append(f"ECE inverted invariant failed: got {ece_inverted}")
    else:
        math_checks += 1

    return {
        "schema_version": "org.searchright.calibration-harness-receipt.v1",
        "status": "failed" if errors else "passed",
        "strata_verified": strata_found,
        "math_checks_passed": math_checks,
        "errors": errors,
        "claim_boundary": (
            "Calibration harness checks verify protocol strata, prioritisation metrics and "
            "mathematical invariant stability; prospective human calibration remains pending."
        ),
    }


def main() -> int:
    receipt = check_calibration_harness()
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
