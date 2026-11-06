"""Tests for Track 29: External methodological evaluation and sustainability.

Source-level and network-free. They verify that the evaluation, publication and
sustainability documents are internally consistent and that none of them
over-claims. They are not preregistration, recruitment, execution, publication
or observed-sustainability evidence; those gates remain open.
"""
from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVALUATION_DOC = ROOT / "docs/evaluation/external-methodological-evaluation.md"
BENCHMARK_DOC = ROOT / "docs/evaluation/benchmark-and-calibration-protocol.md"
SUSTAINABILITY_DOC = ROOT / "docs/governance/sustainability.md"
JOSS_PAPER = ROOT / "registry/joss/paper.md"
GOVERNANCE = ROOT / "GOVERNANCE.md"
CONTRACT = ROOT / "evaluation/external-evaluation.json"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def collapse(text: str) -> str:
    return re.sub(r"\s+", " ", text)


class ExternalEvaluationSustainabilityGovernanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.contract = json.loads(read(CONTRACT))

    def test_protocol_document_matches_machine_contract(self) -> None:
        text = collapse(read(EVALUATION_DOC))
        self.assertIn("preregistered, multi-topic crossover evaluation", text)
        self.assertIn("at least two independent information specialists", text)
        self.assertIn("blinded adjudication", text)
        self.assertIn("response-to-findings matrix", text)
        self.assertIn("at least one living-review update", text)
        self.assertIn("deliberately adversarial provider data", text)
        for topic in self.contract["design"]["topic_strata"]:
            with self.subTest(topic=topic):
                self.assertIn(topic, text)
        for outcome in ("Seed-set recall", "critical PRESS findings", "unreported lossy translations",
                        "reproduction success", "consequential error rate"):
            with self.subTest(outcome=outcome):
                self.assertIn(outcome, text)
        self.assertEqual(self.contract["independence"]["minimum_information_specialists"], 2)

    def test_evaluation_document_denies_preregistration_and_validation(self) -> None:
        text = collapse(read(EVALUATION_DOC))
        self.assertIn("It is not preregistration, recruitment, execution, ethics approval "
                      "or external validation evidence", text)
        self.assertEqual(self.contract["status"], "prepared_not_preregistered")
        self.assertTrue(all(value is None for value in self.contract["external_evidence"].values()))

    def test_adjudication_is_blinded_and_independent_of_developers(self) -> None:
        independence = self.contract["independence"]
        design = self.contract["design"]
        self.assertTrue(independence["development_team_excluded_from_adjudication"])
        self.assertTrue(design["blinded_adjudication"])
        self.assertTrue(design["sealed_labels_required"])
        self.assertIsNone(self.contract["external_evidence"]["response_to_findings"])

    def test_benchmark_protocol_controls_leakage_and_keeps_internal_claim_bounded(self) -> None:
        text = collapse(read(BENCHMARK_DOC))
        for phrase in (
            "Keep hidden evaluation reviews separate from development fixtures",
            "Hash and version every split",
            "Report failed and null runs",
            "Do not use test labels for prompt construction, query expansion or stopping",
            "ranking cannot discard records",
            "never substitutes for independent information-specialist review or live database evidence",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

    def test_joss_paper_is_an_unsubmitted_draft_with_no_empirical_claims(self) -> None:
        text = collapse(read(JOSS_PAPER))
        self.assertIn("Draft only — not submitted", text)
        self.assertIn("must not be submitted until the software is compiled, benchmarked, archived "
                      "and used outside its initial development context", text)
        self.assertGreaterEqual(text.count("To be completed") + text.count("To be generated"), 3)
        for forbidden in ("outperforms", "validated", "state-of-the-art", "peer-reviewed"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, text.lower())

    def test_sustainability_requires_observation_not_documentation(self) -> None:
        text = collapse(read(SUSTAINABILITY_DOC))
        self.assertIn("Version contracts before implementations", text)
        self.assertIn("Publish deprecation windows and migration recipes", text)
        self.assertIn("Record standards surveillance and licence changes", text)
        self.assertIn("reproducible by a fresh maintainer from a tagged commit", text)
        self.assertIn("do not demonstrate maintenance history, standards surveillance or succession "
                      "practice", text)
        sustainability = self.contract["sustainability"]
        self.assertEqual(sustainability["status"], "prepared_not_observed")
        self.assertEqual(sustainability["observation_receipts"], [])
        self.assertEqual(
            set(sustainability["required_observations"]),
            {"maintenance_release_history", "standards_surveillance_history",
             "deprecation_or_migration_exercise", "succession_rehearsal"},
        )

    def test_governance_classifies_decisions_and_blocks_inferred_external_claims(self) -> None:
        text = collapse(read(GOVERNANCE))
        for decision_class in ("Routine", "Contract", "Methodological", "Security/privacy", "External claim"):
            with self.subTest(decision_class=decision_class):
                self.assertIn(f"**{decision_class}:**", text)
        self.assertIn("must not be inferred from repository metadata alone", text)

    def test_funding_and_conflict_disclosure_is_mandatory_and_explicit(self) -> None:
        text = collapse(read(SUSTAINABILITY_DOC))
        self.assertIn("must be disclosed with their effect on roadmap priority and benchmark independence", text)
        disclosures = self.contract["disclosures"]
        self.assertEqual(set(disclosures), {"funding", "conflicts", "provider_relationships"})
        self.assertTrue(all(value.strip() for value in disclosures.values()))
        self.assertTrue(self.contract["independence"]["conflicts_must_be_disclosed"])
        self.assertIn("not preregistration", self.contract["claim_boundary"])


if __name__ == "__main__":
    unittest.main()
