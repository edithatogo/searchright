"""Independent qualification of the pinned CSL input schema and Searchright IDs."""
from __future__ import annotations

import json
import sys
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import check_csl_corpus as gate


class CslCorpusSchemaTests(unittest.TestCase):
    def test_pinned_schema_and_valid_bibliography(self) -> None:
        raw = b'[{"id":"a","type":"article-journal","author":[{"family":"Example"}],"issued":{"date-parts":[[2025,1,2]]},"custom":{"source_id":"synthetic"}}]'
        self.assertEqual(gate.check(raw), 1)
        self.assertEqual(gate.check(b"[]"), 0)

    def test_invalid_types_and_fields_are_not_admitted(self) -> None:
        cases = [
            [{"id": "a", "type": "not-a-csl-type"}],
            [{"id": "a", "type": "report", "title": 5}],
            [{"id": "a", "type": "report", "affiliations": []}],
            [{"id": "a", "type": "report", "issued": {"date-parts": [[]]}}],
            [{"id": "a", "type": "report", "author": [{"unknown": "x"}]}],
            {"id": "a", "type": "report"},
            [{"id": "a"}],
        ]
        for value in cases:
            with self.subTest(value=value), self.assertRaises(gate.CorpusError):
                gate.check(json.dumps(value).encode())

    def test_searchright_identity_is_stricter_than_upstream(self) -> None:
        for ids in ([1], [" "], ["a", "a"]):
            with self.subTest(ids=ids), self.assertRaises(gate.CorpusError):
                gate.check(json.dumps([{"id": i, "type": "report"} for i in ids]).encode())
        self.assertEqual(gate.check(b'[{"id":"A","type":"report"},{"id":"a","type":"report"}]'), 2)

    def test_ambiguous_or_non_json_inputs_are_rejected_before_schema(self) -> None:
        cases = [
            b'[{"id":"a","id":"b","type":"report"}]',
            b'[{"id":"a","type":"report","custom":{"x":1,"x":2}}]',
            b'[{"id":"a","type":"report","volume":NaN}]',
            b'[{"id":"a","type":"report","volume":Infinity}]',
            b'[{"id":"a","type":"report","volume":1e999}]',
            b'[{"id":"a","type":"report","title":"\\ud800"}]',
            b'\xff',
        ]
        for raw in cases:
            with self.subTest(raw=raw), self.assertRaises(gate.CorpusError):
                gate.check(raw)

    def test_schema_pin_rejects_modified_bytes_even_if_json_is_equivalent(self) -> None:
        with patch.object(Path, "read_bytes", return_value=gate.SCHEMA.read_bytes() + b" "):
            with self.assertRaisesRegex(gate.CorpusError, "schema digest"):
                gate.check(b"[]")

    def test_preflight_limit_and_cli_exit_status(self) -> None:
        with patch.object(gate, "MAX_BYTES", 1):
            with self.assertRaisesRegex(gate.CorpusError, "byte limit"):
                gate.check(b"[]")
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "corpus.json"
            for raw, expected in [(b"[]", 0), (b"{}", 1)]:
                source.write_bytes(raw)
                completed = subprocess.run(
                    [sys.executable, str(ROOT / "scripts/check_csl_corpus.py"), str(source)],
                    capture_output=True, text=True, check=False,
                )
                self.assertEqual(completed.returncode, expected)
                self.assertEqual(source.read_bytes(), raw)

    def test_error_does_not_echo_rejected_record_value(self) -> None:
        with self.assertRaises(gate.CorpusError) as result:
            gate.check(b'[{"id":"private-identifier","type":"invalid-secret-value"}]')
        self.assertNotIn("private-identifier", str(result.exception))
        self.assertNotIn("invalid-secret-value", str(result.exception))


if __name__ == "__main__":
    unittest.main()
