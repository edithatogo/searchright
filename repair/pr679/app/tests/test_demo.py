from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

APP_ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("chhhs_research_demo", APP_ROOT / "chhhs_research_demo.py")
assert SPEC and SPEC.loader
DEMO = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DEMO)


class AttributionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.config = DEMO.load_json(APP_ROOT / "config.json")
        cls.fixtures = DEMO.load_json(APP_ROOT / "fixtures" / "records.json")

    def test_full_service_alias_is_confirmed(self) -> None:
        result = DEMO.attribute(self.fixtures[2], self.config)
        self.assertEqual(result.status, "confirmed")
        self.assertGreaterEqual(result.score, 0.9)
        self.assertTrue(any(item.startswith("affiliation_alias:") for item in result.evidence))

    def test_facility_alias_is_confirmed(self) -> None:
        result = DEMO.attribute(self.fixtures[3], self.config)
        self.assertEqual(result.status, "confirmed")
        self.assertIn("affiliation_alias:Atherton Hospital", result.evidence)

    def test_cairns_geographic_mention_is_not_attributed(self) -> None:
        result = DEMO.attribute(self.fixtures[4], self.config)
        self.assertEqual(result.status, "insufficient_evidence")
        self.assertEqual(result.evidence, ("geographic_cairns_only",))

    def test_non_affiliation_service_mention_requires_review(self) -> None:
        record = {
            "title": "Collaboration with Cairns Hospital",
            "abstract": "The authors acknowledge support but do not report a health-service affiliation.",
            "affiliations": ["Example University"],
            "institution_ids": [],
        }
        result = DEMO.attribute(record, self.config)
        self.assertEqual(result.status, "review_required")
        self.assertLess(result.score, 0.5)


class PipelineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.config = DEMO.load_json(APP_ROOT / "config.json")
        cls.fixtures = DEMO.load_json(APP_ROOT / "fixtures" / "records.json")

    def test_doi_overlap_is_deduplicated_without_losing_sources(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            state_path = Path(directory) / "state.json"
            state = DEMO.update_state(
                self.fixtures,
                self.config,
                state_path,
                "2026-08-31T00:00:00+00:00",
            )
        self.assertEqual(len(state["records"]), 5)
        duplicate = next(
            record for record in state["records"] if record["doi"] == "10.9999/chhhs.demo.001"
        )
        self.assertEqual(duplicate["sources"], ["openalex", "pubmed"])
        self.assertEqual(set(duplicate["source_record_ids"]), {"openalex", "pubmed"})
        self.assertEqual(duplicate["attribution"]["status"], "confirmed")

    def test_classification_retains_matched_terms_and_taxonomy_version(self) -> None:
        result = DEMO.classify(self.fixtures[0], self.config)
        theme_ids = {theme["id"] for theme in result["themes"]}
        self.assertIn("emergency_and_critical_care", theme_ids)
        self.assertIn("toxinology", theme_ids)
        self.assertEqual(result["study_type"], "cohort_study")
        self.assertEqual(result["taxonomy_version"], "2026-09-demo")
        self.assertIn("retrospective cohort", result["study_type_terms"])

    def test_report_excludes_insufficient_evidence_and_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            state_path = root / "state.json"
            state = DEMO.update_state(
                self.fixtures,
                self.config,
                state_path,
                "2026-08-31T00:00:00+00:00",
            )
            first = DEMO.render_report(state, "2026-08", root / "first")
            second = DEMO.render_report(state, "2026-08", root / "second")
            first_json = first["json"].read_bytes()
            second_json = second["json"].read_bytes()
            first_csv = first["csv"].read_bytes()
            second_csv = second["csv"].read_bytes()
            first_html = first["html"].read_bytes()
            second_html = second["html"].read_bytes()

        self.assertEqual(first_json, second_json)
        self.assertEqual(first_csv, second_csv)
        self.assertEqual(first_html, second_html)
        report = json.loads(first_json)
        self.assertEqual(report["candidate_records"], 5)
        self.assertEqual(report["included_candidates"], 3)
        self.assertEqual(report["counts_by_attribution"]["confirmed"], 3)
        self.assertEqual(report["counts_by_attribution"]["insufficient_evidence"], 2)
        self.assertNotIn("Community perceptions of dengue prevention in Cairns", first_html.decode())

    def test_adapter_output_accepts_supported_wire_shapes(self) -> None:
        record = {"title": "Example"}
        self.assertEqual(DEMO.read_adapter_output(json.dumps([record])), [record])
        self.assertEqual(DEMO.read_adapter_output(json.dumps({"records": [record]})), [record])
        self.assertEqual(DEMO.read_adapter_output(json.dumps(record) + "\n"), [record])

    def test_adapter_output_rejects_unknown_shape(self) -> None:
        with self.assertRaises(ValueError):
            DEMO.read_adapter_output(json.dumps({"items": []}))


if __name__ == "__main__":
    unittest.main()
