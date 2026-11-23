#!/usr/bin/env python3
"""Regression for the pinned chrono-free upstream rmcp dependency experiment.

Structural evidence only: not an upstream release or security trust certification.
"""

from __future__ import annotations

from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
UPSTREAM = "9043cdfa8694d83bf506c2f3eb0313ec173edd7e"
SOURCE = "https://github.com/modelcontextprotocol/rust-sdk.git"


class ChronoFreeRmcpLockTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.locked = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
        cls.manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))

    def test_resolved_graph_excludes_unmaintained_chrono(self):
        packages = self.locked["package"]
        self.assertFalse(
            [package for package in packages if package["name"] == "chrono"],
            "RUSTSEC-2026-0543 must not be present in committed Cargo.lock",
        )

    def test_rmcp_and_macros_pin_exact_same_upstream_commit(self):
        packages = self.locked["package"]
        for crate in ("rmcp", "rmcp-macros"):
            matching = [package for package in packages if package["name"] == crate]
            self.assertEqual(len(matching), 1, f"one resolved {crate} expected")
            self.assertEqual(matching[0]["version"], "3.11.0")
            self.assertEqual(
                matching[0]["source"], f"git+{SOURCE}?rev={UPSTREAM}#{UPSTREAM}"
            )

    def test_feature_configuration_disables_default_chrono(self):
        rmcp = self.manifest["workspace"]["dependencies"]["rmcp"]
        self.assertIs(rmcp["default-features"], False)
        self.assertEqual(rmcp["git"], SOURCE)
        self.assertEqual(rmcp["rev"], UPSTREAM)
        self.assertNotIn("chrono", rmcp["features"])
        self.assertEqual(
            set(rmcp["features"]),
            {"base64", "server", "client", "elicitation", "macros", "transport-io", "schemars"},
        )

    def test_jiff_is_present_without_chrono_feature(self):
        packages = self.locked["package"]
        self.assertEqual(len([p for p in packages if p["name"] == "jiff"]), 1)
        self.assertFalse([p for p in packages if p["name"] == "iana-time-zone"])

    def test_advisory_policy_and_global_gates_not_suppressed(self):
        deny = (ROOT / "deny.toml").read_text(encoding="utf-8")
        self.assertIn("ignore = []", deny)
        self.assertIn("all-features = true", deny)
        self.assertNotIn("RUSTSEC-2026-0543", deny)
        self.assertNotIn("RUSTSEC-2026-0592", deny)


if __name__ == "__main__":
    unittest.main()
