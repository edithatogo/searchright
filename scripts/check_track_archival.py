#!/usr/bin/env python3
"""Run deterministic archival renderer regressions as a repository gate."""

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    return subprocess.run(
        [sys.executable, str(ROOT / "tests/test_track_archival.py")],
        cwd=ROOT,
        check=False,
    ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
