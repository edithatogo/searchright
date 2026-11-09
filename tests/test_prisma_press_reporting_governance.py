#!/usr/bin/env python3
"""Governance and contract tests for Track 08: PRISMA, PRESS and reporting."""

from __future__ import annotations

import json
from pathlib import Path
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]


class PrismaPressReportingGovernanceTests(unittest.TestCase):
    """Verify PRISMA reporting contracts, PRESS assessment integrity, and PRISMA.jl interop boundaries."""

    def test_prisma_s_reporting_contract_boundary(self):
        """Verify PRISMA-S is enforced as reporting completeness and not conduct certification."""
        agents_md = ROOT / "AGENTS.md"
        self.assertTrue(agents_md.is_file(), "Missing AGENTS.md")
        agents_content = agents_md.read_text(encoding="utf-8")
        self.assertIn("Never represent PRISMA-S as a conduct standard; it is a reporting contract", agents_content)

        doc_path = ROOT / "docs/standards-and-provenance.md"
        self.assertTrue(doc_path.is_file(), "Missing docs/standards-and-provenance.md")
        doc_content = doc_path.read_text(encoding="utf-8")
        self.assertIn("reporting completeness", doc_content)

        prisma_lib = ROOT / "crates/searchright-prisma/src/lib.rs"
        self.assertTrue(prisma_lib.is_file(), "Missing crates/searchright-prisma/src/lib.rs")
        lib_content = prisma_lib.read_text(encoding="utf-8")
        self.assertIn("SearchReportingEvidence", lib_content)
        self.assertIn("prisma_s_ledger", lib_content)

    def test_standards_packs_integrity(self):
        """Verify PRISMA-ScR (2018) and PRISMA-LSR (2024) standard packs."""
        scr_path = ROOT / "contracts/standards/packs/prisma-scr-2018.yaml"
        self.assertTrue(scr_path.is_file(), "Missing prisma-scr-2018.yaml")
        scr_data = yaml.safe_load(scr_path.read_text(encoding="utf-8"))
        self.assertEqual(scr_data.get("pack_id"), "prisma-scr-2018")
        self.assertEqual(scr_data.get("family"), "prisma_sc_r")
        self.assertEqual(scr_data.get("version"), "2018")
        self.assertGreaterEqual(len(scr_data.get("items", [])), 2)

        lsr_path = ROOT / "contracts/standards/packs/prisma-lsr-2024.yaml"
        self.assertTrue(lsr_path.is_file(), "Missing prisma-lsr-2024.yaml")
        lsr_data = yaml.safe_load(lsr_path.read_text(encoding="utf-8"))
        self.assertEqual(lsr_data.get("pack_id"), "prisma-lsr-2024")
        self.assertEqual(lsr_data.get("family"), "prisma_lsr")
        self.assertEqual(lsr_data.get("version"), "2024")
        self.assertGreaterEqual(len(lsr_data.get("items", [])), 2)

    def test_press_review_workflow_contract(self):
        """Verify six-domain PRESS review structure and representative workflow fixtures."""
        press_fixture_path = ROOT / "verification/fixtures/track-08/press-workflow-complete.yaml"
        self.assertTrue(press_fixture_path.is_file(), "Missing press-workflow-complete.yaml")
        press_data = yaml.safe_load(press_fixture_path.read_text(encoding="utf-8"))
        self.assertEqual(press_data.get("schema_version"), "org.searchright.search-validation.v1")
        reviews = press_data.get("press_reviews", [])
        self.assertGreaterEqual(len(reviews), 1)
        findings = reviews[0].get("findings", [])
        self.assertEqual(len(findings), 6)

        expected_elements = {
            "translation_of_question",
            "boolean_and_proximity",
            "subject_headings",
            "text_words",
            "spelling_syntax_and_lines",
            "limits_and_filters",
        }
        observed_elements = {f.get("element") for f in findings}
        self.assertEqual(expected_elements, observed_elements)

    def test_prisma_flow_example_structure(self):
        """Verify PRISMA 2020 flow example has required stages and numbers."""
        flow_path = ROOT / "contracts/examples/prisma-flow.json"
        self.assertTrue(flow_path.is_file(), "Missing prisma-flow.json")
        flow = json.loads(flow_path.read_text(encoding="utf-8"))
        self.assertEqual(flow.get("schema_version"), "org.searchright.prisma-flow.v1")
        self.assertIn("records_databases", flow)
        self.assertIn("records_screened", flow)
        self.assertIn("records_excluded", flow)
        self.assertIn("studies_included", flow)
        self.assertIn("reports_included", flow)

    def test_prisma_jl_interop_passport_and_fixtures(self):
        """Verify PRISMA.jl interoperability passport records explicit loss boundaries."""
        passport_path = ROOT / "integration/passports/prisma-jl-flow-interop.json"
        self.assertTrue(passport_path.is_file(), "Missing prisma-jl-flow-interop.json")
        passport = json.loads(passport_path.read_text(encoding="utf-8"))
        self.assertEqual(passport.get("schema_version"), "org.searchright.integration-passport.v1")
        self.assertEqual(passport.get("integration_id"), "prisma-jl-flow-interop")
        self.assertIn("PRISMA.jl", passport.get("repository", ""))

        parity_fixture = ROOT / "integration/fixtures/prisma-jl/flow-parity-v1.json"
        self.assertTrue(parity_fixture.is_file(), "Missing flow-parity-v1.json")
        fixture_data = json.loads(parity_fixture.read_text(encoding="utf-8"))
        self.assertEqual(fixture_data.get("schema_version"), "org.searchright.prisma-jl-flow-parity.v1")
        self.assertIn("mapped_counts", fixture_data)
        self.assertIn("loss_report", fixture_data)
        self.assertGreaterEqual(len(fixture_data["loss_report"]), 1)

    def test_prisma_renderer_formats_declared(self):
        """Verify crates/searchright-prisma/src/render.rs implements expected multi-format output."""
        render_rs = ROOT / "crates/searchright-prisma/src/render.rs"
        self.assertTrue(render_rs.is_file(), "Missing crates/searchright-prisma/src/render.rs")
        render_content = render_rs.read_text(encoding="utf-8")
        self.assertIn("enum FlowFormat", render_content)
        self.assertIn("Markdown", render_content)
        self.assertIn("Json", render_content)
        self.assertIn("Mermaid", render_content)
        self.assertIn("Svg", render_content)
        self.assertIn("Typst", render_content)
        self.assertIn("DocxFriendlyHtml", render_content)
        self.assertIn("pub fn render_flow", render_content)


if __name__ == "__main__":
    unittest.main()
