"""Unit tests for Generated SDKs, Fixture-backed Documentation and Adoption Operations Governance (Track 35)."""

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SdkDocsAdoptionGovernanceTests(unittest.TestCase):
    def setUp(self):
        self.sdk_manifest = json.loads(
            (ROOT / "sdk/manifest.json").read_text(encoding="utf-8")
        )
        self.contract_bindings = json.loads(
            (ROOT / "sdk/generated-contract-bindings.json").read_text(encoding="utf-8")
        )
        self.interface_catalog = json.loads(
            (ROOT / "contracts/interface-catalog.json").read_text(encoding="utf-8")
        )

    def test_sdk_manifest_and_generation_boundaries(self):
        """Verify SDK manifest enforces thin generated clients without duplicating Rust domain logic."""
        self.assertEqual(
            self.sdk_manifest.get("schema_version"),
            "org.searchright.sdk-manifest.v1",
        )
        targets = {t["language"]: t["status"] for t in self.sdk_manifest.get("targets", [])}
        self.assertEqual(targets.get("rust"), "native_facade")
        self.assertEqual(targets.get("python"), "generated_client_planned")
        self.assertEqual(targets.get("typescript"), "generated_client_planned")

        self.assertEqual(
            self.sdk_manifest.get("generation_policy"),
            "generated_clients_must_not_reimplement_domain_logic",
        )
        self.assertEqual(
            self.sdk_manifest.get("compatibility_policy"),
            "consumer_contract_and_semver_gate",
        )
        self.assertFalse(self.sdk_manifest.get("automatic_publication", True))
        self.assertIn("Contract-only", self.sdk_manifest.get("claim_boundary", ""))

    def test_contract_bindings_compiler_checked_status(self):
        """Verify contract bindings are compiler-checked, contain no domain logic, and cover canonical contracts."""
        bindings_meta = self.sdk_manifest.get("contract_bindings", {})
        self.assertEqual(bindings_meta.get("status"), "generated_compiler_checked")
        self.assertFalse(bindings_meta.get("domain_logic", True))
        self.assertFalse(bindings_meta.get("automatic_publication", True))

        self.assertEqual(
            self.contract_bindings.get("schema_version"),
            "org.searchright.generated-contract-bindings.v1",
        )
        self.assertEqual(self.contract_bindings.get("contracts"), 74)
        self.assertFalse(self.contract_bindings.get("domain_logic", True))
        self.assertFalse(self.contract_bindings.get("automatic_publication", True))
        self.assertEqual(self.contract_bindings.get("packages", {}).get("python"), "searchright-contracts")
        self.assertEqual(self.contract_bindings.get("packages", {}).get("typescript"), "@searchright/contracts")

        files = self.contract_bindings.get("files", {})
        for relative_path in files:
            self.assertTrue((ROOT / relative_path).exists(), f"Binding file {relative_path} must exist")

    def test_interface_catalog_cross_surface_parity(self):
        """Verify interface catalog defines canonical operations with CLI, MCP, and Rust facade mappings."""
        self.assertEqual(
            self.interface_catalog.get("schema_version"),
            "org.searchright.interface-catalog.v1",
        )
        entries = self.interface_catalog.get("entries", [])
        self.assertGreaterEqual(len(entries), 35)

        for entry in entries:
            self.assertTrue(bool(entry.get("operation")))
            self.assertTrue(bool(entry.get("cli_variant")))
            self.assertTrue(bool(entry.get("mcp_tool")))
            self.assertTrue(bool(entry.get("facade_method")))
            self.assertTrue(bool(entry.get("authority")))

        proc = subprocess.run(
            ["python3", "scripts/check_cli_mcp_parity.py"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, f"check_cli_mcp_parity failed: {proc.stderr}")

    def test_fixture_backed_tutorials_and_adoption_docs(self):
        """Verify adoption documentation mandates offline fixture-backed tutorials and evidence ceilings."""
        doc = (ROOT / "docs/sdk-and-adoption.md").read_text(encoding="utf-8")
        self.assertIn("The Rust facade is canonical", doc)
        self.assertIn("thin transport/validation layers", doc)
        self.assertIn("Every tutorial is fixture-backed, network-off by default", doc)
        self.assertIn("distinction between agent recommendations and human decisions", doc)

        quickstart = ROOT / "examples/quickstart/README.md"
        self.assertTrue(quickstart.exists())
        quickstart_text = quickstart.read_text(encoding="utf-8")
        self.assertIn("Searchright", quickstart_text)

        proc = subprocess.run(
            ["python3", "scripts/check_sdk_examples.py"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, f"check_sdk_examples failed: {proc.stderr}")

    def test_adoption_governance_and_support_operations(self):
        """Verify governance doctrine covers deprecation, support, and non-negotiable methodological contracts."""
        gov = (ROOT / "GOVERNANCE.md").read_text(encoding="utf-8")
        self.assertIn("Searchright", gov)

        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("Searchright", readme)

        agents_rule = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
        self.assertIn("Never represent PRISMA-S as a conduct standard; it is a reporting contract", agents_rule)
        self.assertIn("Never auto-exclude a study using an agent", agents_rule)

    def test_public_package_policy_unpublishable_bounds(self):
        """Verify public package policy keeps workspace crates unpublishable by default."""
        packages_doc = json.loads((ROOT / "release/public-packages.json").read_text(encoding="utf-8"))
        self.assertEqual(
            packages_doc.get("policy"),
            "deny_unlisted_and_require_compiler_semver_supply_chain_evidence",
        )
        for pkg in packages_doc.get("packages", []):
            self.assertFalse(pkg.get("publish_ready", True))
            self.assertEqual(pkg.get("publication_status"), "blocked")


if __name__ == "__main__":
    unittest.main()
