"""Unit tests for Track 24 WASI components, HTTP MCP, and scalable execution governance."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
OPENAPI_SPEC_PATH = ROOT / "contracts" / "openapi" / "searchright-http.openapi.yaml"
WIT_SPEC_PATH = ROOT / "contracts" / "wit" / "search-provider.wit"
PROVIDER_COMPONENT_EXAMPLE = ROOT / "contracts" / "examples" / "provider-component.yaml"
PROVIDER_COMPONENT_SCHEMA = ROOT / "contracts" / "json-schema" / "provider-component.v1.schema.json"
RELEASE_SIGNATURE_SCHEMA = ROOT / "contracts" / "json-schema" / "provider-component-release-signature.v1.schema.json"
TRUST_POLICY_SCHEMA = ROOT / "contracts" / "json-schema" / "provider-component-trust-policy.v1.schema.json"
TRACEABILITY_PATH = ROOT / "conductor" / "tracks" / "24-wasi-http-scale" / "traceability.json"


class WasiHttpScaleGovernanceTests(unittest.TestCase):
    def test_openapi_spec_structure_and_endpoints(self) -> None:
        self.assertTrue(OPENAPI_SPEC_PATH.is_file(), f"{OPENAPI_SPEC_PATH} must exist")
        raw = yaml.safe_load(OPENAPI_SPEC_PATH.read_text(encoding="utf-8"))

        self.assertEqual(raw.get("openapi"), "3.1.0")
        info = raw.get("info", {})
        self.assertEqual(info.get("title"), "Searchright HTTP API")
        self.assertEqual(info.get("version"), "0.1.0-alpha.1")

        paths = raw.get("paths", {})
        self.assertIn("/v1/reviews/{review_id}/compile", paths)
        self.assertIn("/v1/reviews/{review_id}/audit/verify", paths)

        compile_op = paths["/v1/reviews/{review_id}/compile"].get("post", {})
        self.assertEqual(compile_op.get("operationId"), "compileStrategy")
        params = compile_op.get("parameters", [])
        self.assertTrue(any(p.get("name") == "review_id" and p.get("in") == "path" for p in params))
        responses = compile_op.get("responses", {})
        self.assertIn("200", responses)
        self.assertIn("422", responses)

        verify_op = paths["/v1/reviews/{review_id}/audit/verify"].get("post", {})
        self.assertEqual(verify_op.get("operationId"), "verifyAudit")
        params = verify_op.get("parameters", [])
        self.assertTrue(any(p.get("name") == "review_id" and p.get("in") == "path" for p in params))
        self.assertIn("200", verify_op.get("responses", {}))

    def test_wit_search_provider_contract(self) -> None:
        self.assertTrue(WIT_SPEC_PATH.is_file(), f"{WIT_SPEC_PATH} must exist")
        content = WIT_SPEC_PATH.read_text(encoding="utf-8")

        self.assertIn("package searchright:provider@0.1.0;", content)
        self.assertIn("interface types {", content)
        self.assertIn("record compiled-strategy {", content)
        self.assertIn("record execution-policy {", content)
        self.assertIn("record search-request {", content)
        self.assertIn("record record-envelope {", content)
        self.assertIn("record provider-page {", content)
        self.assertIn("variant provider-error {", content)
        self.assertIn("interface search-provider {", content)
        self.assertIn("manifest: func() -> string;", content)
        self.assertIn("execute: func(request: search-request) -> result<provider-page, provider-error>;", content)
        self.assertIn("world provider-component {", content)
        self.assertIn("export search-provider;", content)

    def test_provider_component_conforms_to_schema_and_invariants(self) -> None:
        self.assertTrue(PROVIDER_COMPONENT_EXAMPLE.is_file())
        self.assertTrue(PROVIDER_COMPONENT_SCHEMA.is_file())

        example = yaml.safe_load(PROVIDER_COMPONENT_EXAMPLE.read_text(encoding="utf-8"))
        schema = json.loads(PROVIDER_COMPONENT_SCHEMA.read_text(encoding="utf-8"))

        validator = Draft202012Validator(schema)
        errors = list(validator.iter_errors(example))
        self.assertEqual(errors, [], f"provider-component.yaml must conform to schema: {errors}")

        self.assertEqual(example.get("schema_version"), "org.searchright.provider-component.v1")
        self.assertEqual(example.get("abi_version"), "searchright:provider/search-provider@0.1.0")
        self.assertEqual(example.get("component_id"), "io.github.edithatogo.searchright.fixture")

        capabilities = example.get("capabilities", [])
        self.assertIn("search", capabilities)
        self.assertIn("metadata_read", capabilities)

        self.assertGreater(example.get("max_memory_mib", 0), 0)
        self.assertGreater(example.get("max_fuel", 0), 0)
        self.assertIs(example.get("fixture_mode"), True)
        self.assertTrue(bool(example.get("component_digest")))

    def test_component_trust_schemas_are_valid(self) -> None:
        self.assertTrue(RELEASE_SIGNATURE_SCHEMA.is_file())
        self.assertTrue(TRUST_POLICY_SCHEMA.is_file())

        sig_schema = json.loads(RELEASE_SIGNATURE_SCHEMA.read_text(encoding="utf-8"))
        policy_schema = json.loads(TRUST_POLICY_SCHEMA.read_text(encoding="utf-8"))

        Draft202012Validator.check_schema(sig_schema)
        Draft202012Validator.check_schema(policy_schema)

        self.assertEqual(sig_schema.get("$id"), "https://schemas.searchright.dev/provider-component-release-signature.v1.schema.json")
        self.assertEqual(policy_schema.get("$id"), "https://schemas.searchright.dev/provider-component-trust-policy.v1.schema.json")

    def test_track_24_traceability_governance(self) -> None:
        self.assertTrue(TRACEABILITY_PATH.is_file(), "Track 24 traceability.json must exist")
        traceability = json.loads(TRACEABILITY_PATH.read_text(encoding="utf-8"))

        self.assertEqual(traceability.get("track_id"), "24")
        self.assertEqual(traceability.get("implementation_state"), "source_implemented")
        self.assertEqual(traceability.get("evidence_level"), "fixture_proven")

        assertions = traceability.get("assertions", [])
        expected_ids = {f"T24-A{i:03d}" for i in range(1, 7)}
        actual_ids = {a["assertion_id"] for a in assertions}
        self.assertEqual(actual_ids, expected_ids)

        trust_path = "verification/receipts/track-24-component-trust.json"
        trust = json.loads((ROOT / trust_path).read_text(encoding="utf-8"))
        self.assertEqual(trust["status"], "passed")
        self.assertTrue(any(
            result.get("status") == "passed"
            and "cargo test" in result.get("command", "")
            and "-p searchright-plugin-sdk" in result["command"]
            for result in trust["commands"]
        ))
        mcp_path = "verification/receipts/track-24-advanced-mcp.json"
        mcp = json.loads((ROOT / mcp_path).read_text(encoding="utf-8"))
        self.assertTrue(any(
            "test -p searchright-mcp" in result["command"] and "passed" in result["result"]
            for result in mcp["passed"]
        ))
        by_id = {assertion["assertion_id"]: assertion for assertion in assertions}
        self.assertIn(trust_path, by_id["T24-A001"]["evidence_receipts"])
        self.assertIn(mcp_path, by_id["T24-A004"]["evidence_receipts"])
        # Local cryptographic admission and stdio MCP do not prove these capabilities.
        external_boundaries = {
            "T24-A001": ("publisher identity", "key custody"),
            "T24-A002": ("wasmtime",),
            "T24-A003": ("arrow", "duckdb"),
            "T24-A004": ("streamable http", "remote"),
            "T24-A005": ("oauth",),
            "T24-A006": ("wasi", "remote"),
        }

        for assertion in assertions:
            self.assertEqual(assertion.get("state"), "source_implemented")
            self.assertEqual(assertion.get("mapping_confidence"), "symbol_and_gate_mapped")
            self.assertGreater(len(assertion.get("implementation_symbols", [])), 0)
            self.assertGreater(len(assertion.get("deterministic_tests", [])), 0)
            self.assertGreater(len(assertion.get("native_test_commands", [])), 0)
            self.assertGreater(len(assertion.get("evidence_receipts", [])), 0)
            gates = " ".join(assertion.get("open_gates", [])).lower()
            for boundary in external_boundaries[assertion["assertion_id"]]:
                self.assertIn(boundary, gates, assertion["assertion_id"])
            self.assertTrue(bool(assertion.get("permitted_claim", "").strip()))
            for path in assertion.get("implementation_paths", []):
                self.assertTrue((ROOT / path).exists(), f"Mapped path {path} must exist on disk")


if __name__ == "__main__":
    unittest.main()
