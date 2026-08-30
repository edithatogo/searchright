"""Network-free regressions for exact, owner-approved Cargo Vet exceptions."""

from __future__ import annotations

import contextlib
import copy
import datetime as dt
import io
import json
import subprocess
import unittest
from unittest.mock import Mock, patch

import check_cargo_vet_governance as checker
import generate_cargo_vet_exemption_ledger as generator


class ApprovalDay(dt.date):
    @classmethod
    def today(cls):
        return cls(2026, 8, 30)


class CargoVetGovernanceTests(unittest.TestCase):
    def setUp(self):
        self.config = checker.load_toml(checker.STORE / "config.toml")
        self.ledger = checker.load_json(checker.STORE / "exemption-proposals.json")
        self.real_load_toml = checker.load_toml
        self.real_load_json = checker.load_json
        self.lock = checker.load_toml(checker.ROOT / "Cargo.lock")
        self.approval = checker.load_json(checker.ROOT / checker.TRACK06_RECEIPT)
        self.missing_approval = False
        self.metadata_error = None
        self.all_features_metadata = None
        self.metadata_calls = []
        package_id = "registry+https://github.com/rust-lang/crates.io-index#quick-xml@0.41.0"
        self.metadata = {
            "packages": [{"id": package_id, "name": "quick-xml", "version": "0.41.0"}],
            "resolve": {"nodes": [{"id": package_id, "features": ["default"]}]},
        }

    def proposal(self):
        return next(p for p in self.ledger["proposals"] if p["id"] == "CVX-0259")

    def run_check(self, date=ApprovalDay):
        def load_toml(path):
            if path == checker.STORE / "config.toml":
                return copy.deepcopy(self.config)
            if path == checker.ROOT / "Cargo.lock":
                return copy.deepcopy(self.lock)
            return self.real_load_toml(path)

        def load_json(path):
            if path == checker.STORE / "exemption-proposals.json":
                return copy.deepcopy(self.ledger)
            if path == checker.ROOT / checker.TRACK06_RECEIPT:
                if self.missing_approval:
                    raise FileNotFoundError(path)
                return copy.deepcopy(self.approval)
            return self.real_load_json(path)

        def metadata(command, **kwargs):
            self.assertEqual(command[:6], ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"])
            self.assertIn(command[6:], ([], ["--all-features"]))
            self.metadata_calls.append(command)
            self.assertEqual(kwargs["cwd"], checker.ROOT)
            self.assertTrue(kwargs["check"])
            if self.metadata_error:
                raise self.metadata_error
            resolved = self.all_features_metadata if command[6:] and self.all_features_metadata is not None else self.metadata
            return subprocess.CompletedProcess(command, 0, json.dumps(resolved), "")

        output = io.StringIO()
        with (
            patch.object(checker, "load_toml", side_effect=load_toml),
            patch.object(checker, "load_json", side_effect=load_json),
            patch.object(checker.dt, "date", date),
            patch.object(checker.subprocess, "run", side_effect=metadata),
            contextlib.redirect_stdout(output),
        ):
            result = checker.main()
        return result, json.loads(output.getvalue())

    def assert_rejected(self, text):
        result, receipt = self.run_check()
        self.assertEqual(result, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any(text in error for error in receipt["errors"]), receipt)

    def test_unapproved_peer_registry_is_rejected(self):
        self.config["imports"]["unapproved"] = {"url": "https://example.invalid/audits.toml"}
        self.assert_rejected("peer registry")

    def test_publisher_trust_is_rejected(self):
        self.config["trusted"] = {"quick-xml": [{"user-id": 1}]}
        self.assert_rejected("publisher trust")

    def test_baseline_notes_cannot_be_changed(self):
        self.config["exemptions"]["aho-corasick"][0]["notes"] = "approved"
        self.assert_rejected("governance notes")

    def test_baseline_rationale_cannot_be_changed(self):
        next(p for p in self.ledger["proposals"] if p["id"] == "CVX-0001")["rationale"] = "later blanket approval"
        self.assert_rejected("altered rationale")

    def test_duplicate_proposal_ids_are_rejected(self):
        self.ledger["proposals"].append(copy.deepcopy(self.ledger["proposals"][0]))
        self.assert_rejected("duplicate exemption proposal")

    def test_exact_approved_exception_and_baseline_pass(self):
        result, receipt = self.run_check()
        self.assertEqual(result, 0, receipt)
        self.assertEqual(receipt["effective_exemption_count"], 259)
        self.assertEqual(receipt["errors"], [])

    def test_baseline_without_track06_still_passes(self):
        self.ledger["proposals"] = [p for p in self.ledger["proposals"] if p["id"] != "CVX-0259"]
        del self.config["exemptions"]["quick-xml"]
        result, receipt = self.run_check()
        self.assertEqual(result, 0, receipt)
        self.assertEqual(receipt["effective_exemption_count"], 258)
        self.assertEqual(self.metadata_calls, [])

    def test_expired_exception_fails(self):
        class AfterExpiry(ApprovalDay):
            @classmethod
            def today(cls):
                return cls(2026, 9, 30)
        result, receipt = self.run_check(date=AfterExpiry)
        self.assertEqual(result, 1)
        self.assertIn("approved proposal CVX-0259 is expired", receipt["errors"])

    def test_expiry_cannot_be_extended(self):
        self.proposal()["expires_at"] = "2026-09-30"
        self.assert_rejected("review deadline")

    def test_exact_locked_identity_is_required(self):
        original = copy.deepcopy(self.lock)
        for field, value in (("version", "0.42.0"), ("checksum", "0" * 64), ("source", "registry+https://example.invalid")):
            with self.subTest(field=field):
                self.lock = copy.deepcopy(original)
                next(p for p in self.lock["package"] if p["name"] == "quick-xml")[field] = value
                self.assert_rejected("locked identity")

    def test_feature_changes_are_not_covered(self):
        for features in ([], ["default", "serialize"], ["default", "async-tokio"]):
            with self.subTest(features=features):
                self.metadata["resolve"]["nodes"][0]["features"] = features
                self.assert_rejected("resolved features")

    def test_metadata_unavailable_fails_closed(self):
        self.metadata_error = subprocess.CalledProcessError(1, ["cargo"])
        self.assert_rejected("cannot verify")

    def test_all_features_resolution_cannot_expand_approved_scope(self):
        self.all_features_metadata = copy.deepcopy(self.metadata)
        self.all_features_metadata["resolve"]["nodes"][0]["features"] = ["default", "serialize"]
        self.assert_rejected("resolved features")
        self.assertTrue(any("--all-features" in command for command in self.metadata_calls))

    def test_missing_metadata_resolution_fails_closed(self):
        self.metadata["resolve"] = None
        self.assert_rejected("cannot verify")

    def test_missing_dependency_resolution_fails_closed(self):
        self.metadata["resolve"]["nodes"] = []
        self.assert_rejected("resolved features")

    def test_receipt_identity_and_decision_cannot_drift(self):
        original = copy.deepcopy(self.approval)
        for field, value in (("decision", "pending"), ("owner", "agent"), ("version", "0.42.0"), ("checksum", "0" * 64), ("features", ["default", "serialize"]), ("expires_at", "2026-09-30"), ("criterion", "safe-to-run"), ("track_id", "20")):
            with self.subTest(field=field):
                self.approval = copy.deepcopy(original)
                self.approval[field] = value
                self.assert_rejected("receipt")

    def test_missing_approval_receipt_fails_closed(self):
        self.missing_approval = True
        self.assert_rejected("receipt")

    def test_approval_evidence_cannot_be_replaced(self):
        self.proposal()["decision_evidence"] = [checker.REQUIRED_ISSUE]
        self.assert_rejected("decision evidence")

    def test_approval_cannot_cover_another_dependency(self):
        self.proposal()["version"] = "0.42.0"
        self.assert_rejected("altered dependency")

    def test_pending_proposal_cannot_enable_exemption(self):
        self.proposal()["status"] = "not_authorized"
        self.assert_rejected("lack exact approvals")

    def test_track06_notes_cannot_reuse_baseline_approval(self):
        self.config["exemptions"]["quick-xml"][0]["notes"] = self.config["exemptions"]["aho-corasick"][0]["notes"]
        self.assert_rejected("governance notes")

    def test_track06_cannot_be_relabelled_as_baseline_approval(self):
        proposal = self.proposal()
        proposal.update({
            "id": "CVX-0260",
            "rationale": checker.REQUIRED_RATIONALE,
            "expires_at": "2026-11-10",
            "proposed_at": "2026-08-12T00:00:00+10:00",
            "decided_at": "2026-08-12T00:00:00+10:00",
            "decision_evidence": [checker.REQUIRED_ISSUE],
        })
        result, receipt = self.run_check()
        self.assertEqual(result, 1, receipt)
        self.assertTrue(any("quick-xml" in error for error in receipt["errors"]), receipt)

    def test_baseline_generator_refuses_to_overwrite_new_decision(self):
        ledger = Mock()
        ledger.exists.return_value = True
        ledger.read_text.return_value = json.dumps(self.ledger)
        config = Mock()
        with patch.object(generator, "LEDGER", ledger), patch.object(generator, "CONFIG", config):
            with self.assertRaisesRegex(SystemExit, "post-baseline owner decisions"):
                generator.main()
        ledger.write_text.assert_not_called()
        config.read_text.assert_not_called()
        config.write_text.assert_not_called()


if __name__ == "__main__":
    unittest.main()
