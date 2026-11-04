"""Unit tests for Track 25 Provenance and research-object interoperability governance."""
from __future__ import annotations

import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import review_bundle  # noqa: E402

REVIEW_BUNDLE_MANIFEST_SCHEMA = ROOT / "contracts" / "json-schema" / "review-bundle-manifest.v1.schema.json"
REVIEW_BUNDLE_MANIFEST_EXAMPLE = ROOT / "contracts" / "examples" / "review-bundle-manifest.json"
HANDOFF_PLAN_SCHEMA = ROOT / "contracts" / "json-schema" / "research-object-handoff-plan.v1.schema.json"
HANDOFF_PLAN_EXAMPLE = ROOT / "contracts" / "examples" / "research-object-handoff-plan.json"
EXAMPLE_SRPACK = ROOT / "examples" / "review-bundle" / "example.srpack"
TRACEABILITY_PATH = ROOT / "conductor" / "tracks" / "25-provenance-research-objects" / "traceability.json"
STANDARDS_MAP = ROOT / "docs" / "standards-and-provenance.md"
PROVENANCE_LIB = ROOT / "crates" / "searchright-provenance" / "src" / "lib.rs"


class ProvenanceResearchObjectsGovernanceTests(unittest.TestCase):
    def test_ro_crate_schema_and_context(self) -> None:
        """Verify RO-Crate 1.3 schema and context representations."""
        self.assertTrue(PROVENANCE_LIB.is_file(), f"{PROVENANCE_LIB} must exist")
        lib_source = PROVENANCE_LIB.read_text(encoding="utf-8")

        # Context and conformance URIs for RO-Crate 1.3
        self.assertIn("https://w3id.org/ro/crate/1.3/context", lib_source)
        self.assertIn("https://w3id.org/ro/crate/1.3", lib_source)
        self.assertIn("ro-crate-metadata.json", lib_source)

        # Standards and provenance documentation mapping
        self.assertTrue(STANDARDS_MAP.is_file(), f"{STANDARDS_MAP} must exist")
        standards_doc = STANDARDS_MAP.read_text(encoding="utf-8")
        self.assertIn("RO-Crate", standards_doc)
        self.assertIn("W3C PROV", standards_doc)

    def test_prov_entities_activities_agents_predecessors(self) -> None:
        """Verify W3C PROV entities, activities, agents, and predecessor derivations."""
        self.assertTrue(PROVENANCE_LIB.is_file())
        lib_source = PROVENANCE_LIB.read_text(encoding="utf-8")

        # W3C PROV namespaces
        self.assertIn("http://www.w3.org/ns/prov#", lib_source)
        self.assertIn("https://schemas.searchright.dev/prov/", lib_source)

        # W3C PROV vocabulary mappings
        self.assertIn("prov:type", lib_source)
        self.assertIn("prov:startedAtTime", lib_source)
        self.assertIn("prov:generatedAtTime", lib_source)
        self.assertIn("wasAssociatedWith", lib_source)
        self.assertIn("wasDerivedFrom", lib_source)
        self.assertIn("prov:generatedEntity", lib_source)
        self.assertIn("prov:usedEntity", lib_source)

        # Predecessor derivation error variant
        self.assertIn("UnresolvedPredecessor", lib_source)
        self.assertIn("ReviewMismatch", lib_source)
        self.assertIn("DuplicateIdentifier", lib_source)

    def test_review_bundle_packaging_invariants(self) -> None:
        """Verify review bundle manifest schema and example.srpack packaging invariants."""
        self.assertTrue(REVIEW_BUNDLE_MANIFEST_SCHEMA.is_file())
        self.assertTrue(REVIEW_BUNDLE_MANIFEST_EXAMPLE.is_file())
        self.assertTrue(EXAMPLE_SRPACK.is_file())

        schema = json.loads(REVIEW_BUNDLE_MANIFEST_SCHEMA.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)

        example = json.loads(REVIEW_BUNDLE_MANIFEST_EXAMPLE.read_text(encoding="utf-8"))
        validator = Draft202012Validator(schema)
        errors = list(validator.iter_errors(example))
        self.assertEqual(errors, [], f"example review-bundle-manifest must validate: {errors}")

        # Verify example.srpack archive and manifest
        with zipfile.ZipFile(EXAMPLE_SRPACK, "r") as archive:
            self.assertIn("manifest.json", archive.namelist())
            manifest_bytes = archive.read("manifest.json")
            manifest = json.loads(manifest_bytes.decode("utf-8"))

            manifest_errors = list(validator.iter_errors(manifest))
            self.assertEqual(manifest_errors, [], f"manifest.json in srpack must validate: {manifest_errors}")

            # Verify deterministic review bundle policy invariants
            policy = manifest["policy"]
            self.assertIs(policy["deterministic"], True)
            self.assertIs(policy["network_required"], False)
            self.assertIs(policy["symlinks_allowed"], False)
            self.assertIs(policy["external_writes_allowed"], False)
            self.assertIs(policy["secret_scan_required"], True)

            # Verify entries and payload hashes
            entry_digests: list[str] = []
            for entry in manifest["entries"]:
                archive_path = entry["archive_path"]
                self.assertTrue(archive_path.startswith("payload/"))
                self.assertIn(archive_path, archive.namelist())

                payload_data = archive.read(archive_path)
                self.assertEqual(len(payload_data), entry["size"])
                actual_hash = review_bundle.sha256_bytes(payload_data)
                self.assertEqual(actual_hash, entry["sha256"])
                entry_digests.append(entry["sha256"])

            # Verify descriptor merkle root calculation across canonical manifest entry descriptors
            descriptor_digests = [
                review_bundle.sha256_bytes(review_bundle.canonical_json_bytes(entry))
                for entry in manifest["entries"]
            ]
            expected_merkle = review_bundle.merkle_root(descriptor_digests)
            self.assertEqual(expected_merkle, manifest["descriptor_merkle_root"])

        # Verify bundle self-verification using review_bundle.verify
        verify_receipt = review_bundle.verify(EXAMPLE_SRPACK)
        self.assertEqual(verify_receipt.get("status"), "passed")
        self.assertEqual(verify_receipt.get("errors"), [])

        # Verify self-test runs cleanly
        self_test_receipt = review_bundle.self_test()
        self.assertEqual(self_test_receipt.get("status"), "passed")

    def test_review_bundle_safety_guards(self) -> None:
        """Verify review bundle safety guards against secrets, traversal, and tampering."""
        # High risk secrets must be rejected
        with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as temp_secret:
            temp_secret.write(b"client_secret = 'simulated-unauthorized-secret-token'")
            temp_secret_path = Path(temp_secret.name)

        try:
            with self.assertRaises(review_bundle.BundleError):
                review_bundle.scan_for_secrets(temp_secret_path, temp_secret_path.read_bytes())
        finally:
            temp_secret_path.unlink(missing_ok=True)

        # Unsafe destinations must be rejected
        for unsafe in ("../outside", "payload/../../root", ".git/config", ""):
            with self.assertRaises(review_bundle.BundleError):
                review_bundle.safe_destination(unsafe)

    def test_research_object_handoff_plan_governance(self) -> None:
        """Verify research object handoff plan schema, example, and default-deny invariants."""
        self.assertTrue(HANDOFF_PLAN_SCHEMA.is_file())
        self.assertTrue(HANDOFF_PLAN_EXAMPLE.is_file())

        schema = json.loads(HANDOFF_PLAN_SCHEMA.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)

        example = json.loads(HANDOFF_PLAN_EXAMPLE.read_text(encoding="utf-8"))
        validator = Draft202012Validator(schema)
        errors = list(validator.iter_errors(example))
        self.assertEqual(errors, [], f"handoff-plan example must validate: {errors}")

        # Governance invariants: no automated deposit, dry-run only
        self.assertEqual(example.get("execution_mode"), "dry_run")
        self.assertIs(example.get("deposit_authorized"), False)
        self.assertIs(example.get("ro_crate_conformance_claimed"), False)
        self.assertIs(example.get("osf_acceptance_claimed"), False)
        self.assertEqual(example.get("delegated_export_track"), "25")

    def test_track_25_traceability_governance(self) -> None:
        """Verify Track 25 traceability assertions and mapping confidence."""
        self.assertTrue(TRACEABILITY_PATH.is_file(), "Track 25 traceability.json must exist")
        traceability = json.loads(TRACEABILITY_PATH.read_text(encoding="utf-8"))

        self.assertEqual(traceability.get("track_id"), "25")
        self.assertEqual(traceability.get("implementation_state"), "partially_implemented")
        self.assertEqual(traceability.get("evidence_level"), "source_verified")

        assertions = traceability.get("assertions", [])
        self.assertEqual(len(assertions), 6, "Track 25 must define exactly 6 assertions")

        expected_ids = {f"T25-A{i:03d}" for i in range(1, 7)}
        actual_ids = {a["assertion_id"] for a in assertions}
        self.assertEqual(actual_ids, expected_ids)

        for assertion in assertions:
            self.assertEqual(assertion.get("state"), "partially_implemented")
            self.assertEqual(assertion.get("mapping_confidence"), "symbol_and_gate_mapped")
            self.assertGreater(len(assertion.get("implementation_symbols", [])), 0)
            self.assertGreater(len(assertion.get("deterministic_tests", [])), 0)
            self.assertGreater(len(assertion.get("native_test_commands", [])), 0)
            self.assertGreater(len(assertion.get("evidence_receipts", [])), 0)
            self.assertGreater(len(assertion.get("open_gates", [])), 0)
            self.assertTrue(bool(assertion.get("permitted_claim", "").strip()))
            for path in assertion.get("implementation_paths", []):
                self.assertTrue((ROOT / path).exists(), f"Mapped path {path} must exist on disk")


if __name__ == "__main__":
    unittest.main()
