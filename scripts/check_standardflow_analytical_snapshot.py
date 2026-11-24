"""Validate a pinned StandardFlow discovery snapshot locally; never admit a pack."""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import re

HEX40 = re.compile(r'[0-9a-f]{40}')
HEX64 = re.compile(r'[0-9a-f]{64}')
IDENTITY = re.compile(r'equator:[a-z0-9][a-z0-9._-]*')
SCHEMAS = {
    'catalogue_identity': {**dict.fromkeys(('key', 'source_revision', 'source_blob_sha', 'page_url', 'metadata_status', 'title', 'acronym',
        'declared_doi', 'declared_pmid', 'study_types_json', 'clinical_areas_json', 'report_sections_json', 'lookup_status', 'resolved_doi',
        'resolved_pmid', 'pmcid', 'lookup_origin', 'rights_status', 'disposition'), 'str'), 'identity_conflict': 'bool'},
    'acquisitions': {**dict.fromkeys(('key', 'doi', 'pmcid', 'source_sha256', 'status', 'disposition', 'rights_status'), 'str'),
        'expected_rows': 'int', 'matched_rows': 'int'},
    'acquisition_items': dict.fromkeys(('key', 'edition', 'table_id', 'item_id', 'source_anchor', 'fingerprint', 'disposition'), 'str'),
    'review_findings': dict.fromkeys(('key', 'code', 'record'), 'str'),
}


def matches(pattern, value):
    return isinstance(value, str) and pattern.fullmatch(value) is not None


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def validate_manifest(raw, expected_sha, producer, source, allow_replay=False):
    if digest(raw) != expected_sha or not matches(HEX40, producer) or not matches(HEX40, source):
        raise ValueError('manifest digest or revision pin mismatch')
    m = json.loads(raw)
    if (m.get('schema_version') != 'dev.standardflow.equator-analytical-snapshot.v1' or
            m.get('producer_repository') != 'edithatogo/standards_check' or
            m.get('producer_revision') != producer or m.get('source_revision') != source or
            m.get('canonical_packs_promoted') != 0 or
            m.get('origin') not in ({'scheduled_or_live', 'historical_replay'} if allow_replay else {'scheduled_or_live'})):
        raise ValueError('snapshot boundary or revision mismatch')
    hashes = m.get('input_sha256s', {})
    if set(hashes) != {'catalogue', 'survey', 'retrospective'} or not all(matches(HEX64, h) for h in hashes.values()):
        raise ValueError('missing input evidence')
    if set(m.get('tables', {})) != set(SCHEMAS):
        raise ValueError('unexpected snapshot tables')
    for name, table in m['tables'].items():
        if (table.get('schema') != SCHEMAS[name] or not matches(HEX64, table.get('sha256')) or
                type(table.get('rows')) is not int or not 0 <= table['rows'] <= 100_000):
            raise ValueError(f'invalid schema, digest or row budget: {name}')
    return m


def validate_rows(tables, source):
    if set(tables) != set(SCHEMAS):
        raise ValueError('unexpected tables')
    for name, rows in tables.items():
        keys = [row.get('key') for row in rows]
        if any(not isinstance(key, str) or not key for key in keys) or len(set(keys)) != len(keys):
            raise ValueError(f'duplicate or missing key: {name}')
        if any(set(row) != set(SCHEMAS[name]) for row in rows):
            raise ValueError(f'unexpected columns: {name}')
    for row in tables['catalogue_identity']:
        key = row['key']
        if (not matches(IDENTITY, key) or row['source_revision'] != source or
                not matches(HEX40, row['source_blob_sha']) or
                row['page_url'] != 'https://equator-network.org/reporting-guidelines/' + key.split(':', 1)[1] + '/' or
                row['rights_status'] != 'unassessed' or type(row['identity_conflict']) is not bool or
                row['disposition'] != ('held_identity_conflict' if row['identity_conflict'] else 'discovery_only')):
            raise ValueError('invalid or promoted catalogue identity')
    editions = {row['key'] for row in tables['acquisitions']}
    if any(row['disposition'] != 'held_review' or row['rights_status'] != 'unassessed' for row in tables['acquisitions']):
        raise ValueError('promoted acquisition')
    for row in tables['acquisition_items']:
        if row['edition'] not in editions or row['disposition'] != 'held_review' or not matches(HEX64, row['fingerprint']):
            raise ValueError('unbound, invalid or promoted acquisition item')
    return {name: len(rows) for name, rows in tables.items()}


def read_bounded(path, maximum):
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'not a regular snapshot file: {path.name}')
    with path.open('rb') as stream:
        raw = stream.read(maximum + 1)
    if len(raw) > maximum:
        raise ValueError(f'snapshot file exceeds byte budget: {path.name}')
    return raw


def check_snapshot(root, expected_sha, producer, source, allow_replay=False):
    import polars as pl  # optional, experimental integration dependency
    m = validate_manifest(read_bounded(root / 'manifest.json', 100_000), expected_sha, producer, source, allow_replay)
    types = {'str': pl.String, 'bool': pl.Boolean, 'int': pl.Int64}
    tables = {}
    for name, entry in m['tables'].items():
        raw = read_bounded(root / (name + '.parquet'), 30_000_000)
        if digest(raw) != entry['sha256']:
            raise ValueError(f'Parquet digest mismatch: {name}')
        frame = pl.read_parquet(io.BytesIO(raw))
        if dict(frame.schema) != {k: types[v] for k, v in SCHEMAS[name].items()} or frame.height != entry['rows']:
            raise ValueError(f'Parquet schema or row count mismatch: {name}')
        tables[name] = frame.to_dicts()
    counts = validate_rows(tables, source)
    return {'schema_version': 'org.searchright.standardflow-discovery-receipt.v1',
            'manifest_sha256': expected_sha, 'producer_revision': producer, 'source_revision': source,
            'origin': m['origin'], 'rows': counts, 'identity_conflicts': sum(row['identity_conflict'] for row in tables['catalogue_identity']),
            'canonical_packs_promoted': 0, 'review_state_mutations': 0, 'claim_boundary': 'discovery evidence only'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('snapshot', type=Path)
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--producer-revision', required=True)
    parser.add_argument('--source-revision', required=True)
    parser.add_argument('--allow-historical-replay', action='store_true')
    args = parser.parse_args()
    receipt = check_snapshot(args.snapshot, args.manifest_sha256, args.producer_revision, args.source_revision, args.allow_historical_replay)
    print(json.dumps(receipt, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
