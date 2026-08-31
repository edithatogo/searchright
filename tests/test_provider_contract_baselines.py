"""Offline format/source admission and synthetic XML shape regressions."""
from __future__ import annotations

import hashlib
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("provider_baselines", ROOT / "scripts/check_provider_contract_baselines.py")
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

XML = b'<PubmedArticleSet><PubmedArticle><PMID>123</PMID><Title>A &amp; <i>B</i></Title></PubmedArticle></PubmedArticleSet>'


class ProviderBaselineTests(unittest.TestCase):
    def item(self, raw: bytes = XML) -> dict:
        return {
            "format": "xml", "xml_root": "PubmedArticleSet",
            "fixture_sha256": hashlib.sha256(raw).hexdigest(),
            "shape_assertions": [
                {"path": "PubmedArticle/PMID", "equals": "123"},
                {"path": "PubmedArticle/Title", "equals": "A & B", "nonempty": True},
            ],
        }

    def test_xml_digest_root_and_mixed_text_shape(self) -> None:
        self.assertEqual(MODULE.fixture_errors(self.item(), XML), [])

    def test_digest_and_root_drift_are_rejected(self) -> None:
        item = self.item()
        item["fixture_sha256"] = "0" * 64
        self.assertTrue(MODULE.fixture_errors(item, XML))
        item = self.item()
        item["xml_root"] = "WrongRoot"
        self.assertTrue(MODULE.fixture_errors(item, XML))

    def test_missing_ambiguous_and_invalid_paths_are_rejected(self) -> None:
        for path in ["Missing", ".//PMID", "../PMID", "", "PubmedArticle/*"]:
            with self.subTest(path=path):
                item = self.item()
                item["shape_assertions"] = [{"path": path, "nonempty": True}]
                self.assertTrue(MODULE.fixture_errors(item, XML))
        duplicate = XML.replace(b"<PMID>123</PMID>", b"<PMID>123</PMID><PMID>123</PMID>")
        self.assertTrue(MODULE.fixture_errors(self.item(duplicate), duplicate))

    def test_dtd_and_custom_entities_rejected_before_xml_parser(self) -> None:
        for raw in [b'<!DOCTYPE x SYSTEM "https://example.invalid/dtd">' + XML,
                    b'<!DOCTYPE x [<!ENTITY e "value">]>' + XML,
                    XML.replace(b"123", b"&custom;")]:
            with self.subTest(raw=raw), patch.object(MODULE.ET, "fromstring") as parse:
                self.assertTrue(MODULE.fixture_errors(self.item(raw), raw))
                parse.assert_not_called()

    def test_oversize_and_invalid_encoding_are_rejected_before_parse(self) -> None:
        for raw in [b" " * (MODULE.MAX_XML_FIXTURE_BYTES + 1), b"\xff"]:
            with patch.object(MODULE.ET, "fromstring") as parse:
                self.assertTrue(MODULE.fixture_errors(self.item(raw), raw))
                parse.assert_not_called()

    def test_unknown_format_rejected_and_json_default_preserved(self) -> None:
        item = self.item()
        item["format"] = "yaml"
        self.assertTrue(MODULE.fixture_errors(item, XML))
        raw = b'{"items":[{"id":"123"}]}'
        item = {"fixture_sha256": hashlib.sha256(raw).hexdigest(),
                "shape_assertions": [{"pointer": "/items/0/id", "equals": "123"}]}
        self.assertEqual(MODULE.fixture_errors(item, raw), [])

    def test_only_declared_connector_sources_are_admitted(self) -> None:
        self.assertEqual(MODULE.parser_source({}), MODULE.CONNECTOR_SOURCE)
        for source in ["crates/searchright-connectors/src/lib.rs", "crates/searchright-connectors/src/efetch.rs"]:
            self.assertEqual(MODULE.parser_source({"parser_source": source}), ROOT / source)
        for source in ["../lib.rs", "/etc/passwd", "scripts/check_provider_contract_baselines.py", "crates/searchright-connectors/src/../src/lib.rs", None, [], ""]:
            with self.subTest(source=source), self.assertRaises(ValueError):
                MODULE.parser_source({"parser_source": source})


if __name__ == "__main__":
    unittest.main()
