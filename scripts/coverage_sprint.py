#!/usr/bin/env python3
"""Track 16 coverage sprint helper.

Counts #[test] items across both inline #[cfg(test)] modules under src/
and integration tests under tests/, fixing the earlier tests/-only
undercount that misreported searchright-mcp as 0 tests.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ROOT / "crates"


def count_tests(crate: str) -> tuple[int, int]:
    inline = 0
    integ = 0
    src_dir = CRATES / crate / "src"
    if src_dir.is_dir():
        inline = sum(f.read_text(errors="ignore").count("#[test]") or f.read_text(errors="ignore").count("#[tokio::test]") for f in src_dir.glob("*.rs"))
    test_dir = CRATES / crate / "tests"
    if test_dir.is_dir():
        integ = sum(f.read_text(errors="ignore").count("#[test]") or f.read_text(errors="ignore").count("#[tokio::test]") for f in test_dir.glob("*.rs"))
    return inline, integ


def main() -> int:
    parser = argparse.ArgumentParser(description="Track 16 coverage sprint analysis")
    parser.add_argument("--limit", type=int, default=20, help="Show crates with total <= limit")
    args = parser.parse_args()

    rows = []
    for crate_path in sorted(CRATES.iterdir()):
        if not crate_path.is_dir():
            continue
        name = crate_path.name
        inline, integ = count_tests(name)
        rows.append((name, inline, integ, inline + integ))
    rows.sort(key=lambda r: r[3])

    print("crate\tinline\tintegration\ttotal")
    for name, inline, integ, total in rows:
        if total <= args.limit:
            print(f"{name}\t{inline}\t{integ}\t{total}")
    print()
    low = [r for r in rows if r[3] < 10]
    print(f"Below 10 total: {len(low)}")
    for r in low:
        print(f"  {r[0]}: {r[3]} (inline {r[1]}, integration {r[2]})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

