"""Unit tests for Authenticated Remote MCP, Tenancy, and Data Residency Governance (Track 34)."""

import json
from pathlib import Path
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]


class AuthenticatedRemoteMcpGovernanceTests(unittest.TestCase):
    def setUp(self):
        self.tenant_policy = json.loads(
            (ROOT / "contracts/examples/tenant-policy.json").read_text(encoding="utf-8")
        )
        self.access_request = json.loads(
            (ROOT / "contracts/examples/access-request.json").read_text(encoding="utf-8")
        )
        self.access_decision = json.loads(
            (ROOT / "contracts/examples/access-decision.json").read_text(encoding="utf-8")
        )
        self.auth_receipt = json.loads(
            (ROOT / "verification/receipts/remote-mcp-auth-conformance.json").read_text(encoding="utf-8")
        )
        self.tenancy_receipt = json.loads(
            (ROOT / "verification/receipts/remote-mcp-tenancy-adversarial.json").read_text(encoding="utf-8")
        )

    def test_tenant_policy_contract_and_boundary(self):
        """Verify tenant policy conforms to schema and enforces default-deny boundaries."""
        self.assertEqual(
            self.tenant_policy.get("schema_version"),
            "org.searchright.tenant-policy.v1",
        )
        self.assertEqual(self.tenant_policy.get("tenant_id"), "tenant-demo")
        self.assertIn("AU", self.tenant_policy.get("allowed_regions", []))
        self.assertIn("review_read", self.tenant_policy.get("allowed_scopes", []))
        self.assertGreaterEqual(self.tenant_policy.get("maximum_concurrent_tasks", 0), 1)

        # Default-deny safe defaults
        self.assertFalse(self.tenant_policy.get("external_model_processing_allowed", True))
        self.assertFalse(self.tenant_policy.get("restricted_full_text_persistence_allowed", True))
        self.assertFalse(self.tenant_policy.get("cross_tenant_aggregation_allowed", True))
        self.assertTrue(bool(self.tenant_policy.get("approved_by")))

    def test_access_request_and_decision_contracts(self):
        """Verify access request and decision schemas and default-deny structures."""
        self.assertEqual(
            self.access_request.get("schema_version"),
            "org.searchright.access-request.v1",
        )
        self.assertEqual(
            self.access_decision.get("schema_version"),
            "org.searchright.access-decision.v1",
        )
        self.assertEqual(self.access_request.get("tenant_id"), self.access_decision.get("tenant_id"))
        self.assertEqual(self.access_request.get("request_id"), self.access_decision.get("request_id"))
        self.assertTrue(self.access_decision.get("permitted"))
        self.assertEqual(self.access_decision.get("blockers"), [])

    def test_remote_mcp_security_architecture_spec(self):
        """Verify security architecture documentation enforces loopback, TLS, and read-only surfaces."""
        doc = (ROOT / "docs/security/authenticated-remote-mcp.md").read_text(encoding="utf-8")
        self.assertIn("SEARCHRIGHT_REMOTE_MCP_ENABLED=1", doc)
        self.assertIn("loopback", doc.lower())
        self.assertIn("tls", doc.lower())
        self.assertIn("jwks", doc.lower())
        self.assertIn("read-only", doc.lower())
        self.assertIn("replay", doc.lower())
        self.assertIn("audit", doc.lower())

        threat_doc = (ROOT / "docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn("remote mcp", threat_doc.lower())
        self.assertIn("tenant", threat_doc.lower())

    def test_remote_mcp_conformance_receipts(self):
        """Verify deterministic remote MCP auth and tenancy conformance receipts pass all controls."""
        auth_command = self.auth_receipt.get("command", {})
        self.assertEqual(auth_command.get("returncode"), 0)
        auth_controls = {c["control"]: c["status"] for c in self.auth_receipt.get("controls", [])}
        for expected in ("issuer", "principal", "replay", "approval", "transport", "audit", "request_budget"):
            self.assertEqual(auth_controls.get(expected), "passed", f"Auth control {expected} must pass")

        tenancy_command = self.tenancy_receipt.get("command", {})
        self.assertEqual(tenancy_command.get("returncode"), 0)
        tenancy_controls = {c["control"]: c["status"] for c in self.tenancy_receipt.get("controls", [])}
        for expected in ("tenant", "region", "scope", "rate", "concurrency"):
            self.assertEqual(tenancy_controls.get(expected), "passed", f"Tenancy control {expected} must pass")
        self.assertTrue(
            self.tenancy_receipt.get("adversarial_cases", {})
            .get("multiple_violations_accumulate", {})
            .get("observed")
        )

    def test_non_human_agent_exclusion_prohibition(self):
        """Verify final eligibility decisions require human principal and cannot be made autonomously by agents."""
        # Principal kind in access request must be human for final decisions
        self.assertEqual(self.access_request.get("principal_kind"), "human")
        self.assertTrue(self.access_request.get("final_eligibility_decision"))
        self.assertTrue(self.access_request.get("human_approval"))

        # In Rust access tests, final_eligibility_decision_by_non_human_is_denied passed
        auth_tests = []
        for c in self.auth_receipt.get("controls", []):
            auth_tests.extend(c.get("tests", []))
        self.assertIn("final_eligibility_decision_by_non_human_is_denied", auth_tests)

    def test_openapi_spec_remote_surface(self):
        """Verify OpenAPI specification defines future approval-gated HTTP surface over core contracts."""
        openapi_path = ROOT / "contracts/openapi/searchright-http.openapi.yaml"
        self.assertTrue(openapi_path.exists())
        spec = yaml.safe_load(openapi_path.read_text(encoding="utf-8"))
        self.assertEqual(spec.get("openapi"), "3.1.0")
        paths = spec.get("paths", {})
        self.assertIn("/v1/reviews/{review_id}/compile", paths)
        self.assertIn("/v1/reviews/{review_id}/audit/verify", paths)


if __name__ == "__main__":
    unittest.main()
