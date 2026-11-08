"""Regression checks for Conductor evidence inventory and canonical projections."""

import json
from pathlib import Path
import re
import unittest
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]


def load(path):
    return json.loads((ROOT / path).read_text())


class ConductorReconciliationTests(unittest.TestCase):
    def test_public_mapping_inventory_matches_assertion_ledgers(self):
        tracks = load("conductor/roadmap-coverage.json")["tracks"]
        assertions = []
        for track in tracks:
            path = f"conductor/tracks/{track['track_id']}-{track['slug']}/traceability.json"
            assertions.extend(load(path)["assertions"])
        individually_mapped = sum(
            row.get("mapping_confidence") != "track_level_only" for row in assertions
        )
        claim = next(
            text for text in load("context/claim-boundaries.json")["allowed_claims"]
            if text.startswith("The roadmap contains ")
        )
        match = re.search(r"contains (\d+) acceptance assertions, of which (\d+)", claim)
        self.assertIsNotNone(match)
        self.assertEqual(tuple(map(int, match.groups())), (len(assertions), individually_mapped))

    def test_evidence_links_are_local_files_or_well_formed_https_urls(self):
        for track in load("conductor/roadmap-coverage.json")["tracks"]:
            for field in ("runtime_evidence", "remote_github_evidence", "external_evidence"):
                for path in track.get(field, []):
                    with self.subTest(track=track["track_id"], field=field, path=path):
                        url = urlsplit(path)
                        if url.scheme:
                            self.assertEqual(url.scheme, "https")
                            self.assertTrue(url.hostname)
                            self.assertIsNone(url.username)
                            self.assertIsNone(url.password)
                            continue  # Syntax only; no remote availability or approval claim.
                        self.assertFalse(Path(path).is_absolute())
                        self.assertNotIn("..", Path(path).parts)
                        self.assertTrue((ROOT / path).is_file())

    def test_generated_track_evidence_preserves_canonical_state_and_links(self):
        for track in load("conductor/roadmap-coverage.json")["tracks"]:
            path = f"conductor/tracks/{track['track_id']}-{track['slug']}/evidence.json"
            evidence = load(path)
            for field in (
                "status", "implementation_state", "evidence_level", "blockers",
                "runtime_evidence", "remote_github_evidence", "external_evidence",
            ):
                with self.subTest(track=track["track_id"], field=field):
                    default = [] if field.endswith("evidence") else None
                    self.assertEqual(evidence.get(field, default), track.get(field, default))


if __name__ == "__main__":
    unittest.main()
