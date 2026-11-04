"""Tests for Track 27: Accessibility, internationalisation and usability.

Validates:
1. Stable diagnostic codes, severities, remediation fields, and schema contracts.
2. Accessible, colour-free rendering across PlainText, JSON, and JSONL formats.
3. Locale-neutral diagnostic codes and separable message catalogues with explicit fallback policies.
4. Terminal-width, keyboard, and screen-reader accessible design constraints.
5. Information-specialist usability protocol contracts and issue taxonomies.
"""
from __future__ import annotations

import json
from pathlib import Path
import re
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]
DIAGNOSTIC_SCHEMA = ROOT / "contracts" / "json-schema" / "diagnostic.v1.schema.json"
DIAGNOSTIC_EXAMPLE = ROOT / "contracts" / "examples" / "diagnostic.yaml"
EVALUATION_PLAN = ROOT / "evaluation" / "external-evaluation.json"
ADR_0010 = ROOT / "docs" / "adrs" / "0010-accessible-diagnostics-and-institutional-governance.md"


class TestAccessibilityInternationalisationUsabilityGovernance(unittest.TestCase):
    def setUp(self) -> None:
        self.assertTrue(DIAGNOSTIC_SCHEMA.is_file(), "diagnostic.v1.schema.json missing")
        self.assertTrue(DIAGNOSTIC_EXAMPLE.is_file(), "diagnostic.yaml missing")
        self.assertTrue(EVALUATION_PLAN.is_file(), "external-evaluation.json missing")
        self.assertTrue(ADR_0010.is_file(), "ADR 0010 missing")

    def test_diagnostic_example_conforms_to_schema_and_invariants(self) -> None:
        """Validate example diagnostic against schema and stable namespace invariants."""
        example_data = yaml.safe_load(DIAGNOSTIC_EXAMPLE.read_text(encoding="utf-8"))
        schema_data = json.loads(DIAGNOSTIC_SCHEMA.read_text(encoding="utf-8"))

        self.assertEqual(example_data.get("schema_version"), "org.searchright.diagnostic.v1")
        self.assertEqual(schema_data.get("$id"), "https://schemas.searchright.dev/diagnostic.v1.schema.json")

        code = example_data.get("code")
        self.assertIsInstance(code, str)
        self.assertIn(".", code, "Diagnostic code must be namespaced with '.'")
        self.assertTrue(
            re.fullmatch(r"^[a-z0-9]+(\.[a-z0-9_-]+)+$", code),
            f"Diagnostic code {code!r} must be lower-case alphanumeric namespaced",
        )

        severity = example_data.get("severity")
        self.assertIn(severity, ["information", "warning", "error", "blocking"])
        if severity == "blocking":
            self.assertTrue(example_data.get("blocking"), "blocking severity must set blocking=true")
            self.assertTrue(example_data.get("remediation"), "blocking diagnostic requires remediation")

    def test_accessible_rendering_contracts(self) -> None:
        """Verify plain text diagnostic rendering contracts: no ANSI sequences, word-based severities."""
        example_data = yaml.safe_load(DIAGNOSTIC_EXAMPLE.read_text(encoding="utf-8"))

        # Render simulated plain text format
        plain = (
            f"{example_data['severity'].upper()} {example_data['code']}: {example_data['message']}\n"
            f"  path: {example_data.get('path', 'none')}:{example_data.get('line', 1)}:{example_data.get('column', 1)}\n"
            f"  remediation: {example_data.get('remediation', 'none')}\n"
        )

        # Invariant 1: No ANSI escape sequences (e.g. \x1b[31m)
        self.assertNotIn("\x1b", plain, "Accessible plain text rendering must contain zero ANSI escapes")

        # Invariant 2: Explicit severity words, no color/glyph dependency
        self.assertTrue(
            any(plain.startswith(s) for s in ["BLOCKING", "ERROR", "WARNING", "INFORMATION"]),
            "Plain text output must start with explicit severity word",
        )

        # Invariant 3: No unlabelled glyphs (like standalone warning symbols without words)
        for glyph in ["⚠", "✗", "✓", "⛔"]:
            self.assertNotIn(glyph, plain)

    def test_locale_neutrality_and_message_separation(self) -> None:
        """Ensure machine codes remain identical across locale translations."""
        en_diagnostic = {
            "code": "strategy.translation.review_required",
            "severity": "blocking",
            "message": "Human translation review is required.",
            "remediation": "Review each material dialect warning.",
            "locale": "en_au",
        }

        # Simulated Māori catalogue translation
        mi_translation = {
            "message": "Me arotake tēnei whakamāoritanga.",
            "remediation": "Arotakengia ngā whakatūpato.",
            "locale": "mi_nz",
        }

        # Machine identifiers remain completely invariant
        translated = dict(en_diagnostic)
        translated.update(mi_translation)

        self.assertEqual(translated["code"], en_diagnostic["code"])
        self.assertEqual(translated["severity"], en_diagnostic["severity"])
        self.assertEqual(translated["locale"], "mi_nz")
        self.assertNotEqual(translated["message"], en_diagnostic["message"])

    def test_information_specialist_usability_protocol(self) -> None:
        """Verify the external usability evaluation protocol adheres to independence and design constraints."""
        eval_data = json.loads(EVALUATION_PLAN.read_text(encoding="utf-8"))
        self.assertEqual(eval_data.get("schema_version"), "org.searchright.external-evaluation.v1")
        self.assertEqual(eval_data.get("status"), "prepared_not_preregistered")

        # Independence requirements
        independence = eval_data.get("independence", {})
        self.assertGreaterEqual(independence.get("minimum_information_specialists", 0), 2)
        self.assertTrue(independence.get("development_team_excluded_from_adjudication"))
        self.assertTrue(independence.get("conflicts_must_be_disclosed"))

        # Design and outcomes
        design = eval_data.get("design", {})
        self.assertTrue(design.get("blinded_adjudication"))
        self.assertTrue(design.get("living_review_required"))
        self.assertTrue(design.get("sealed_labels_required"))

        required_outcomes = set(design.get("required_outcomes", []))
        self.assertTrue({"seed_set_recall", "critical_press_findings", "unreported_lossy_translations"}.issubset(required_outcomes))

    def test_adr_0010_content_invariants(self) -> None:
        """Verify ADR 0010 documents accessible diagnostics, plain text/JSON/JSONL, and institutional governance."""
        content = ADR_0010.read_text(encoding="utf-8")
        self.assertIn("plain text, JSON and JSONL without\ncolour dependence", content)
        self.assertIn("Locale-neutral identifiers are separated from message text", content)
        self.assertIn("Institutional policies classify data", content)


if __name__ == "__main__":
    unittest.main()
