"""Tests for Track 15 GitHub estate audit, pattern scanning, and companion change packets."""
from __future__ import annotations

import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import audit_search_code
import check_companion_change_packets


class TestAuditSearchCode(unittest.TestCase):
    def setUp(self) -> None:
        self.patterns = json.loads((ROOT / "migration/estate/patterns.json").read_text(encoding="utf-8"))[
            "patterns"
        ]

    def test_self_test_passes_with_canonical_patterns(self) -> None:
        self.assertTrue(audit_search_code.self_test(self.patterns))

    def test_self_test_fails_with_empty_or_incomplete_patterns(self) -> None:
        self.assertFalse(audit_search_code.self_test([]))
        incomplete = [p for p in self.patterns if p["id"] == "pubmed_endpoint"]
        self.assertFalse(audit_search_code.self_test(incomplete))

    def test_scan_detects_patterns_and_skips_ignored_directories(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)

            # File with PubMed endpoint
            py_file = tmp_path / "review_service.py"
            py_file.write_text(
                "import requests\nesearch = 'https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi'\n",
                encoding="utf-8",
            )

            # File with PRISMA logging
            doc_file = tmp_path / "protocol.md"
            doc_file.write_text(
                "# PRISMA Flow\nrecords_screened = 150\nduplicates_removed = 20\n",
                encoding="utf-8",
            )

            # File with search strategy
            query_file = tmp_path / "search_strategy.sh"
            query_file.write_text(
                'QUERY="PubMed AND (cancer OR oncology)"\n',
                encoding="utf-8",
            )

            # Ignored suffix (binary file)
            bin_file = tmp_path / "blob.bin"
            bin_file.write_bytes(b"https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi")

            # Files in ignored directories (.git, target, node_modules, .venv)
            for ignored_dir in (".git", "target", "node_modules", ".venv"):
                sub_dir = tmp_path / ignored_dir / "nested"
                sub_dir.mkdir(parents=True, exist_ok=True)
                ignored_file = sub_dir / "hidden.py"
                ignored_file.write_text(
                    "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi PubMed AND cancer\n",
                    encoding="utf-8",
                )

            findings = audit_search_code.scan(tmp_path, self.patterns)
            found_paths = {item["path"] for item in findings}

            # Verify expected files detected
            self.assertIn("review_service.py", found_paths)
            self.assertIn("protocol.md", found_paths)
            self.assertIn("search_strategy.sh", found_paths)
            self.assertEqual(len(findings), 3)

            # Verify skipped paths
            for item in findings:
                self.assertNotIn(".git", item["path"])
                self.assertNotIn("target", item["path"])
                self.assertNotIn("node_modules", item["path"])
                self.assertNotIn(".venv", item["path"])
                self.assertNotIn("blob.bin", item["path"])

            # Verify hit structures
            py_finding = next(f for f in findings if f["path"] == "review_service.py")
            self.assertTrue(any(hit["pattern_id"] == "pubmed_endpoint" for hit in py_finding["hits"]))
            self.assertEqual(len(py_finding["sha256"]), 64)

    def test_audit_search_code_main_self_test(self) -> None:
        buf = io.StringIO()
        with patch.object(sys, "argv", ["audit_search_code.py", "--self-test"]), contextlib.redirect_stdout(buf):
            ret = audit_search_code.main()
        self.assertEqual(ret, 0)
        receipt = json.loads(buf.getvalue())
        self.assertEqual(receipt["status"], "passed")
        self.assertGreaterEqual(receipt["patterns"], 12)


