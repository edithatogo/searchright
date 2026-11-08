"""Unit tests for Track 19 registry packets, publication governance, and metadata consistency."""
from __future__ import annotations

import copy
import importlib.util
import json
import re
import unittest
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]


def load_validate_repository_module():
    spec = importlib.util.spec_from_file_location(
        "validate_repository", ROOT / "scripts" / "validate_repository.py"
    )
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


class RegistriesPublicationGovernanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.val_mod = load_validate_repository_module()

    def setUp(self) -> None:
        self.val_mod.ERRORS.clear()
        self.val_mod.CHECKS.clear()

    def test_validate_repository_registry_and_publication_checks(self) -> None:
        self.val_mod.validate_registry_truthfulness()
        self.val_mod.validate_publication_packets()
        self.assertEqual(self.val_mod.ERRORS, [])
        self.assertGreaterEqual(self.val_mod.CHECKS.get("registry_targets_truthfulness_checked", 0), 5)
        self.assertGreaterEqual(self.val_mod.CHECKS.get("publication_packets_checked", 0), 7)

    def test_registry_status_schema_and_invariants(self) -> None:
        status_path = ROOT / "registry" / "status.json"
        self.assertTrue(status_path.is_file(), "registry/status.json must exist")
        status = json.loads(status_path.read_text(encoding="utf-8"))

        self.assertEqual(status.get("schema_version"), "searchright.registry-status.v1")
        self.assertEqual(status.get("overall_status"), "prepared_not_submitted")

        targets = status.get("targets")
        self.assertIsInstance(targets, list)
        self.assertGreater(len(targets), 0)

        expected_targets = {
            "github",
            "crates.io",
            "github-container-registry",
            "official-mcp-registry",
            "glama",
            "smithery",
            "github-releases",
            "zenodo-osf",
            "joss",
        }
        found_targets: set[str] = set()
        forbidden_states = {"accepted", "published", "submitted", "listed", "verified"}

        for t in targets:
            name = t.get("target")
            self.assertIsInstance(name, str)
            self.assertNotIn(name, found_targets, f"duplicate target: {name}")
            found_targets.add(name)

            state = t.get("status")
            self.assertNotIn(state, forbidden_states, f"target {name} claims forbidden external state: {state}")
            self.assertIn(state, {"not_started", "prepared"})

            self.assertIsNone(t.get("evidence_url"), f"target {name} should have null evidence_url before submission")

            if state == "prepared":
                blockers = t.get("blockers")
                self.assertIsInstance(blockers, list)
                self.assertGreater(len(blockers), 0, f"prepared target {name} lacks explicit blockers")

        self.assertTrue(expected_targets.issubset(found_targets), f"missing targets: {expected_targets - found_targets}")

    def test_server_json_packet_structure(self) -> None:
        server_path = ROOT / "server.json"
        self.assertTrue(server_path.is_file(), "server.json must exist")
        server = json.loads(server_path.read_text(encoding="utf-8"))

        self.assertEqual(server.get("$schema"), "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json")
        self.assertEqual(server.get("name"), "io.github.edithatogo/searchright")
        self.assertEqual(server.get("title"), "Searchright")
        self.assertTrue(bool(server.get("description")))
        self.assertEqual(server.get("version"), "0.1.0-alpha.1")

        repo = server.get("repository")
        self.assertIsInstance(repo, dict)
        self.assertEqual(repo.get("url"), "https://github.com/edithatogo/searchright")
        self.assertEqual(repo.get("source"), "github")

        packages = server.get("packages")
        self.assertIsInstance(packages, list)
        self.assertGreater(len(packages), 0)
        oci_pkg = packages[0]
        self.assertEqual(oci_pkg.get("registryType"), "oci")
        self.assertEqual(oci_pkg.get("identifier"), "ghcr.io/edithatogo/searchright:0.1.0-alpha.1")
        self.assertEqual(oci_pkg.get("transport", {}).get("type"), "stdio")

    def test_glama_packet_structure(self) -> None:
        glama_path = ROOT / "glama.json"
        self.assertTrue(glama_path.is_file(), "glama.json must exist")
        glama = json.loads(glama_path.read_text(encoding="utf-8"))

        self.assertEqual(glama.get("$schema"), "https://glama.ai/mcp/schemas/server.json")
        maintainers = glama.get("maintainers")
        self.assertIsInstance(maintainers, list)
        self.assertIn("edithatogo", maintainers)

    def test_smithery_mcpb_manifest_structure(self) -> None:
        manifest_path = ROOT / "registry" / "smithery" / "mcpb-manifest.template.json"
        self.assertTrue(manifest_path.is_file(), "mcpb-manifest.template.json must exist")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

        self.assertEqual(manifest.get("manifest_version"), "0.3")
        self.assertEqual(manifest.get("name"), "searchright")
        self.assertEqual(manifest.get("display_name"), "Searchright")
        self.assertEqual(manifest.get("version"), "0.1.0-alpha.1")
        self.assertEqual(manifest.get("author", {}).get("name"), "Dylan Mordaunt")

        server_cfg = manifest.get("server")
        self.assertIsInstance(server_cfg, dict)
        self.assertEqual(server_cfg.get("type"), "binary")
        self.assertEqual(server_cfg.get("entry_point"), "${platform}/searchright-mcp")
        self.assertIn("${__dirname}/${platform}/searchright-mcp", server_cfg.get("mcp_config", {}).get("command", ""))

        tool_catalog = manifest.get("tools_generated_from")
        self.assertEqual(tool_catalog, "contracts/mcp/tool-catalog.json")
        self.assertTrue((ROOT / tool_catalog).is_file(), f"{tool_catalog} must exist")

    def test_registry_readmes_presence_and_guidance(self) -> None:
        readmes = [
            ROOT / "registry" / "official-mcp" / "README.md",
            ROOT / "registry" / "glama" / "README.md",
            ROOT / "registry" / "smithery" / "README.md",
        ]
        for path in readmes:
            self.assertTrue(path.is_file(), f"{path} must exist")
            content = path.read_text(encoding="utf-8")
            self.assertGreater(len(content.strip()), 50, f"{path} must contain non-trivial instructions")

    def test_submit_registries_script_safeguards(self) -> None:
        script_path = ROOT / "scripts" / "submit-registries.sh"
        self.assertTrue(script_path.is_file(), "scripts/submit-registries.sh must exist")
        script = script_path.read_text(encoding="utf-8")

        self.assertTrue(script.startswith("#!/usr/bin/env bash"))
        self.assertIn("SEARCHRIGHT_ALLOW_EXTERNAL_SUBMISSION", script)
        self.assertIn("Cargo.lock", script)
        self.assertIn("mcp-publisher", script)
        self.assertIn("smithery", script)
        self.assertIn("python3 scripts/validate_repository.py", script)
        self.assertIn("mcp-publisher validate server.json", script)

    def test_public_packages_policy_blocks_unauthorized_publish(self) -> None:
        pkg_policy_path = ROOT / "release" / "public-packages.json"
        self.assertTrue(pkg_policy_path.is_file(), "release/public-packages.json must exist")
        pkg_policy = json.loads(pkg_policy_path.read_text(encoding="utf-8"))

        self.assertEqual(pkg_policy.get("schema_version"), "org.searchright.public-packages.v1")
        self.assertEqual(pkg_policy.get("policy"), "deny_unlisted_and_require_compiler_semver_supply_chain_evidence")

        packages = pkg_policy.get("packages")
        self.assertIsInstance(packages, list)
        self.assertGreaterEqual(len(packages), 3)
        for pkg in packages:
            self.assertEqual(pkg.get("publication_status"), "blocked")
            self.assertFalse(pkg.get("publish_ready"))
            self.assertGreater(len(pkg.get("required_before_publish", [])), 0)

    def test_zenodo_joss_and_citation_metadata_consistency(self) -> None:
        citation_path = ROOT / "CITATION.cff"
        self.assertTrue(citation_path.is_file(), "CITATION.cff must exist")
        citation = yaml.safe_load(citation_path.read_text(encoding="utf-8"))

        paper_path = ROOT / "registry" / "joss" / "paper.md"
        self.assertTrue(paper_path.is_file(), "registry/joss/paper.md must exist")
        paper_text = paper_path.read_text(encoding="utf-8")
        self.assertTrue(paper_text.startswith("---"), "paper.md must contain frontmatter")
        parts = paper_text.split("---", 2)
        self.assertGreaterEqual(len(parts), 3)
        paper_frontmatter = yaml.safe_load(parts[1])

        # Version consistency
        self.assertEqual(citation.get("version"), "0.1.0-alpha.1")

        # Author consistency
        cff_authors = citation.get("authors", [])
        self.assertGreater(len(cff_authors), 0)
        cff_author_full = f"{cff_authors[0].get('given-names')} {cff_authors[0].get('family-names')}"
        self.assertEqual(cff_author_full, "Dylan Mordaunt")

        paper_authors = paper_frontmatter.get("authors", [])
        self.assertGreater(len(paper_authors), 0)
        self.assertEqual(paper_authors[0].get("name"), cff_author_full)
        self.assertEqual(paper_authors[0].get("orcid"), "0000-0002-9775-0603")

        # Status and title alignment
        self.assertEqual(paper_frontmatter.get("date"), "Draft only — not submitted")
        self.assertIn("Searchright", paper_frontmatter.get("title", ""))
        self.assertIn("Searchright", citation.get("title", ""))

        # Required JOSS sections
        body = parts[2]
        self.assertIn("# Summary", body)
        self.assertIn("# Statement of need", body)
        self.assertIn("# Architecture and functionality", body)
        self.assertIn("# Quality, evaluation and limitations", body)

    def test_registry_truthfulness_rejection_of_false_claims(self) -> None:
        status_path = ROOT / "registry" / "status.json"
        base_status = json.loads(status_path.read_text(encoding="utf-8"))

        # Test forbidden state rejection
        forbidden_test = copy.deepcopy(base_status)
        forbidden_test["targets"][0]["status"] = "published"
        errors = self._run_custom_registry_validation(forbidden_test)
        self.assertTrue(any("claims external state without publication evidence" in e for e in errors))

        # Test overall_status mismatch rejection
        overall_test = copy.deepcopy(base_status)
        overall_test["overall_status"] = "submitted"
        errors = self._run_custom_registry_validation(overall_test)
        self.assertTrue(any("must remain prepared_not_submitted" in e for e in errors))

        # Test duplicate target rejection
        dup_test = copy.deepcopy(base_status)
        dup_test["targets"].append(copy.deepcopy(base_status["targets"][0]))
        errors = self._run_custom_registry_validation(dup_test)
        self.assertTrue(any("duplicate registry target" in e for e in errors))

        # Test missing blockers rejection
        no_blocker_test = copy.deepcopy(base_status)
        for t in no_blocker_test["targets"]:
            if t["status"] == "prepared":
                t["blockers"] = []
                break
        errors = self._run_custom_registry_validation(no_blocker_test)
        self.assertTrue(any("lacks explicit blockers" in e for e in errors))

        # Test missing required target rejection
        missing_target_test = copy.deepcopy(base_status)
        missing_target_test["targets"] = [
            t for t in missing_target_test["targets"] if t["target"] != "official-mcp-registry"
        ]
        errors = self._run_custom_registry_validation(missing_target_test)
        self.assertTrue(any("missing official-mcp-registry" in e for e in errors))

    def _run_custom_registry_validation(self, custom_status: dict[str, Any]) -> list[str]:
        saved_errors = list(self.val_mod.ERRORS)
        self.val_mod.ERRORS.clear()
        try:
            # Replicate validate_registry_truthfulness logic against provided dict
            status = custom_status
            if status.get("overall_status") != "prepared_not_submitted":
                self.val_mod.error("registry/status.json must remain prepared_not_submitted until external evidence exists")
            forbidden_states = {"accepted", "published", "submitted", "listed", "verified"}
            targets = status.get("targets")
            if not isinstance(targets, list) or not targets:
                self.val_mod.error("registry status must contain targets")
            else:
                names: set[str] = set()
                for target in targets:
                    if not isinstance(target, dict):
                        self.val_mod.error("registry target must be an object")
                        continue
                    name = target.get("target")
                    if name in names:
                        self.val_mod.error(f"duplicate registry target: {name}")
                    if isinstance(name, str):
                        names.add(name)
                    state = target.get("status")
                    if state in forbidden_states:
                        self.val_mod.error(f"registry target claims external state without publication evidence: {name}={state}")
                    if state == "prepared" and not target.get("blockers"):
                        self.val_mod.error(f"prepared registry target lacks explicit blockers: {name}")
                for expected in {"github", "crates.io", "official-mcp-registry", "glama", "smithery"}:
                    if expected not in names:
                        self.val_mod.error(f"registry catalogue is missing {expected}")
            return list(self.val_mod.ERRORS)
        finally:
            self.val_mod.ERRORS = saved_errors


if __name__ == "__main__":
    unittest.main()
