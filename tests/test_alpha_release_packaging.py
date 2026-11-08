"""Unit tests for Track 18 alpha release source packaging and reproducibility checks."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_module(name: str, rel_path: str):
    path = ROOT / rel_path
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


PACKAGE_SOURCE = load_module("package_source", "scripts/package_source.py")
CHECK_REPRODUCIBILITY = load_module("check_packaging_reproducibility", "scripts/check_packaging_reproducibility.py")


class AlphaReleasePackagingTests(unittest.TestCase):
    def test_sha256_computes_expected_digest(self) -> None:
        with tempfile.NamedTemporaryFile(prefix="searchright-test-", delete=False) as f:
            f.write(b"searchright-alpha-release-deterministic-bytes\n")
            temp_path = Path(f.name)
        try:
            expected = hashlib.sha256(b"searchright-alpha-release-deterministic-bytes\n").hexdigest()
            self.assertEqual(PACKAGE_SOURCE.sha256(temp_path), expected)
            self.assertEqual(CHECK_REPRODUCIBILITY.digest(temp_path), expected)
        finally:
            temp_path.unlink(missing_ok=True)

    def test_epoch_respects_source_date_epoch(self) -> None:
        original = os.environ.get("SOURCE_DATE_EPOCH")
        try:
            os.environ["SOURCE_DATE_EPOCH"] = "1700000000"
            self.assertEqual(PACKAGE_SOURCE.epoch(), 1700000000)
        finally:
            if original is not None:
                os.environ["SOURCE_DATE_EPOCH"] = original
            else:
                os.environ.pop("SOURCE_DATE_EPOCH", None)

    def test_tracked_files_listing_excludes_forbidden_directories(self) -> None:
        members = PACKAGE_SOURCE.files()
        self.assertIsInstance(members, list)
        self.assertGreater(len(members), 100)
        for member in members:
            self.assertTrue(member.is_file())
            rel_parts = member.relative_to(ROOT).parts
            self.assertNotIn(".git", rel_parts)
            self.assertNotIn("target", rel_parts)
            self.assertNotIn("dist", rel_parts)
            self.assertNotIn("__pycache__", rel_parts)
        # Members must be sorted for deterministic archives
        self.assertEqual(members, sorted(members))

    def test_verify_members_identifies_discrepancies(self) -> None:
        manifest_paths = {"file1.txt", "file2.txt", "dir/file3.txt"}
        expected_archive = {"searchright/file1.txt", "searchright/file2.txt", "searchright/dir/file3.txt"}

        # Matching archive
        self.assertEqual(CHECK_REPRODUCIBILITY.verify_members(expected_archive, manifest_paths), [])

        # Missing member
        missing_archive = {"searchright/file1.txt", "searchright/dir/file3.txt"}
        missing_errors = CHECK_REPRODUCIBILITY.verify_members(missing_archive, manifest_paths)
        self.assertTrue(any("membership differs" in e and "file2.txt" in e for e in missing_errors))

        # Unexpected member
        extra_archive = expected_archive | {"searchright/unexpected.txt"}
        extra_errors = CHECK_REPRODUCIBILITY.verify_members(extra_archive, manifest_paths)
        self.assertTrue(any("membership differs" in e and "unexpected.txt" in e for e in extra_errors))

        # Leaked .git path
        git_archive = expected_archive | {"searchright/.git/config"}
        git_errors = CHECK_REPRODUCIBILITY.verify_members(git_archive, manifest_paths)
        self.assertTrue(any("archive contains .git content" in e for e in git_errors))

    def test_compare_archives_detects_mismatch(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sr-pkg-test-a-") as dir_a, \
             tempfile.TemporaryDirectory(prefix="sr-pkg-test-b-") as dir_b:
            path_a = Path(dir_a)
            path_b = Path(dir_b)

            manifest_content = json.dumps({"schema_version": "v1", "files": []}).encode("utf-8")
            (path_a / "searchright-source-manifest.json").write_bytes(manifest_content)
            (path_b / "searchright-source-manifest.json").write_bytes(manifest_content + b" ")

            (path_a / "searchright-source-packaging-receipt.json").write_bytes(b"receipt-a")
            (path_b / "searchright-source-packaging-receipt.json").write_bytes(b"receipt-b")

            # Create minimal valid zip archives with different contents
            with zipfile.ZipFile(path_a / "searchright-source.zip", "w") as z_a:
                z_a.writestr("searchright/a.txt", b"a")
            with zipfile.ZipFile(path_b / "searchright-source.zip", "w") as z_b:
                z_b.writestr("searchright/b.txt", b"b")

            # Create minimal valid tar archives with different contents
            with tarfile.open(path_a / "searchright-source.tar.gz", "w:gz") as t_a:
                pass
            with tarfile.open(path_b / "searchright-source.tar.gz", "w:gz") as t_b:
                pass

            errors = CHECK_REPRODUCIBILITY.compare_archives(path_a, path_b)
            self.assertTrue(any("non-reproducible artifact: searchright-source-manifest.json" in e for e in errors))
            self.assertTrue(any("non-reproducible artifact: searchright-source-packaging-receipt.json" in e for e in errors))

    def test_package_source_receipt_and_manifest_schema(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sr-pkg-test-") as temp_dir:
            out_path = Path(temp_dir)
            receipt = PACKAGE_SOURCE.package_source(
                output_dir=out_path,
                prefix="testpkg",
                source_epoch=1700000000,
                verify_clean=False,
            )

            self.assertEqual(receipt.get("schema_version"), "org.searchright.source-packaging-receipt.v1")
            self.assertEqual(receipt.get("source_date_epoch"), 1700000000)
            self.assertGreater(receipt.get("file_count", 0), 0)

            # Verify files were generated
            zip_file = out_path / "testpkg-source.zip"
            tar_file = out_path / "testpkg-source.tar.gz"
            manifest_file = out_path / "testpkg-source-manifest.json"
            receipt_file = out_path / "testpkg-source-packaging-receipt.json"

            self.assertTrue(zip_file.is_file())
            self.assertTrue(tar_file.is_file())
            self.assertTrue(manifest_file.is_file())
            self.assertTrue(receipt_file.is_file())

            # Validate manifest JSON structure
            manifest = json.loads(manifest_file.read_text(encoding="utf-8"))
            self.assertEqual(manifest.get("schema_version"), "org.searchright.source-archive-manifest.v1")
            self.assertEqual(manifest.get("source_date_epoch"), 1700000000)
            self.assertIsInstance(manifest.get("files"), list)
            self.assertGreater(len(manifest["files"]), 0)

            first_file = manifest["files"][0]
            self.assertIn("path", first_file)
            self.assertIn("size", first_file)
            self.assertIn("sha256", first_file)
            self.assertIn("executable", first_file)

            # Validate tar archive header normalization (uid=0, gid=0, mtime=source_epoch)
            with tarfile.open(tar_file, "r:gz") as tar:
                members = tar.getmembers()
                self.assertGreater(len(members), 0)
                first_tar_member = members[0]
                self.assertEqual(first_tar_member.uid, 0)
                self.assertEqual(first_tar_member.gid, 0)
                self.assertEqual(first_tar_member.uname, "")
                self.assertEqual(first_tar_member.gname, "")
                self.assertEqual(first_tar_member.mtime, 1700000000)

            # Validate zip archive
            with zipfile.ZipFile(zip_file, "r") as z:
                z_namelist = z.namelist()
                self.assertGreater(len(z_namelist), 0)
                self.assertTrue(all(name.startswith("testpkg/") for name in z_namelist))


if __name__ == "__main__":
    unittest.main()
