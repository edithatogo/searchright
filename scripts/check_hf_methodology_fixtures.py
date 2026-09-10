#!/usr/bin/env python3
"""Run the network-free Hugging Face package and publisher safety suite."""
from __future__ import annotations

import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    suite = unittest.defaultTestLoader.discover(
        start_dir=str(ROOT / "tests"),
        pattern="test_huggingface_*.py",
        top_level_dir=str(ROOT),
    )
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    receipt = {
        "schema_version": "org.searchright.huggingface-package-test-receipt.v1",
        "status": "passed" if result.wasSuccessful() else "failed",
        "tests_run": result.testsRun,
        "failures": len(result.failures),
        "errors": len(result.errors),
        "skipped": len(result.skipped),
        "limitations": [
            "Network-free package and publisher-policy tests only; no Hugging Face repository or remote revision is created or observed."
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
