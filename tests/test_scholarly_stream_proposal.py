"""Structural proposal tests only; no runtime or scientific compatibility claim."""
from pathlib import Path
import copy
import hashlib
import json
import unittest
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "contracts/proposals/scholarly-stream.v1.schema.json"
FIXTURE = ROOT / "contracts/proposals/fixtures/scholarly-stream-upsert.json"

class ScholarlyStreamProposal(unittest.TestCase):
    def setUp(self):
        self.schema = json.loads(SCHEMA.read_text())
        self.event = json.loads(FIXTURE.read_text())
        self.validator = Draft202012Validator(self.schema, format_checker=FormatChecker())

    def test_schema_and_fixture(self):
        Draft202012Validator.check_schema(self.schema)
        self.validator.validate(self.event)

    def test_ascii_integer_fixture_digest(self):
        # This fixture contains ASCII strings and integer date parts only.
        # It is not a general RFC8785 canonicalizer or conformance test.
        payload = json.dumps(self.event["change"], sort_keys=True, separators=(",", ":")).encode()
        self.assertEqual(hashlib.sha256(payload).hexdigest(), self.event["payload_digest"]["hex"])

    def test_reject_bad_envelopes(self):
        for key, value in [("version", "999.0.0"), ("sequence", 1), ("sequence", "-1"), ("observed_at", "yesterday")]:
            with self.subTest(key=key, value=value):
                invalid = copy.deepcopy(self.event)
                invalid[key] = value
                self.assertTrue(list(self.validator.iter_errors(invalid)))
        invalid = copy.deepcopy(self.event)
        del invalid["rights"]
        self.assertTrue(list(self.validator.iter_errors(invalid)))

    def test_nonpublication_uses_typed_artifact(self):
        event = copy.deepcopy(self.event)
        event["change"]["object"] = {"kind": "grant", "key": "fixture:grant", "record": event["source_revision"], "extensions": []}
        self.validator.validate(event)
        event["change"]["object"]["csl"] = {"id": "grant", "type": "article-journal"}
        self.assertTrue(list(self.validator.iter_errors(event)))

    def test_explicit_change_operations(self):
        for op in ["retract", "correct", "delete"]:
            with self.subTest(op=op):
                event = copy.deepcopy(self.event)
                event["change"] = {"op": op, "target": "fixture:paper-1", "reason": "synthetic test", "replacement": None}
                self.validator.validate(event)
        event["change"]["op"] = "execute"
        self.assertTrue(list(self.validator.iter_errors(event)))

if __name__ == "__main__":
    unittest.main()
