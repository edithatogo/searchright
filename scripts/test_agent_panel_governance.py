"""Regression tests for the single-owner, sealed agent-panel review model."""
from __future__ import annotations

import json
from pathlib import Path
import re
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
SKILL_ROOT = ROOT / "skills" / "systematic-search"
EXPECTED_PANEL_ROLES = [
    "methodology",
    "information_retrieval",
    "implementation_testing",
    "security_privacy_rights",
    "adversarial_replication",
]


def normalized(path: Path) -> str:
    return re.sub(r"\s+", " ", path.read_text(encoding="utf-8").lower())


class AgentPanelGovernanceTests(unittest.TestCase):
    def test_skill_declares_single_owner_and_sealed_panel(self) -> None:
        text = normalized(SKILL_ROOT / "SKILL.md")
        for phrase in (
            "single accountable human-owner",
            "sealed panel",
            "preserves individual findings, abstentions and dissent",
            "agent panel may perform press-aligned critique",
            "it is not independent human press peer review",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

    def test_obsolete_mandatory_second_human_language_is_absent(self) -> None:
        skill = (SKILL_ROOT / "SKILL.md").read_text(encoding="utf-8")
        authority = (SKILL_ROOT / "references" / "authority.md").read_text(
            encoding="utf-8"
        )
        plan = skill + "\n" + authority
        self.assertNotIn("A human information specialist should review", plan)
        self.assertNotIn("Two reviewers/adjudicator as protocolled", plan)

    def test_workflow_requires_owner_adjudication_before_live_execution(self) -> None:
        workflow = yaml.safe_load(
            (SKILL_ROOT / "workflows" / "systematic-review.yaml").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(workflow["accountable_owner_model"], "single_human_owner")
        press = next(stage for stage in workflow["stages"] if stage["id"] == "press")
        self.assertEqual(press["review_model"], "owner_adjudicated_agent_panel")
        self.assertEqual(press["isolation"], "sealed_first_passes")
        self.assertEqual(press["authority"], "advisory_only")
        self.assertEqual(press["panel_roles"], EXPECTED_PANEL_ROLES)
        self.assertEqual(
            press["claim_boundary"],
            "agent_panel_is_not_independent_human_peer_review",
        )
        execute = next(stage for stage in workflow["stages"] if stage["id"] == "execute")
        self.assertEqual(
            execute["modes"]["live"]["requires"],
            [
                "agent_panel_adjudication",
                "strategy_and_press_approval",
                "live_execution_approval",
            ],
        )

    def test_screening_remains_advisory(self) -> None:
        workflow = yaml.safe_load(
            (SKILL_ROOT / "workflows" / "systematic-review.yaml").read_text(
                encoding="utf-8"
            )
        )
        screen = next(stage for stage in workflow["stages"] if stage["id"] == "screen")
        self.assertEqual(screen["authority"], "advisory_only")
        self.assertEqual(
            screen["final_decision_authority"],
            "accountable_owner_or_designated_human",
        )

    def test_agent_panel_template_is_unexecuted_and_fail_closed(self) -> None:
        template = json.loads(
            (SKILL_ROOT / "evaluations" / "agent-panel-template.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(
            template["schema_version"],
            "org.searchright.agent-panel-review.v1",
        )
        self.assertEqual(template["panel_roles"], EXPECTED_PANEL_ROLES)
        self.assertEqual(template["responses"], [])
        self.assertIsNone(template["owner_adjudication_sha256"])
        self.assertIsNone(template["owner_id"])
        self.assertEqual(template["status"], "awaiting_panel")
        self.assertTrue(template["blocking_hazards"])

    def test_external_human_calibration_is_optional_and_unobserved(self) -> None:
        template = json.loads(
            (
                SKILL_ROOT
                / "evaluations"
                / "human-calibration-template.json"
            ).read_text(encoding="utf-8")
        )
        self.assertEqual(template["status"], "optional_not_scheduled")
        self.assertEqual(template["reviewers"], [])
        self.assertEqual(template["blocking_hazards"], [])
        self.assertIn("optional", template["claim_boundary"].lower())


if __name__ == "__main__":
    unittest.main()
