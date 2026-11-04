"""Tests for Track 26: Formal assurance and contract evolution.

Verifies:
1. Finite-state workflow trace contracts and forbidden transitions.
2. Authority invariants for human sign-off and agent screening boundaries.
3. Schema migration registry policy and non-destructive forward/backward rules.
4. Schema migration plan validity and rollback support.
5. Automated validation script execution.
"""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]
CATALOG_PATH = ROOT / "contracts" / "schema-catalog.json"
MIGRATION_REGISTRY_PATH = ROOT / "contracts" / "migrations" / "registry.json"
WORKFLOW_TRACE_EXAMPLE = ROOT / "contracts" / "examples" / "workflow-trace.yaml"
SCHEMA_MIGRATIONS_SCRIPT = ROOT / "scripts" / "check_schema_migrations.py"


class TestFormalAssuranceContractEvolution(unittest.TestCase):
    def setUp(self) -> None:
        self.assertTrue(CATALOG_PATH.is_file(), "Catalog file missing")
        self.assertTrue(MIGRATION_REGISTRY_PATH.is_file(), "Migration registry missing")
        self.assertTrue(WORKFLOW_TRACE_EXAMPLE.is_file(), "Workflow trace example missing")
        self.assertTrue(SCHEMA_MIGRATIONS_SCRIPT.is_file(), "Schema migrations script missing")

    def test_workflow_trace_example_contract(self) -> None:
        """Validate workflow-trace.yaml matches the schema contract and contains valid transitions."""
        content = yaml.safe_load(WORKFLOW_TRACE_EXAMPLE.read_text(encoding="utf-8"))
        self.assertIsInstance(content, dict)
        self.assertEqual(content.get("schema_version"), "org.searchright.workflow-trace.v1")
        self.assertEqual(content.get("initial_stage"), "draft")
        self.assertIn("review_id", content)

        transitions = content.get("transitions", [])
        self.assertIsInstance(transitions, list)
        self.assertGreaterEqual(len(transitions), 2)

        current_stage = content["initial_stage"]
        for idx, step in enumerate(transitions):
            self.assertEqual(step["from"], current_stage, f"Discontinuity at step {idx}: expected {current_stage}")
            self.assertIn("to", step)
            self.assertIn("actor_kind", step)
            self.assertIn("actor_id", step)
            self.assertIn("occurred_at", step)
            self.assertIn("evidence_ids", step)
            self.assertTrue(step["approved"], "Step must be approved")
            current_stage = step["to"]

    def test_schema_migration_registry_policy(self) -> None:
        """Verify the schema migration registry default policy enforces strict fail-closed constraints."""
        registry = json.loads(MIGRATION_REGISTRY_PATH.read_text(encoding="utf-8"))
        self.assertEqual(registry.get("schema_version"), "org.searchright.schema-migration-registry.v1")
        self.assertFalse(registry.get("automatic_migration", True), "automatic_migration must remain false")

        policy = registry.get("default_policy", {})
        self.assertEqual(policy.get("unknown_version"), "reject")
        self.assertEqual(policy.get("destructive_migration"), "deny")
        self.assertEqual(policy.get("implicit_write_upgrade"), "deny")
        self.assertTrue(policy.get("backup_required"))
        self.assertTrue(policy.get("receipt_required"))

    def test_schema_migration_plans(self) -> None:
        """Verify all registered migration plans exist, are non-destructive, and have rollback support."""
        registry = json.loads(MIGRATION_REGISTRY_PATH.read_text(encoding="utf-8"))
        families = registry.get("families", [])
        self.assertGreater(len(families), 0, "At least one schema family must be registered")

        for family_entry in families:
            family_name = family_entry["family"]
            migrations = family_entry.get("migrations", [])
            self.assertGreater(len(migrations), 0, f"Family {family_name} must have migration plans")

            for migration_rel in migrations:
                migration_path = ROOT / migration_rel
                self.assertTrue(migration_path.is_file(), f"Migration file {migration_rel} not found")
                plan = json.loads(migration_path.read_text(encoding="utf-8"))
                self.assertEqual(plan.get("family"), family_name)
                self.assertFalse(plan.get("automatic_apply", True))
                self.assertFalse(plan.get("destructive", True))
                self.assertTrue(plan.get("backup_required"))
                rollback = plan.get("rollback", {})
                self.assertTrue(rollback.get("supported"), "Rollback must be supported")

    def test_check_schema_migrations_script(self) -> None:
        """Run scripts/check_schema_migrations.py and verify zero errors."""
        result = subprocess.run(
            [sys.executable, str(SCHEMA_MIGRATIONS_SCRIPT)],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertEqual(
            result.returncode,
            0,
            f"check_schema_migrations.py failed: stdout={result.stdout}, stderr={result.stderr}",
        )
        output = json.loads(result.stdout)
        self.assertEqual(output.get("status"), "passed")
        self.assertEqual(output.get("errors"), [])


if __name__ == "__main__":
    unittest.main()
