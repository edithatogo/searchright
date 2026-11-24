import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from scripts.check_standardflow_analytical_snapshot import SCHEMAS, validate_manifest, validate_rows

PRODUCER = 'a' * 40
SOURCE = 'b' * 40


def manifest():
    return {'schema_version': 'dev.standardflow.equator-analytical-snapshot.v1',
            'producer_repository': 'edithatogo/standards_check', 'producer_revision': PRODUCER,
            'source_revision': SOURCE, 'origin': 'historical_replay', 'canonical_packs_promoted': 0,
            'claim_boundary': 'discovery only', 'polars_version': '1.35.2',
            'input_sha256s': dict.fromkeys(('catalogue', 'survey', 'retrospective'), 'c' * 64),
            'tables': {name: {'schema': schema, 'rows': 0, 'sha256': 'd' * 64} for name, schema in SCHEMAS.items()}}


def rows():
    out = {name: [] for name in SCHEMAS}
    out['catalogue_identity'] = [dict.fromkeys(SCHEMAS['catalogue_identity'], None) | {
        'key': 'equator:example', 'source_revision': SOURCE, 'source_blob_sha': 'c' * 40,
        'page_url': 'https://equator-network.org/reporting-guidelines/example/',
        'rights_status': 'unassessed', 'disposition': 'discovery_only', 'identity_conflict': False}]
    return out


class StandardFlowSnapshotTests(unittest.TestCase):
    def test_pinned_manifest_and_opt_in_replay(self):
        raw = json.dumps(manifest()).encode()
        digest = hashlib.sha256(raw).hexdigest()
        self.assertEqual(validate_manifest(raw, digest, PRODUCER, SOURCE, True)['origin'], 'historical_replay')
        for args in [(digest, PRODUCER, SOURCE, False), ('0' * 64, PRODUCER, SOURCE, True), (digest, '0' * 40, SOURCE, True), (digest, PRODUCER, '0' * 40, True)]:
            with self.subTest(args=args), self.assertRaises(ValueError):
                validate_manifest(raw, *args)

    def test_manifest_rejects_promotions_and_schema_drift(self):
        for change in [{'canonical_packs_promoted': 1}, {'tables': {}}, {'producer_repository': 'other/repo'}]:
            m = manifest() | change
            raw = json.dumps(m).encode()
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_manifest(raw, hashlib.sha256(raw).hexdigest(), PRODUCER, SOURCE, True)

    def test_discovery_never_promotes(self):
        self.assertEqual(validate_rows(rows(), SOURCE)['catalogue_identity'], 1)
        for change in [{'rights_status': 'cleared'}, {'disposition': 'admitted'}, {'identity_conflict': True}, {'source_revision': '0' * 40}, {'page_url': 'https://example.org/'}]:
            data = rows()
            data['catalogue_identity'][0].update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_rows(data, SOURCE)

    def test_duplicate_and_unbound_item_fail(self):
        data = rows()
        data['catalogue_identity'] *= 2
        with self.assertRaises(ValueError):
            validate_rows(data, SOURCE)
        data = rows()
        data['acquisition_items'] = [dict.fromkeys(SCHEMAS['acquisition_items'], None) | {'key': 'item', 'edition': 'missing', 'disposition': 'held_review', 'fingerprint': 'c' * 64}]
        with self.assertRaises(ValueError):
            validate_rows(data, SOURCE)

if __name__ == '__main__':
    unittest.main()
