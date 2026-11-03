"""Unit tests for Track 22 living updates, amendments, and lineage governance."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
LIVING_UPDATE_EXAMPLE = ROOT / "contracts" / "examples" / "living-update.yaml"
PROTOCOL_AMENDMENT_EXAMPLE = ROOT / "contracts" / "examples" / "protocol-amendment.yaml"
LIVING_UPDATE_SCHEMA = ROOT / "contracts" / "json-schema" / "living-update.v1.schema.json"
PROTOCOL_AMENDMENT_SCHEMA = ROOT / "contracts" / "json-schema" / "protocol-amendment.v1.schema.json"
PRISMA_LSR_PACK = ROOT / "contracts" / "standards" / "packs" / "prisma-lsr-2024.yaml"
TRACEABILITY_PATH = ROOT / "conductor" / "tracks" / "22-living-updates" / "traceability.json"


class LivingUpdatesGovernanceTests(unittest.TestCase):
    def test_living_update_example_conforms_to_json_schema(self) -> None:
        self.assertTrue(LIVING_UPDATE_EXAMPLE.is_file(), "contracts/examples/living-update.yaml must exist")
        self.assertTrue(LIVING_UPDATE_SCHEMA.is_file(), "contracts/json-schema/living-update.v1.schema.json must exist")

        example = yaml.safe_load(LIVING_UPDATE_EXAMPLE.read_text(encoding="utf-8"))
        schema = json.loads(LIVING_UPDATE_SCHEMA.read_text(encoding="utf-8"))

        validator = Draft202012Validator(schema)
        errors = list(validator.iter_errors(example))
        self.assertEqual(errors, [], f"living-update.yaml must validate against schema: {errors}")

    def test_living_update_invariants_and_human_release_safeguard(self) -> None:
        example = yaml.safe_load(LIVING_UPDATE_EXAMPLE.read_text(encoding="utf-8"))

        self.assertEqual(example.get("schema_version"), "org.searchright.living-update.v1")
        self.assertEqual(example.get("status"), "completed")
        self.assertIsNotNone(example.get("completed_at"))
        self.assertNotEqual(example.get("run_id"), example.get("parent_run_id"))
        self.assertNotEqual(example.get("run_id"), example.get("supersedes_run_id"))

        # Invariant: human release is strictly required when records have changed
        changes = example.get("changes", [])
        self.assertGreater(len(changes), 0, "Example update run must contain record changes")
        self.assertIs(
            example.get("requires_human_release"),
            True,
            "Living update record changes must require explicit human release before downstream screening",
        )

        # Invariant: cursors before and after must define valid strategy and values
        for cursor_key in ("cursors_before", "cursors_after"):
            cursors = example.get(cursor_key, [])
            self.assertGreater(len(cursors), 0)
            for cursor in cursors:
                self.assertIn("provider_id", cursor)
                self.assertIn("cursor_kind", cursor)
                self.assertIn("value", cursor)
                self.assertIn(cursor["cursor_kind"], {"date", "cursor", "high_water_mark", "snapshot"})

        # Invariant: each record change must contain at least one content digest
        for change in changes:
            self.assertIn(change.get("kind"), {"added", "updated", "missing_from_source", "merged", "restored"})
            has_digest = bool(change.get("before_digest") or change.get("after_digest"))
            self.assertTrue(has_digest, f"Record change {change.get('record_id')} must contain at least one content digest")

    def test_protocol_amendment_example_conforms_to_json_schema(self) -> None:
        self.assertTrue(PROTOCOL_AMENDMENT_EXAMPLE.is_file(), "contracts/examples/protocol-amendment.yaml must exist")
        self.assertTrue(
            PROTOCOL_AMENDMENT_SCHEMA.is_file(), "contracts/json-schema/protocol-amendment.v1.schema.json must exist"
        )

        example = yaml.safe_load(PROTOCOL_AMENDMENT_EXAMPLE.read_text(encoding="utf-8"))
        schema = json.loads(PROTOCOL_AMENDMENT_SCHEMA.read_text(encoding="utf-8"))

        validator = Draft202012Validator(schema)
        errors = list(validator.iter_errors(example))
        self.assertEqual(errors, [], f"protocol-amendment.yaml must validate against schema: {errors}")

    def test_protocol_amendment_invariants_and_decision_safeguards(self) -> None:
        example = yaml.safe_load(PROTOCOL_AMENDMENT_EXAMPLE.read_text(encoding="utf-8"))

        self.assertEqual(example.get("schema_version"), "org.searchright.protocol-amendment.v1")
        self.assertNotEqual(
            example.get("version_before"),
            example.get("version_after"),
            "Protocol amendment must advance protocol version",
        )

        # Decision metadata invariants
        decision = example.get("decision")
        self.assertIn(decision, {"proposed", "approved", "rejected", "withdrawn"})
        if decision in {"approved", "rejected", "withdrawn"}:
            self.assertTrue(bool(example.get("decided_by")), "Decided amendment must record decided_by")
            self.assertTrue(bool(example.get("decided_at")), "Decided amendment must record decided_at")
        else:
            self.assertIsNone(example.get("decided_by"))
            self.assertIsNone(example.get("decided_at"))

        # Changes invariants
        changes = example.get("changes", [])
        self.assertGreater(len(changes), 0, "Amendment must record at least one change")
        paths: set[str] = set()
        for change in changes:
            self.assertTrue(bool(change.get("path")), "Change must define non-empty JSON pointer/path")
            self.assertNotIn(change["path"], paths, "Duplicate change paths are forbidden")
            paths.add(change["path"])
            self.assertIn("after", change, "Change must define after replacement value")
            self.assertTrue(bool(change.get("rationale")), "Change must record explicit rationale")

        self.assertTrue(bool(example.get("retrospective_impact")), "Retrospective impact must be evaluated")
        self.assertIsInstance(example.get("requires_reprocessing"), bool)

    def test_prisma_lsr_reporting_contract_boundaries(self) -> None:
        self.assertTrue(PRISMA_LSR_PACK.is_file(), "contracts/standards/packs/prisma-lsr-2024.yaml must exist")
        pack = yaml.safe_load(PRISMA_LSR_PACK.read_text(encoding="utf-8"))

        self.assertEqual(pack.get("schema_version"), "org.searchright.standard-pack.v1")
        self.assertEqual(pack.get("pack_id"), "prisma-lsr-2024")
        self.assertEqual(pack.get("family"), "prisma_lsr")

        # Non-negotiable rule: PRISMA-S/PRISMA-LSR must NEVER be represented as conduct standards
        items = pack.get("items", [])
        self.assertGreater(len(items), 0)
        for item in items:
            self.assertEqual(
                item.get("scope"),
                "reporting",
                f"PRISMA-LSR item {item.get('item_id')} must declare scope: reporting (never conduct)",
            )

        crosswalks = {crosswalk for item in items for crosswalk in item.get("crosswalks", [])}
        self.assertIn("living-update", crosswalks)
        self.assertIn("protocol-amendment", crosswalks)

    def test_track_22_traceability_governance(self) -> None:
        self.assertTrue(TRACEABILITY_PATH.is_file(), "Track 22 traceability.json must exist")
        traceability = json.loads(TRACEABILITY_PATH.read_text(encoding="utf-8"))

        self.assertEqual(traceability.get("track_id"), "22")
        self.assertEqual(traceability.get("implementation_state"), "partially_implemented")
        self.assertEqual(traceability.get("evidence_level"), "source_verified")

        assertions = traceability.get("assertions", [])
        self.assertEqual(len(assertions), 6, "Track 22 must define exactly 6 assertions")

        expected_ids = {f"T22-A{i:03d}" for i in range(1, 7)}
        actual_ids = {a["assertion_id"] for a in assertions}
        self.assertEqual(actual_ids, expected_ids)

        for assertion in assertions:
            self.assertEqual(assertion.get("state"), "partially_implemented")
            self.assertEqual(assertion.get("mapping_confidence"), "symbol_and_gate_mapped")
            self.assertGreater(len(assertion.get("implementation_symbols", [])), 0)
            self.assertGreater(len(assertion.get("deterministic_tests", [])), 0)
            self.assertGreater(len(assertion.get("open_gates", [])), 0)
            for path in assertion.get("implementation_paths", []):
                self.assertTrue((ROOT / path).exists(), f"Mapped path {path} must exist on disk")


if __name__ == "__main__":
    unittest.main()
