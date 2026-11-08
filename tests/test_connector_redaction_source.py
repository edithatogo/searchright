"""Mutation coverage for the bounded lexical transport-redaction check."""
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from check_redaction_policy import connector_transport_errors


class ConnectorRedactionSourceTests(unittest.TestCase):
    def setUp(self):
        self.source = (ROOT / "crates/searchright-connectors/src/lib.rs").read_text()

    def test_actual_byte_reader_passes(self):
        self.assertEqual(connector_transport_errors(self.source), [])

    def test_unknown_reader_boundary_fails_closed(self):
        for marker in ("async fn fetch_bytes(", "fn decode_json(",
                       "let status = response.status()", "while let Some(chunk) = response"):
            with self.subTest(marker=marker):
                self.assertTrue(connector_transport_errors(self.source.replace(marker, "changed", 1)))

    def test_each_failure_requires_its_own_redaction_marker(self):
        marker = "endpoint and query details were redacted"
        start = self.source.index("async fn fetch_bytes(")
        first = self.source.index(marker, start)
        second = self.source.index(marker, first + len(marker))
        for position in (first, second):
            with self.subTest(position=position):
                changed = self.source[:position] + "removed" + self.source[position + len(marker):]
                self.assertTrue(connector_transport_errors(changed))

    def test_request_and_stream_error_stringification_rejected(self):
        for marker in ("network request failed before a response was available; ",
                       "response body retrieval failed; "):
            with self.subTest(marker=marker):
                changed = self.source.replace(marker, marker + "error.to_string()", 1)
                self.assertTrue(connector_transport_errors(changed))


if __name__ == "__main__":
    unittest.main()
