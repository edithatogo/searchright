#!/usr/bin/env python3
"""Regression: the locked URL dependency must exclude RUSTSEC-2026-0592.

This is a lockfile-level guard, not an independent audit of dependency source.
"""

from __future__ import annotations

from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "Cargo.lock"


class UrlAdvisoryLockTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.packages = tomllib.loads(LOCK.read_text(encoding="utf-8"))["package"]

    def test_url_avoids_stack_overflow_advisory(self):
        packages = [item for item in self.packages if item["name"] == "url"]
        self.assertEqual(len(packages), 1, "Exactly one locked URL crate expected")
        version = tuple(int(n) for n in packages[0]["version"].split("."))
        self.assertGreaterEqual(
            version, (2, 5, 9), "RUSTSEC-2026-0592 requires url >=2.5.9"
        )
        self.assertEqual(
            packages[0]["source"], "registry+https://github.com/rust-lang/crates.io-index"
        )
        self.assertEqual(len(packages[0]["checksum"]), 64)

    def test_resolver_updated_form_urlencoded_together_with_url(self):
        packages = [item for item in self.packages if item["name"] == "form_urlencoded"]
        self.assertEqual(len(packages), 1)
        version = tuple(int(n) for n in packages[0]["version"].split("."))
        self.assertGreaterEqual(version, (1, 2, 3))

    def test_no_active_workaround_disables_advisory_policy(self):
        # Chrono's unrelated unmaintained advisory remains open; do not suppress it.
        deny = (ROOT / "deny.toml").read_text(encoding="utf-8")
        self.assertIn("ignore = []", deny)
        self.assertNotIn("RUSTSEC-2026-0592", deny)
        self.assertNotIn("RUSTSEC-2026-0543", deny)


if __name__ == "__main__":
    unittest.main()