class TestCompanionChangePackets(unittest.TestCase):
    def run_main(self) -> tuple[int, dict]:
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            code = check_companion_change_packets.main()
        return code, json.loads(buf.getvalue())

    def test_checked_in_companion_packets_and_estate_registers_pass(self) -> None:
        code, receipt = self.run_main()
        self.assertEqual(code, 0)
        self.assertEqual(receipt["status"], "passed")
        self.assertEqual(receipt["errors"], [])
        self.assertEqual(receipt["packets_checked"], 11)
        self.assertEqual(receipt["remote_mutations"], 0)
        self.assertEqual(receipt["schema_version"], "org.searchright.companion-change-packet-receipt.v1")

    def test_adversarial_rejection_of_remote_mutation_permitted(self) -> None:
        packet_path = ROOT / "migration/companion-repositories/citeweft.json"
        original_data = json.loads(packet_path.read_text(encoding="utf-8"))
        mutated_data = dict(original_data)
        mutated_data["remote_mutation_permitted"] = True

        real_read_text = Path.read_text

        def mock_read_text(path_obj, *args, **kwargs):
            if path_obj == packet_path:
                return json.dumps(mutated_data)
            return real_read_text(path_obj, *args, **kwargs)

        with patch.object(Path, "read_text", mock_read_text):
            code, receipt = self.run_main()
        self.assertEqual(code, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any("remote mutation must be false" in err for err in receipt["errors"]))

    def test_adversarial_rejection_of_change_automatic_apply(self) -> None:
        packet_path = ROOT / "migration/companion-repositories/citeweft.json"
        original_data = json.loads(packet_path.read_text(encoding="utf-8"))
        mutated_data = json.loads(json.dumps(original_data))
        mutated_data["changes"][0]["automatic_apply"] = True

        real_read_text = Path.read_text

        def mock_read_text(path_obj, *args, **kwargs):
            if path_obj == packet_path:
                return json.dumps(mutated_data)
            return real_read_text(path_obj, *args, **kwargs)

        with patch.object(Path, "read_text", mock_read_text):
            code, receipt = self.run_main()
        self.assertEqual(code, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any("automatic apply prohibited" in err for err in receipt["errors"]))

    def test_adversarial_rejection_of_index_automatic_apply(self) -> None:
        index_path = check_companion_change_packets.INDEX
        original_data = json.loads(index_path.read_text(encoding="utf-8"))
        mutated_data = dict(original_data)
        mutated_data["automatic_apply"] = True

        real_read_text = Path.read_text

        def mock_read_text(path_obj, *args, **kwargs):
            if path_obj == index_path:
                return json.dumps(mutated_data)
            return real_read_text(path_obj, *args, **kwargs)

        with patch.object(Path, "read_text", mock_read_text):
            code, receipt = self.run_main()
        self.assertEqual(code, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any("automatic apply prohibited" in err for err in receipt["errors"]))

    def test_adversarial_rejection_of_invalid_estate_repositories(self) -> None:
        repos_path = check_companion_change_packets.ESTATE_REPOSITORIES
        original_data = json.loads(repos_path.read_text(encoding="utf-8"))
        mutated_data = dict(original_data)
        mutated_data["schema_version"] = "org.searchright.estate-repositories.invalid"

        real_read_text = Path.read_text

        def mock_read_text(path_obj, *args, **kwargs):
            if path_obj == repos_path:
                return json.dumps(mutated_data)
            return real_read_text(path_obj, *args, **kwargs)

        with patch.object(Path, "read_text", mock_read_text):
            code, receipt = self.run_main()
        self.assertEqual(code, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any("estate repositories: invalid schema version" in err for err in receipt["errors"]))

    def test_adversarial_rejection_of_duplicate_estate_repository(self) -> None:
        repos_path = check_companion_change_packets.ESTATE_REPOSITORIES
        original_data = json.loads(repos_path.read_text(encoding="utf-8"))
        mutated_data = json.loads(json.dumps(original_data))
        mutated_data["repositories"].append(dict(mutated_data["repositories"][0]))

        real_read_text = Path.read_text

        def mock_read_text(path_obj, *args, **kwargs):
            if path_obj == repos_path:
                return json.dumps(mutated_data)
            return real_read_text(path_obj, *args, **kwargs)

        with patch.object(Path, "read_text", mock_read_text):
            code, receipt = self.run_main()
        self.assertEqual(code, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(any("duplicate repository" in err for err in receipt["errors"]))

    def test_adversarial_rejection_of_invalid_estate_decisions(self) -> None:
        decisions_path = check_companion_change_packets.ESTATE_REPLACEMENTS
        original_data = json.loads(decisions_path.read_text(encoding="utf-8"))
        mutated_data = dict(original_data)
        mutated_data["schema_version"] = "org.searchright.estate-replacement-decisions.invalid"

        real_read_text = Path.read_text

        def mock_read_text(path_obj, *args, **kwargs):
            if path_obj == decisions_path:
                return json.dumps(mutated_data)
            return real_read_text(path_obj, *args, **kwargs)

        with patch.object(Path, "read_text", mock_read_text):
            code, receipt = self.run_main()
        self.assertEqual(code, 1)
        self.assertEqual(receipt["status"], "failed")
        self.assertTrue(
            any("estate replacement decisions: invalid schema version" in err for err in receipt["errors"])
        )


if __name__ == "__main__":
    unittest.main()
