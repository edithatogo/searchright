from __future__ import annotations

import importlib.util
import contextlib
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_pr_track_scope", ROOT / "scripts" / "check_pr_track_scope.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
EXCEPTION_LABEL = MODULE.EXCEPTION_LABEL
check = MODULE.check


def event(body: str, labels: tuple[str, ...] = ()) -> dict[str, object]:
    return {
        "pull_request": {
            "body": body,
            "labels": [{"name": label} for label in labels],
        }
    }


def body(track: str, exception: str = "none", rationale: str = "none") -> str:
    return f"""- Conductor track: `{track}`
- Multi-track exception: `none`
- Exception tracks (only when absolutely inseparable): `{exception}`
- Why the work cannot be split (required for an exception): `{rationale}`
"""


def single_track_passes() -> None:
    receipt = check(event(body("10")), ["conductor/tracks/10-mcp-mvp/plan.md", "src/lib.rs"])
    assert receipt["status"] == "passed"


def literal_escaped_newlines_are_normalized() -> None:
    receipt = check(event(body("31").replace("\n", "\\n")), [])
    assert receipt["status"] == "passed"


def single_track_rejects_another_track_path() -> None:
    receipt = check(
        event(body("10")),
        ["conductor/tracks/10-mcp-mvp/plan.md", "conductor/tracks/16-quality/plan.md"],
    )
    assert receipt["status"] == "failed"


def multi_requires_label_and_justification() -> None:
    receipt = check(event(body("MULTI", "10, 16", "too short")), [])
    assert receipt["status"] == "failed"
    assert len(receipt["errors"]) == 2


def necessary_multi_track_exception_passes() -> None:
    receipt = check(
        event(
            body(
                "MULTI",
                "10, 16",
                "The shared protocol baseline and its admission gate must change atomically.",
            ),
            (EXCEPTION_LABEL,),
        ),
        ["conductor/tracks/10-mcp-mvp/plan.md", "conductor/tracks/16-quality/plan.md"],
    )
    assert receipt["status"] == "passed"


def single_track_rejects_exception_label() -> None:
    receipt = check(event(body("10"), (EXCEPTION_LABEL,)), [])
    assert receipt["status"] == "failed"


class TrackScopeTests(unittest.TestCase):
    def test_single_track_passes(self) -> None:
        single_track_passes()

    def test_literal_escaped_newlines_are_normalized(self) -> None:
        literal_escaped_newlines_are_normalized()

    def test_single_track_rejects_another_track_path(self) -> None:
        single_track_rejects_another_track_path()

    def test_multi_requires_label_and_justification(self) -> None:
        multi_requires_label_and_justification()

    def test_necessary_multi_track_exception_passes(self) -> None:
        necessary_multi_track_exception_passes()

    def test_single_track_rejects_exception_label(self) -> None:
        single_track_rejects_exception_label()

    def test_real_single_and_multiple_page_shapes_enforce_scope(self) -> None:
        first = {"filename": "conductor/tracks/10-mcp-mvp/plan.md"}
        second = {"filename": "conductor/tracks/16-quality/plan.md"}
        for payload in [[first, second], [[first, second]], [[first], [second]]]:
            with self.subTest(payload=payload):
                files = MODULE.changed_files(payload)
                self.assertEqual(files, [first["filename"], second["filename"]])
                receipt = check(event(body("10")), files)
                self.assertEqual(receipt["status"], "failed")
                self.assertEqual(receipt["changed_track_paths"], ["10", "16"])

    def test_single_record_page_preserves_track(self) -> None:
        record = {"filename": "conductor/tracks/10-mcp-mvp/plan.md"}
        for payload in [[record], [[record]]]:
            receipt = check(event(body("10")), MODULE.changed_files(payload))
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["changed_track_paths"], ["10"])

    def test_malformed_empty_and_nested_payloads_fail_closed(self) -> None:
        record = {"filename": "src/lib.rs"}
        payloads = [None, {}, record, "files", 1, [], [[]], [[record], []],
                    [[[record]]], [record, [record]], [None], [[None]], [[1]]]
        for payload in payloads:
            with self.subTest(payload=payload), self.assertRaises(ValueError):
                MODULE.changed_files(payload)

    def test_missing_and_nonstring_filenames_fail_closed(self) -> None:
        for record in [{}, *({"filename": value} for value in [None, "", " ", 1, True, [], {}])]:
            for payload in [[record], [[record]]]:
                with self.subTest(payload=payload), self.assertRaises(ValueError):
                    MODULE.changed_files(payload)

    def test_filename_bytes_are_not_coerced_or_trimmed(self) -> None:
        filename = "directory with spaces/é.txt "
        self.assertEqual(MODULE.changed_files([[{"filename": filename}]]), [filename])

    def test_renames_preserve_both_paths_in_flat_and_page_shapes(self) -> None:
        previous = "conductor/tracks/16-quality/plan.md"
        for destination in ["conductor/tracks/10-mcp-mvp/plan.md", "docs/moved-plan.md"]:
            record = {"filename": destination, "previous_filename": previous, "status": "renamed"}
            for payload in [[record], [[record]], [[{"filename": "README.md"}], [record]]]:
                with self.subTest(destination=destination, payload=payload):
                    files = MODULE.changed_files(payload)
                    self.assertIn(previous, files)
                    self.assertIn(destination, files)
                    receipt = check(event(body("10")), files)
                    self.assertEqual(receipt["status"], "failed")
                    self.assertIn("16", receipt["changed_track_paths"])

    def test_same_track_rename_passes(self) -> None:
        record = {"filename": "conductor/tracks/10-mcp-mvp/new.md",
                  "previous_filename": "conductor/tracks/10-mcp-mvp/old.md", "status": "renamed"}
        for payload in [[record], [[record]]]:
            receipt = check(event(body("10")), MODULE.changed_files(payload))
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["changed_track_paths"], ["10"])

    def test_cross_track_rename_requires_and_accepts_valid_multi_exception(self) -> None:
        record = {"filename": "conductor/tracks/10-mcp-mvp/new.md",
                  "previous_filename": "conductor/tracks/16-quality/old.md", "status": "renamed"}
        declaration = event(body("MULTI", "10, 16",
                                 "The shared protocol baseline and its admission gate must change atomically."),
                            (EXCEPTION_LABEL,))
        for payload in [[record], [[record]]]:
            receipt = check(declaration, MODULE.changed_files(payload))
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["changed_track_paths"], ["10", "16"])

    def test_rename_missing_or_malformed_previous_filename_fails_closed(self) -> None:
        base = {"filename": "docs/moved-plan.md", "status": "renamed"}
        records = [base, *({**base, "previous_filename": value}
                          for value in [None, "", " ", 1, True, [], {}])]
        for record in records:
            for payload in [[record], [[record]]]:
                with self.subTest(payload=payload), self.assertRaisesRegex(ValueError, "previous_filename"):
                    MODULE.changed_files(payload)

    def test_main_reports_nonzero_diagnostic_for_bad_metadata(self) -> None:
        for raw in ["{broken", "[]", "[[{}]]", "[[[{}]]]"]:
            output = io.StringIO()
            with self.subTest(raw=raw), patch("sys.argv", ["scope", "--event", "event.json", "--files-json", "files.json"]):
                with patch.object(Path, "read_text", side_effect=[json.dumps(event(body("10"))), raw]):
                    with contextlib.redirect_stderr(output), self.assertRaises(SystemExit) as error:
                        MODULE.main()
            self.assertEqual(error.exception.code, 2)
            self.assertIn("invalid changed-file metadata", output.getvalue())


if __name__ == "__main__":
    unittest.main()
