"""Tests for Track 28: Institutional governance, privacy and collaboration.

These are network-free, source-level checks. They do not execute the Rust
governance engine; compiler-backed negative scenarios remain an open gate.

Validates:
1. Institutional policy, request and decision examples are mutually consistent.
2. Data classification, retention and region rules fail closed in source.
3. Receipt minimisation redacts secrets and query-bearing URLs deterministically.
4. Human approval and least-privilege lifecycle rules exist in source.
5. Multi-reviewer adjudication independence and the federated threat model.
"""
from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from redaction import load_profile, redact_url, redact_value  # noqa: E402

EXAMPLES = ROOT / "contracts" / "examples"
GOVERNANCE_SRC = ROOT / "crates" / "searchright-governance" / "src" / "lib.rs"
APPROVAL_SRC = ROOT / "crates" / "searchright-governance" / "src" / "approval.rs"
SCREENING_SRC = ROOT / "crates" / "searchright-screening" / "src" / "lib.rs"
THREAT_MODEL = ROOT / "docs" / "security" / "threat-model.md"


def load_example(name: str) -> dict:
    return yaml.safe_load((EXAMPLES / name).read_text(encoding="utf-8"))


class InstitutionalGovernancePrivacyCollaborationTests(unittest.TestCase):
    def test_policy_request_decision_examples_are_consistent(self) -> None:
        policy = load_example("institutional-policy.yaml")
        request = load_example("data-handling-request.yaml")
        decision = load_example("data-handling-decision.yaml")

        self.assertEqual(decision["policy_id"], policy["policy_id"])
        self.assertEqual(decision["request_id"], request["request_id"])
        self.assertIn(request["deployment_mode"], policy["deployment_modes"])
        self.assertIn(request["classification"], policy["allowed_classifications"])
        self.assertLessEqual(request["retention_days"], policy["maximum_retention_days"])
        self.assertIn(request["region"], policy["permitted_regions"])
        self.assertFalse(policy["telemetry_allowed"])
        self.assertFalse(policy["external_model_processing_allowed"])
        self.assertFalse(policy["cross_border_transfer_allowed"])
        # A dry-run on restricted full text may be permitted but never self-approves.
        self.assertTrue(request["dry_run"])
        self.assertTrue(decision["permitted"])
        self.assertTrue(decision["human_approval_required"])
        self.assertIn("governance.dry_run.no_effects", decision["warnings"])

    def test_governance_source_denies_by_default_with_stable_codes(self) -> None:
        source = GOVERNANCE_SRC.read_text(encoding="utf-8")
        for code in (
            "governance.deployment_mode.denied",
            "governance.classification.denied",
            "governance.retention.exceeds_policy",
            "governance.telemetry.denied",
            "governance.full_text_persistence.denied",
            "governance.external_model.denied",
            "governance.cross_border_transfer.denied",
            "governance.region.denied",
            "governance.region.required",
            "governance.multi_tenant.isolation_unproven",
            "governance.export.approval_not_recorded",
        ):
            with self.subTest(code=code):
                self.assertIn(f'"{code}"', source)
        # Decisions are only permitted when no blocker was recorded.
        self.assertIn("let permitted = blockers.is_empty();", source)

    def test_human_approval_required_for_high_risk_classifications(self) -> None:
        source = GOVERNANCE_SRC.read_text(encoding="utf-8")
        self.assertIn('"governance.human_approval.not_recorded"', source)
        for classification in ("Confidential", "RestrictedFullText", "SensitivePersonalData"):
            with self.subTest(classification=classification):
                self.assertIn(f"DataClassification::{classification}", source)
        self.assertIn("human_approval_required: !permitted || requires_human_approval", source)

    def test_lifecycle_authority_is_scoped_and_audit_deletion_denied(self) -> None:
        source = GOVERNANCE_SRC.read_text(encoding="utf-8")
        approval = APPROVAL_SRC.read_text(encoding="utf-8")
        for symbol in (
            "pub fn evaluate_lifecycle(",
            "pub fn evaluate_lifecycle_with_verifier(",
            "pub fn authorize_lifecycle(",
            "pub fn execute_lifecycle(",
            "pub trait LifecycleApprovalVerifier",
            "pub struct LifecycleAuthorization",
        ):
            with self.subTest(symbol=symbol):
                self.assertIn(symbol, source)
        self.assertIn("approval record does not exactly bind principal, policy and request", approval)
        self.assertIn("approval is not currently valid", approval)
        self.assertIn("approval registry exceeds configured bound", approval)
        # LifecycleAuthorization fields stay private so authority cannot be forged.
        start = source.index("pub struct LifecycleAuthorization {")
        body = source[start : source.index("}", start)]
        self.assertNotIn("pub request", body)
        self.assertNotIn("pub decision", body)

    def test_receipt_minimisation_is_deterministic_and_removes_secrets(self) -> None:
        profile = load_profile(ROOT / "policy" / "redaction-profile.json")
        secret = "A1b2C3d4E5f6G7h8I9j0K1L2M3N4"
        payload = {
            "endpoint": f"https://user:{secret}@api.example.test/search?db=pubmed&query=rare+disease&api_key={secret}&retmax=20",
            "authorization": f"Bearer {secret}",
            "contact": "researcher@example.org",
        }
        redacted = redact_value(payload, profile)
        rendered = json.dumps(redacted, sort_keys=True)
        for forbidden in (secret, "rare+disease", "researcher@example.org", "user:"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, rendered)
        self.assertEqual(redact_value(payload, profile), redacted)
        url = redact_url(payload["endpoint"], profile)
        self.assertIn("db=pubmed", url)
        self.assertIn("retmax=20", url)

    def test_multi_reviewer_adjudication_independence_in_source(self) -> None:
        source = SCREENING_SRC.read_text(encoding="utf-8")
        for symbol in (
            "ReviewerRole::Adjudicator",
            "ScreeningError::AdjudicatorNotIndependent",
            "ScreeningError::IncompleteResolution",
            "ScreeningError::RoundFinalized",
            "ScreeningError::NoConflict",
            "ScreeningError::UnauthorizedRole",
            "pub fn resolve(",
            "pub fn confirm_agent_exclusion(",
        ):
            with self.subTest(symbol=symbol):
                self.assertIn(symbol, source)
        # Agent decisions must not count as known human decisions during adjudication.
        self.assertIn("decision.reviewer_kind != ReviewerKind::Agent", source)

    def test_federated_threat_model_keeps_gates_open(self) -> None:
        text = THREAT_MODEL.read_text(encoding="utf-8")
        for threat in ("Agent overreach", "Data exfiltration through telemetry", "Audit tampering"):
            with self.subTest(threat=threat):
                self.assertIn(threat, text)
        self.assertIn("external security review", text)
        evidence = json.loads(
            (ROOT / "conductor/tracks/28-institutional-governance-privacy-collaboration/evidence.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(evidence["evidence_level"], "source_verified")
        self.assertNotEqual(evidence["status"], "complete")


if __name__ == "__main__":
    unittest.main()
