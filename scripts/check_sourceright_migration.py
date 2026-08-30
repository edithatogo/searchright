#!/usr/bin/env python3
"""Validate the pinned Sourceright migration packet and parity-case coverage."""
from __future__ import annotations
import json,re,sys
from pathlib import Path
import yaml
ROOT=Path(__file__).resolve().parents[1]
MAP=ROOT/'migration/sourceright/replacement-map.yaml'
CASES=ROOT/'migration/sourceright/parity-cases.json'
COMPANION=ROOT/'migration/companion-repositories/sourceright.json'
CONTRACT=ROOT/'crates/searchright-contracts/src/migration.rs'
# Exact existing catalogue: changes require a deliberate contract/catalogue review.
CASE_DIMENSIONS={
    'disabled-live': {'execution mode','error classification','endpoint and secret redaction','disabled-live negative behaviour'},
    'fixture-identifiers': {'provider identity','identifiers','normalised fields','fixture determinism'},
    'bounded-retry': {'retry and rate behaviour','timeout behaviour','error classification'},
    'cache-write-replay': {'execution mode','replay and cache behaviour','receipt counts'},
    'malformed-payload': {'error classification','malformed and adversarial response handling'},
    'undeclared-host': {'host policy','malformed and adversarial response handling'},
    'secret-redaction': {'endpoint and secret redaction','cache key redaction'},
}

def rust_catalogue(source, name):
    """Read the literal Rust contract surface; unfamiliar syntax fails closed."""
    match=re.search(r'pub const '+re.escape(name)+r':\s*&\[&str\]\s*=\s*&\[(.*?)\];',source,re.S)
    if match is None:
        return None
    body=match.group(1)
    if re.fullmatch(r'\s*(?:"[^"\\]*"\s*,\s*)*',body) is None:
        return None
    values=re.findall(r'"([^"\\]*)"',body)
    if not values or len(values)!=len(set(values)):
        return None
    return set(values)

errors=[]
mapping=yaml.safe_load(MAP.read_text())
cases=json.loads(CASES.read_text())
companion=json.loads(COMPANION.read_text())
contract=CONTRACT.read_text()
for name,expected in (
    ('SOURCERIGHT_PARITY_CASE_IDS',set(CASE_DIMENSIONS)),
    ('SOURCERIGHT_PARITY_DIMENSIONS',set().union(*CASE_DIMENSIONS.values())),
):
    if rust_catalogue(contract,name)!=expected:
        errors.append(f'Rust {name} differs from the exact migration catalogue')
case_rows=cases.get('cases',[])
if not isinstance(case_rows,list):
    errors.append('parity cases must be a list')
    case_rows=[]
case_ids=[]
covered=set()
for case in case_rows:
    if not isinstance(case,dict):
        errors.append('parity case must be an object')
        continue
    case_id=case.get('case_id')
    if not isinstance(case_id,str) or not case_id.strip():
        errors.append('parity case IDs must be nonblank strings')
        continue
    case_ids.append(case_id)
    dimensions=case.get('dimensions')
    if not isinstance(dimensions,list) or any(not isinstance(d,str) or not d.strip() for d in dimensions):
        errors.append(f'{case_id}: parity dimensions must be nonblank strings in a list')
        continue
    covered.update(dimensions)
    if len(dimensions)!=len(set(dimensions)) or set(dimensions)!=CASE_DIMENSIONS.get(case_id):
        errors.append(f'{case_id}: parity dimensions must match exact case coverage without duplicates')
if len(case_ids)!=len(set(case_ids)) or set(case_ids)!=set(CASE_DIMENSIONS):
    errors.append('parity case IDs must match the exact catalogue once each')
if mapping.get('source',{}).get('blob_sha') != cases.get('source_blob_sha'):
    errors.append('pinned source blob differs between replacement map and parity cases')
revisions={
    mapping.get('source',{}).get('revision'),
    cases.get('source_revision'),
    companion.get('revision'),
}
if None in revisions or len(revisions) != 1:
    errors.append('pinned source revision differs across migration artifacts')
seen=set()
allowed={'planned','partial','source_scaffolded_uncompiled','source_implemented_uncompiled','compatibility_harness_source_implemented_uncompiled'}
for i,item in enumerate(mapping.get('mappings',[])):
    status=item.get('implementation_status')
    if status not in allowed: errors.append(f'mapping {i}: unsupported implementation_status {status!r}')
    for symbol in item.get('existing_symbols',[]):
        if symbol in seen: errors.append(f'duplicate existing symbol mapping: {symbol}')
        seen.add(symbol)
required=set(mapping.get('parity_dimensions',[]))
required_coverage={
    'provider identity': {'provider identity'},
    'execution mode': {'execution mode'},
    'identifiers and normalised candidate fields': {'identifiers','normalised fields'},
    'error classification': {'error classification'},
    'timeout, retry and rate behaviour': {'timeout behaviour','retry and rate behaviour'},
    'fixture determinism': {'fixture determinism'},
    'replay and cache behaviour': {'replay and cache behaviour'},
    'endpoint and secret redaction': {'endpoint and secret redaction'},
    'disabled-live negative behaviour': {'disabled-live negative behaviour'},
    'malformed and adversarial response handling': {'malformed and adversarial response handling'},
}
unknown=required-set(required_coverage)
if unknown: errors.append(f'declared parity dimensions lack an explicit coverage rule: {sorted(unknown)}')
absent=set(required_coverage)-required
if absent: errors.append(f'required parity dimensions are absent from the replacement map: {sorted(absent)}')
missing={dimension:sorted(parts-covered) for dimension,parts in required_coverage.items() if dimension in required and not parts <= covered}
if missing: errors.append(f'declared parity dimensions without exact case coverage: {missing}')
if mapping.get('rules',{}).get('remote_change_status') != 'not_performed': errors.append('remote_change_status must remain not_performed until a remote PR is observed')
receipt={'schema_version':'org.searchright.sourceright-migration-check.v1','status':'passed' if not errors else 'failed','source_revision':cases.get('source_revision'),'source_blob_sha':cases.get('source_blob_sha'),'symbols_mapped':len(seen),'cases_checked':len(case_rows),'declared_dimensions':len(required),'errors':errors,'limitations':['This static check does not itself execute a Sourceright checkout, dual-run, compiler gate or remote pull request.']}
print(json.dumps(receipt,indent=2))
sys.exit(1 if errors else 0)
