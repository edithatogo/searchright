#!/usr/bin/env python3
"""Governance and contract tests for Track 00: Foundation, Conductor and toolchain."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class FoundationToolchainGovernanceTests(unittest.TestCase):
    """Verify Conductor baseline, toolchain pinning, standards inheritance, and bootstrap entrypoints."""

    def test_conductor_upstream_lock_and_installers(self):
        """Verify upstream Conductor lockfile and installer scripts."""
        lock_path = ROOT / "conductor/upstream.lock.json"
        self.assertTrue(lock_path.is_file(), "Missing conductor/upstream.lock.json")
        lock_data = json.loads(lock_path.read_text(encoding="utf-8"))
        self.assertEqual(lock_data.get("schema_version"), "searchright.conductor-upstream-lock.v1")
        self.assertEqual(lock_data.get("observed_version"), "0.3.0")

        for installer in ("scripts/install-conductor.sh", "scripts/install-conductor.ps1"):
            path = ROOT / installer
            self.assertTrue(path.is_file(), f"Missing installer: {installer}")
            content = path.read_text(encoding="utf-8")
            self.assertIn("0.3.0", content)

    def test_conductor_context_files_exist_and_non_empty(self):
        """Verify essential Conductor context documentation exists."""
        required_docs = [
            "conductor/index.md",
            "conductor/product.md",
            "conductor/product-guidelines.md",
            "conductor/tech-stack.md",
            "conductor/workflow.md",
            "conductor/requirements.md",
            "conductor/design.md",
        ]
        for rel_path in required_docs:
            path = ROOT / rel_path
            self.assertTrue(path.is_file(), f"Missing context file: {rel_path}")
            self.assertGreater(path.stat().st_size, 50, f"File too small: {rel_path}")

    def test_rust_toolchain_and_workspace_policy(self):
        """Verify Rust toolchain pin and workspace configuration."""
        toolchain_path = ROOT / "rust-toolchain.toml"
        self.assertTrue(toolchain_path.is_file(), "Missing rust-toolchain.toml")
        content = toolchain_path.read_text(encoding="utf-8")
        self.assertIn('channel = "1.97.1"', content)

        cargo_lock = ROOT / "Cargo.lock"
        self.assertTrue(cargo_lock.is_file(), "Missing Cargo.lock")
        self.assertGreater(cargo_lock.stat().st_size, 1000)

        cargo_toml = ROOT / "Cargo.toml"
        self.assertTrue(cargo_toml.is_file(), "Missing Cargo.toml")
        cargo_content = cargo_toml.read_text(encoding="utf-8")
        self.assertIn("[workspace]", cargo_content)
        self.assertIn('resolver = "3"', cargo_content)

    def test_repository_standards_inheritance_and_companion_packet(self):
        """Verify repository-standards inheritance passport and companion packet."""
        passport_path = ROOT / "integration/passports/repository-standards-inheritance.json"
        self.assertTrue(passport_path.is_file(), "Missing inheritance passport")
        passport = json.loads(passport_path.read_text(encoding="utf-8"))
        self.assertEqual(
            passport.get("schema_version"),
            "org.searchright.integration-passport.v1",
        )
        self.assertEqual(passport.get("repository"), "edithatogo/repository-standards")

        companion_path = ROOT / "migration/companion-repositories/repository-standards.json"
        self.assertTrue(companion_path.is_file(), "Missing companion packet")
        companion = json.loads(companion_path.read_text(encoding="utf-8"))
        self.assertEqual(companion.get("schema_version"), "org.searchright.companion-change-packet.v1")
        self.assertFalse(companion.get("remote_mutation_permitted"))

        alignment_doc = ROOT / "docs/repository-standards-alignment.md"
        self.assertTrue(alignment_doc.is_file(), "Missing alignment doc")
        self.assertIn("edithatogo/repository-standards", alignment_doc.read_text(encoding="utf-8"))

    def test_bootstrap_and_verification_entrypoints(self):
        """Verify one-command bootstrap and verification entrypoints."""
        for script in (
            "scripts/bootstrap.sh",
            "scripts/bootstrap.ps1",
            "scripts/verify.sh",
            "scripts/run_static_harness.py",
            "scripts/validate_repository.py",
        ):
            path = ROOT / script
            self.assertTrue(path.is_file(), f"Missing script: {script}")
            self.assertGreater(path.stat().st_size, 50)

    def test_context_integrity_and_toolchain_checks(self):
        """Verify check_context_integrity.py and check_toolchain_manifest.py pass cleanly network-free."""
        for script in ("scripts/check_context_integrity.py", "scripts/check_toolchain_manifest.py"):
            proc = subprocess.run(
                ["python3", str(ROOT / script)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc.returncode, 0, f"{script} failed: {proc.stderr}")
            data = json.loads(proc.stdout)
            self.assertEqual(data.get("status"), "passed")
            self.assertEqual(data.get("errors"), [])


if __name__ == "__main__":
    unittest.main()
