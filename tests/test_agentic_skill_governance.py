#!/usr/bin/env python3
"""Governance and contract tests for Track 11: Systematic-search agent skill and workflows."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]


class AgenticSkillGovernanceTests(unittest.TestCase):
    """Verify systematic-search agent skill, authority checkpoints, handoff contracts, and sibling policy."""

    def test_skill_manifest_and_workflow_structure(self):
        """Verify SKILL.md and systematic-review workflow stages."""
        skill_path = ROOT / "skills/systematic-search/SKILL.md"
        self.assertTrue(skill_path.is_file(), "Missing skills/systematic-search/SKILL.md")
        content = skill_path.read_text(encoding="utf-8")
        self.assertIn("name: systematic-search", content)
        self.assertIn("independent press review", content.lower())

        wf_path = ROOT / "skills/systematic-search/workflows/systematic-review.yaml"
        self.assertTrue(wf_path.is_file(), "Missing systematic-review.yaml")
        wf = yaml.safe_load(wf_path.read_text(encoding="utf-8"))
        stages = wf.get("stages", [])
        self.assertGreaterEqual(len(stages), 5)

    def test_authority_and_human_checkpoints_boundaries(self):
        """Verify authority boundaries enforce explicit human confirmation for study exclusion."""
        auth_doc = ROOT / "skills/systematic-search/references/authority.md"
        self.assertTrue(auth_doc.is_file(), "Missing references/authority.md")
        auth_text = auth_doc.read_text(encoding="utf-8")
        self.assertIn("denied by default", auth_text.lower())
        self.assertIn("human reviewer", auth_text.lower())

        scenarios_path = ROOT / "skills/systematic-search/evaluations/authority-scenarios.json"
        self.assertTrue(scenarios_path.is_file(), "Missing authority-scenarios.json")
        scenarios = json.loads(scenarios_path.read_text(encoding="utf-8"))
        cases = scenarios.get("cases", [])
        self.assertGreaterEqual(len(cases), 5)

    def test_handoffs_contract_and_schema(self):
        """Verify agent handoff schema and example validate bounded transitions."""
        schema_path = ROOT / "contracts/json-schema/agent-handoff.v1.schema.json"
        self.assertTrue(schema_path.is_file(), "Missing agent-handoff schema")
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        self.assertEqual(schema.get("$schema"), "https://json-schema.org/draft/2020-12/schema")

        example_path = ROOT / "contracts/examples/agent-handoff.json"
        self.assertTrue(example_path.is_file(), "Missing agent-handoff example")
        example = json.loads(example_path.read_text(encoding="utf-8"))
        self.assertEqual(example.get("schema_version"), "org.searchright.agent-handoff.v1")
        self.assertIn("from_role", example)
        self.assertIn("to_role", example)

    def test_sibling_caller_policy_and_admission(self):
        """Verify sibling caller ADR and integration enforce explicit user handoff."""
        adr_path = ROOT / "docs/adrs/0018-searchright-owned-sibling-caller.md"
        self.assertTrue(adr_path.is_file(), "Missing ADR 0018")
        adr_text = adr_path.read_text(encoding="utf-8")
        self.assertIn("searchright-owned sibling", adr_text.lower())

        ars_skill = ROOT / "skills/systematic-search/integrations/academic-research-skills/SKILL.md"
        self.assertTrue(ars_skill.is_file(), "Missing academic-research-skills integration SKILL.md")
        ars_text = ars_skill.read_text(encoding="utf-8")
        self.assertIn("systematic-search", ars_text)

    def test_agent_skill_verification_script(self):
        """Execute check_agent_skill.py deterministically."""
        res = subprocess.run(
            ["python3", str(ROOT / "scripts/check_agent_skill.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"check_agent_skill.py failed: {res.stderr}")
        data = json.loads(res.stdout)
        self.assertEqual(data.get("status"), "passed")

    def test_agent_skill_policy_and_host_eval_tests(self):
        """Execute test_agent_skill_policy.py and test_agent_host_eval.py."""
        res1 = subprocess.run(
            ["python3", str(ROOT / "scripts/test_agent_skill_policy.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res1.returncode, 0, f"test_agent_skill_policy.py failed: {res1.stderr}")

        res2 = subprocess.run(
            ["python3", str(ROOT / "scripts/test_agent_host_eval.py")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        self.assertEqual(res2.returncode, 0, f"test_agent_host_eval.py failed: {res2.stderr}")


if __name__ == "__main__":
    unittest.main()
