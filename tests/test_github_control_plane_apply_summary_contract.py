from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = (
    ROOT
    / "contracts"
    / "json-schema"
    / "github-control-plane-apply-summary.v1.schema.json"
)
EXAMPLE_PATH = ROOT / "contracts" / "examples" / "github-control-plane-apply-summary.json"
RECEIPT_PATH = ROOT / "verification" / "receipts" / "track-31-control-plane-apply.json"


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


class GitHubControlPlaneApplySummaryContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        schema = load_json(SCHEMA_PATH)
        Draft202012Validator.check_schema(schema)
        cls.validator = Draft202012Validator(schema, format_checker=FormatChecker())
        cls.example = load_json(EXAMPLE_PATH)

    def test_canonical_example_and_preserved_receipt_validate(self) -> None:
        self.validator.validate(self.example)
        self.validator.validate(load_json(RECEIPT_PATH))

    def test_fail_closed_evidence_invariants_reject_invalid_values(self) -> None:
        invalid_cases = [
        (("source_revision",), "37acbd0"),
        (("workflow_run", "environment"), "unprotected"),
        (("issue_sync", "delete_operations"), 1),
        (("project_sync", "remaining_after_run"), 1),
        (("audit", "content_drift"), 1),
        (("audit", "mutation_operations"), 1),
        (("artifact", "digest"), "sha256:not-a-digest"),
        ]
        for path, invalid_value in invalid_cases:
            with self.subTest(path=path, invalid_value=invalid_value):
                candidate = copy.deepcopy(self.example)
                target: dict[str, object] = candidate
                for key in path[:-1]:
                    target = target[key]  # type: ignore[assignment]
                target[path[-1]] = invalid_value
                self.assertTrue(list(self.validator.iter_errors(candidate)))

    def test_unknown_receipt_fields_are_rejected(self) -> None:
        candidate = copy.deepcopy(self.example)
        candidate["evidence_promoted"] = True

        self.assertTrue(list(self.validator.iter_errors(candidate)))


if __name__ == "__main__":
    unittest.main()
