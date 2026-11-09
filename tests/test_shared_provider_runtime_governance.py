#!/usr/bin/env python3
"""Governance and contract tests for Track 03: Shared provider runtime and Sourceright extraction."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SharedProviderRuntimeGovernanceTests(unittest.TestCase):
    """Verify provider runtime contracts, Sourceright migration maps, and release-train safety boundaries."""

    def test_sourceright_replacement_map_and_parity_cases(self):
        """Verify replacement map and parity cases exist and are valid."""
        map_path = ROOT / "migration/sourceright/replacement-map.yaml"
        self.assertTrue(map_path.is_file(), "Missing migration/sourceright/replacement-map.yaml")
        map_content = map_path.read_text(encoding="utf-8")
        self.assertIn("evidence-search-core", map_content)

        cases_path = ROOT / "migration/sourceright/parity-cases.json"
        self.assertTrue(cases_path.is_file(), "Missing migration/sourceright/parity-cases.json")
        cases = json.loads(cases_path.read_text(encoding="utf-8"))
        self.assertEqual(cases.get("schema_version"), "searchright.sourceright-parity-cases.v1")
        self.assertGreaterEqual(len(cases.get("cases", [])), 1)

    def test_parity_schemas(self):
        """Verify sourceright parity catalogue and matrix schemas."""
        cat_schema_path = ROOT / "contracts/json-schema/sourceright-parity-catalogue.v2.schema.json"
        self.assertTrue(cat_schema_path.is_file(), "Missing parity catalogue schema")
        cat_schema = json.loads(cat_schema_path.read_text(encoding="utf-8"))
        self.assertEqual(cat_schema.get("$schema"), "https://json-schema.org/draft/2020-12/schema")

        matrix_schema_path = ROOT / "contracts/json-schema/sourceright-parity-matrix.v2.schema.json"
        self.assertTrue(matrix_schema_path.is_file(), "Missing parity matrix schema")
        matrix_schema = json.loads(matrix_schema_path.read_text(encoding="utf-8"))
        self.assertEqual(matrix_schema.get("$schema"), "https://json-schema.org/draft/2020-12/schema")

    def test_companion_repository_packet_fail_closed(self):
        """Verify migration/companion-repositories/sourceright.json is fail-closed."""
        companion_path = ROOT / "migration/companion-repositories/sourceright.json"
        self.assertTrue(companion_path.is_file(), "Missing companion repo packet")
        packet = json.loads(companion_path.read_text(encoding="utf-8"))
        self.assertEqual(packet.get("schema_version"), "org.searchright.companion-change-packet.v1")
        self.assertFalse(packet.get("remote_mutation_permitted"))
        self.assertEqual(packet.get("repository"), "edithatogo/sourceright")

    def test_release_train_boundaries(self):
        """Verify integration/release-train.json defines staged release without unapproved writes."""
        train_path = ROOT / "integration/release-train.json"
        self.assertTrue(train_path.is_file(), "Missing release train")
        train = json.loads(train_path.read_text(encoding="utf-8"))
        self.assertEqual(train.get("schema_version"), "org.searchright.integration-release-train.v1")
        self.assertIn("stages", train)
        self.assertIn("components", train)
        self.assertFalse(train.get("automatic_promotion"))

    def test_provider_contract_and_migration_checks(self):
        """Verify check_sourceright_migration.py and check_provider_contract_baselines.py pass network-free."""
        for script in (
            "scripts/check_sourceright_migration.py",
            "scripts/check_provider_contract_baselines.py",
        ):
            res = subprocess.run(
                ["python3", str(ROOT / script)],
                cwd=str(ROOT),
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 0, f"{script} failed: {res.stderr}")

    def test_traceability_assertion_coverage(self):
        """Verify all Track 03 assertions are mapped and symbol-backed."""
        trace_path = ROOT / "conductor/tracks/03-shared-provider-runtime/traceability.json"
        self.assertTrue(trace_path.is_file(), "Missing traceability.json")
        trace = json.loads(trace_path.read_text(encoding="utf-8"))
        self.assertEqual(trace.get("track_id"), "03")
        self.assertEqual(len(trace.get("assertions", [])), 6)
        for a in trace["assertions"]:
            self.assertEqual(a.get("mapping_confidence"), "symbol_and_gate_mapped")
            self.assertTrue(len(a.get("implementation_paths", [])) > 0)


if __name__ == "__main__":
    unittest.main()
