"""Network-free regression tests for contract generation and parity evidence."""
from __future__ import annotations

import sys
import unittest
import contextlib
import io
import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import check_rust_schema_parity as parity
import generate_contract_bindings as bindings


class ParityTests(unittest.TestCase):
    def test_property_names_are_not_annotations(self):
        left = {"properties": {"title": {"type": "string"}}}
        right = {"properties": {"title": {"type": "integer"}}}
        self.assertNotEqual(parity.canonicalise(left), parity.canonicalise(right))

    def test_instance_data_and_dialect_are_preserved(self):
        schema = {"const": {"title": "value"}, "enum": [{"description": "value"}],
                  "$schema": "dialect", "$id": "base"}
        self.assertEqual(parity.canonicalise(schema), schema)

    def test_only_schema_annotations_are_removed(self):
        schema = {"title": "annotation", "$defs": {"description": {
            "description": "annotation", "type": "string"}}}
        self.assertEqual(parity.canonicalise(schema),
                         {"$defs": {"description": {"type": "string"}}})

    def test_existing_mismatch_changes_digest(self):
        first, second = {"minimum": 1}, {"minimum": 100}
        self.assertEqual(parity.difference_paths({}, first), parity.difference_paths({}, second))
        self.assertNotEqual(parity.semantic_digest(first), parity.semantic_digest(second))

    def test_cli_rejects_drift_at_an_existing_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / "catalog.json"
            catalog.write_text(json.dumps({"entries": [{"id": "example", "schema": "schema.json"}]}))
            (root / "schema.json").write_text(json.dumps({"minimum": 0}))
            exported = SimpleNamespace(returncode=0, stdout=json.dumps({"example": {"minimum": 1}}))
            with patch.object(parity, "ROOT", root), patch.object(parity, "CATALOG", catalog), \
                    patch.object(parity, "REPORT", root / "report.json"), \
                    patch.object(parity.subprocess, "run", return_value=exported), \
                    contextlib.redirect_stdout(io.StringIO()):
                with patch.object(sys, "argv", ["parity", "--write"]):
                    self.assertEqual(parity.main(), 0)
                with patch.object(sys, "argv", ["parity", "--check"]):
                    self.assertEqual(parity.main(), 0)
                    exported.stdout = json.dumps({"example": {"minimum": 100}})
                    self.assertEqual(parity.main(), 1)


class BindingTests(unittest.TestCase):
    def test_python_dynamic_map_is_not_a_typed_dict(self):
        schema = {"type": "object", "additionalProperties": {"type": "string"}}
        objects = {}
        bindings.collect_python_objects(schema, "#", "Map", objects)
        self.assertEqual(objects, {})
        self.assertEqual(bindings.python_type(schema, {}, {}, "#"), "dict[str, str]")

    def test_typescript_union_preserves_required_base(self):
        schema = {"type": "object", "properties": {"id": {"type": "string"}},
                  "required": ["id"], "oneOf": [{"required": ["input"]}, {"required": ["output"]}]}
        result = bindings.typescript_type(schema, {})
        self.assertIn('readonly "id": string;', result)
        self.assertIn('readonly "input":', result)
        self.assertIn('readonly "output":', result)


if __name__ == "__main__":
    unittest.main()
