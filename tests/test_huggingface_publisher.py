"""Safety tests for the protected Hugging Face publisher."""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
BUILD_SPEC = importlib.util.spec_from_file_location(
    "build_hf_methodology_fixtures",
    ROOT / "scripts" / "build_hf_methodology_fixtures.py",
)
assert BUILD_SPEC is not None and BUILD_SPEC.loader is not None
BUILD = importlib.util.module_from_spec(BUILD_SPEC)
BUILD_SPEC.loader.exec_module(BUILD)

PUBLISH_SPEC = importlib.util.spec_from_file_location(
    "publish_hf_methodology_fixtures",
    ROOT / "scripts" / "publish_hf_methodology_fixtures.py",
)
assert PUBLISH_SPEC is not None and PUBLISH_SPEC.loader is not None
PUBLISH = importlib.util.module_from_spec(PUBLISH_SPEC)
PUBLISH_SPEC.loader.exec_module(PUBLISH)


class HuggingFacePublisherTests(unittest.TestCase):
    def test_dry_run_is_network_free_and_writes_bounded_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            package = base / "package"
            receipt_path = base / "receipt.json"
            manifest = BUILD.build(package)
            with mock.patch.dict(os.environ, {}, clear=True):
                receipt = PUBLISH.publish(
                    package_dir=package,
                    repo_id="edithatogo/searchright-methodology-fixtures",
                    visibility="public",
                    receipt_path=receipt_path,
                    apply=False,
                )
            self.assertEqual(receipt["status"], "dry_run_passed")
            self.assertEqual(
                receipt["dataset_root_sha256"], manifest["dataset_root_sha256"]
            )
            stored = json.loads(receipt_path.read_text(encoding="utf-8"))
            self.assertEqual(stored, receipt)
            self.assertNotIn("token", json.dumps(stored).lower())
            self.assertIsNone(stored["remote_revision"])

    def test_wrong_repository_is_rejected_before_network_access(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            package = base / "package"
            BUILD.build(package)
            with self.assertRaisesRegex(ValueError, "must exactly match"):
                PUBLISH.publish(
                    package_dir=package,
                    repo_id="edithatogo/not-searchright",
                    visibility="public",
                    receipt_path=base / "receipt.json",
                    apply=False,
                )

    def test_apply_requires_second_environment_opt_in(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            package = base / "package"
            BUILD.build(package)
            with mock.patch.dict(
                os.environ,
                {"HF_TOKEN": "hf_syntheticvalueonlyfortest123456"},
                clear=True,
            ):
                with self.assertRaisesRegex(PermissionError, "SEARCHRIGHT_HF_APPLY"):
                    PUBLISH.publish(
                        package_dir=package,
                        repo_id="edithatogo/searchright-methodology-fixtures",
                        visibility="public",
                        receipt_path=base / "receipt.json",
                        apply=True,
                    )

    def test_apply_requires_token_after_second_opt_in(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            package = base / "package"
            BUILD.build(package)
            with mock.patch.dict(
                os.environ,
                {"SEARCHRIGHT_HF_APPLY": "1"},
                clear=True,
            ):
                with self.assertRaisesRegex(PermissionError, "HF_TOKEN"):
                    PUBLISH.publish(
                        package_dir=package,
                        repo_id="edithatogo/searchright-methodology-fixtures",
                        visibility="public",
                        receipt_path=base / "receipt.json",
                        apply=True,
                    )


if __name__ == "__main__":
    unittest.main()
