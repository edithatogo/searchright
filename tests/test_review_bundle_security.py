"""Adversarial tests for review-bundle filesystem and archive boundaries."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "searchright_review_bundle", ROOT / "scripts" / "review_bundle.py"
)
if SPEC is None or SPEC.loader is None:  # pragma: no cover - import bootstrap guard
    raise RuntimeError("could not load review_bundle.py")
review_bundle = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = review_bundle
SPEC.loader.exec_module(review_bundle)


class ReviewBundleSecurityTests(unittest.TestCase):
    def test_plan_source_symlink_is_rejected_before_resolution(self) -> None:
        with tempfile.TemporaryDirectory(prefix="searchright-bundle-symlink-") as value:
            root = Path(value)
            target = root / "target.json"
            target.write_text("{}\n", encoding="utf-8")
            link = root / "link.json"
            try:
                link.symlink_to(target)
            except OSError as exc:
                self.skipTest(f"symlink creation unavailable on this platform: {exc}")
            with self.assertRaises(review_bundle.BundleError):
                review_bundle.resolve_plan_source(root, "link.json")

    def test_oversized_member_is_rejected_before_expansion(self) -> None:
        with tempfile.TemporaryDirectory(prefix="searchright-bundle-size-") as value:
            archive_path = Path(value) / "oversized.srpack"
            info = zipfile.ZipInfo("payload/oversized.bin")
            info.external_attr = 0o100644 << 16
            with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_STORED) as archive:
                archive.writestr(info, b"xx")

            original_limit = review_bundle.VERIFY_MAX_FILE_BYTES
            review_bundle.VERIFY_MAX_FILE_BYTES = 1
            try:
                receipt = review_bundle.verify(archive_path)
            finally:
                review_bundle.VERIFY_MAX_FILE_BYTES = original_limit
            self.assertEqual(receipt["status"], "failed")
            self.assertIn("archive member exceeds", " ".join(receipt["errors"]))


if __name__ == "__main__":
    unittest.main()
