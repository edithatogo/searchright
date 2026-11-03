"""Unit tests for Track 21 licensed BYO-access adapter governance and contracts."""
from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
LICENSED_DIR = ROOT / "contracts" / "licensed"
INDEX_PATH = LICENSED_DIR / "index.json"
EXAMPLE_PATH = ROOT / "contracts" / "examples" / "licensed-adapter.yaml"
TRACEABILITY_PATH = ROOT / "conductor" / "tracks" / "21-licensed-adapters" / "traceability.json"

DNS_HOST_PATTERN = re.compile(r"^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$")
ENV_VAR_PATTERN = re.compile(r"^[A-Z0-9_]+$")


class LicensedAdaptersGovernanceTests(unittest.TestCase):
    def test_licensed_adapter_index_schema_and_invariants(self) -> None:
        self.assertTrue(INDEX_PATH.is_file(), "contracts/licensed/index.json must exist")
        index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))

        self.assertEqual(index.get("schema_version"), "org.searchright.licensed-adapter-index.v1")
        self.assertEqual(index.get("default_execution"), "disabled")
        self.assertTrue(index.get("credential_values_must_never_be_logged"))
        self.assertEqual(index.get("raw_response_persistence"), "disabled_by_default")

        profiles = index.get("profiles")
        self.assertIsInstance(profiles, list)
        self.assertGreaterEqual(len(profiles), 3)

        expected_profiles = {"embase.yaml", "scopus.yaml", "web-of-science.yaml"}
        self.assertEqual(set(profiles), expected_profiles)

        actual_yamls = {p.name for p in LICENSED_DIR.glob("*.yaml")}
        self.assertEqual(set(profiles), actual_yamls, "Index profiles must match files on disk")

    def test_licensed_adapter_profiles_conformance(self) -> None:
        index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
        seen_providers: set[str] = set()

        for filename in index["profiles"]:
            profile_path = LICENSED_DIR / filename
            self.assertTrue(profile_path.is_file(), f"{filename} must exist")
            profile = yaml.safe_load(profile_path.read_text(encoding="utf-8"))
            self.assertIsInstance(profile, dict)

            self.assertEqual(
                profile.get("schema_version"),
                "org.searchright.licensed-adapter.v1",
                f"{filename} must have canonical schema version",
            )

            provider_id = profile.get("provider_id")
            self.assertIsInstance(provider_id, str)
            self.assertTrue(provider_id.endswith("-byo"), f"{provider_id} must have -byo suffix")
            self.assertNotIn(provider_id, seen_providers, "provider_id must be unique")
            seen_providers.add(provider_id)

            database = profile.get("database")
            self.assertIsInstance(database, str)
            self.assertTrue(bool(database.strip()))

            platform = profile.get("platform")
            self.assertIsInstance(platform, str)
            self.assertTrue(bool(platform.strip()))

            dialect = profile.get("dialect")
            self.assertIn(dialect, {"embase", "scopus", "web_of_science"})

            cred_var = profile.get("credential_environment_variable")
            self.assertIsInstance(cred_var, str)
            self.assertTrue(ENV_VAR_PATTERN.fullmatch(cred_var))
            self.assertTrue(cred_var.startswith("SEARCHRIGHT_"))

            opt_in_var = profile.get("live_opt_in_environment_variable")
            self.assertIsInstance(opt_in_var, str)
            self.assertTrue(ENV_VAR_PATTERN.fullmatch(opt_in_var))
            self.assertTrue(opt_in_var.endswith("_LIVE"))

            allowed_hosts = profile.get("allowed_hosts")
            self.assertIsInstance(allowed_hosts, list)
            self.assertGreater(len(allowed_hosts), 0)
            for host in allowed_hosts:
                self.assertIsInstance(host, str)
                self.assertNotIn("://", host, "host must not include scheme")
                self.assertNotIn("/", host, "host must not include path")
                self.assertNotIn("@", host, "host must not include user credentials")
                self.assertNotIn(":", host, "host must not include port")
                self.assertTrue(DNS_HOST_PATTERN.fullmatch(host), f"{host} must be valid bare DNS hostname")

            export_formats = profile.get("export_formats")
            self.assertIsInstance(export_formats, list)
            self.assertGreater(len(export_formats), 0)
            for fmt in export_formats:
                self.assertIn(fmt, {"ris", "csv"})

            terms_review = profile.get("terms_review")
            self.assertIsInstance(terms_review, str)
            self.assertGreaterEqual(len(terms_review.strip()), 20)

            # Raw response persistence MUST be false
            self.assertIs(profile.get("persist_raw_responses"), False)

    def test_prohibition_of_scraping_and_bundled_access(self) -> None:
        index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
        for filename in index["profiles"]:
            profile_path = LICENSED_DIR / filename
            content = profile_path.read_text(encoding="utf-8")
            profile = yaml.safe_load(content)

            # Ensure no credentials or keys are hardcoded in the profile
            for sensitive_term in ("secret", "password", "api_key_value", "bearer_token", "private_key"):
                self.assertNotIn(sensitive_term, content.lower())

            # Ensure scraping is not permitted
            for scrap_term in ("scrape", "scraping", "headless", "puppeteer", "selenium"):
                self.assertNotIn(scrap_term, content.lower())

            # Allowed hosts must only be official vendor endpoints
            allowed_hosts = profile["allowed_hosts"]
            for host in allowed_hosts:
                self.assertTrue(
                    host in {"embase.com", "api.elsevier.com", "api.clarivate.com"},
                    f"{host} is not a recognised vendor endpoint",
                )

    def test_example_licensed_adapter_profile_conformance(self) -> None:
        self.assertTrue(EXAMPLE_PATH.is_file(), "contracts/examples/licensed-adapter.yaml must exist")
        example = yaml.safe_load(EXAMPLE_PATH.read_text(encoding="utf-8"))
        self.assertEqual(example.get("schema_version"), "org.searchright.licensed-adapter.v1")
        self.assertTrue(example.get("provider_id", "").endswith("-byo"))
        self.assertIs(example.get("persist_raw_responses"), False)
        self.assertTrue(ENV_VAR_PATTERN.fullmatch(example.get("credential_environment_variable", "")))
        self.assertTrue(ENV_VAR_PATTERN.fullmatch(example.get("live_opt_in_environment_variable", "")))

    def test_track_21_traceability_governance(self) -> None:
        self.assertTrue(TRACEABILITY_PATH.is_file(), "Track 21 traceability.json must exist")
        traceability = json.loads(TRACEABILITY_PATH.read_text(encoding="utf-8"))

        self.assertEqual(traceability.get("track_id"), "21")
        self.assertEqual(traceability.get("implementation_state"), "scaffolded")
        self.assertEqual(traceability.get("evidence_level"), "source_verified")

        assertions = traceability.get("assertions", [])
        self.assertEqual(len(assertions), 6, "Track 21 must define exactly 6 assertions")

        expected_ids = {f"T21-A{i:03d}" for i in range(1, 7)}
        actual_ids = {a["assertion_id"] for a in assertions}
        self.assertEqual(actual_ids, expected_ids)

        for assertion in assertions:
            self.assertEqual(assertion.get("state"), "scaffolded")
            self.assertEqual(assertion.get("mapping_confidence"), "symbol_and_gate_mapped")
            self.assertGreater(len(assertion.get("implementation_symbols", [])), 0)
            self.assertGreater(len(assertion.get("deterministic_tests", [])), 0)
            self.assertGreater(len(assertion.get("open_gates", [])), 0)
            for path in assertion.get("implementation_paths", []):
                self.assertTrue((ROOT / path).exists(), f"Mapped path {path} must exist on disk")


if __name__ == "__main__":
    unittest.main()
