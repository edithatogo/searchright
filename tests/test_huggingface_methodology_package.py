"""Tests for the deterministic Hugging Face methodology-fixture package."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "build_hf_methodology_fixtures",
    ROOT / "scripts" / "build_hf_methodology_fixtures.py",
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def tree_bytes(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class HuggingFaceMethodologyPackageTests(unittest.TestCase):
    def test_repeated_builds_are_byte_identical(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            first = base / "first"
            second = base / "second"
            first_manifest = MODULE.build(first)
            second_manifest = MODULE.build(second)
            self.assertEqual(first_manifest, second_manifest)
            self.assertEqual(tree_bytes(first), tree_bytes(second))

    def test_output_contains_only_manifest_card_and_allowlisted_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "dataset"
            manifest = MODULE.build(output)
            expected = {entry["path"] for entry in manifest["files"]} | {
                "dataset-manifest.json"
            }
            self.assertEqual(set(tree_bytes(output)), expected)
            self.assertNotIn(
                "benchmarks/methodology/fixtures/sealed/manifest.json",
                expected,
            )
            self.assertFalse(any("sealed" in path.lower() for path in expected))

    def test_manifest_hashes_and_root_reject_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "dataset"
            MODULE.build(output)
            target = output / "benchmarks/methodology/fixtures/validation/screening-cases.json"
            target.write_text("{}\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "integrity mismatch"):
                MODULE.verify_output(output)

    def test_extra_file_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "dataset"
            MODULE.build(output)
            (output / "unexpected.txt").write_text("unexpected\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "output paths differ"):
                MODULE.verify_output(output)

    def test_secret_and_email_patterns_are_rejected(self) -> None:
        for label, payload in (
            ("huggingface", b"hf_abcdefghijklmnopqrstuvwxyz123456"),
            ("github", b"ghp_abcdefghijklmnopqrstuvwxyz123456"),
            ("email", b"person@example.org"),
            ("private-key", b"-----BEGIN PRIVATE KEY-----"),
        ):
            with self.subTest(label=label):
                with self.assertRaises(ValueError):
                    MODULE.scan_payload("fixture.txt", payload)

    def test_benchmark_authority_and_leakage_boundaries_are_retained(self) -> None:
        manifest = MODULE.render_manifest()
        card = (
            ROOT
            / "registry/huggingface/searchright-methodology-fixtures/README.md"
        ).read_text(encoding="utf-8").lower()
        methodology = json.loads(
            (ROOT / "benchmarks/methodology/manifest.json").read_text(
                encoding="utf-8"
            )
        )
        screening = next(
            task
            for task in methodology["tasks"]
            if task["id"] == "screening_prioritisation"
        )
        self.assertIn("advisory", screening["claim_boundary"].lower())
        self.assertIn("may not use", card)
        self.assertIn("irreversible final exclusion", card)
        self.assertFalse(methodology["leakage_controls"]["sealed_labels_committed"])
        self.assertFalse(
            methodology["leakage_controls"]["sealed_labels_available_to_agents"]
        )
        self.assertEqual(manifest["status"], "source_package_prepared_not_published")

    def test_registry_status_does_not_claim_publication(self) -> None:
        status = json.loads(
            (
                ROOT
                / "registry/huggingface/searchright-methodology-fixtures/status.json"
            ).read_text(encoding="utf-8")
        )
        self.assertEqual(status["status"], "prepared_not_published")
        self.assertIsNone(status["current_hub_revision"])
        self.assertIn("not claimed", status["claim_boundary"].lower())


if __name__ == "__main__":
    unittest.main()
