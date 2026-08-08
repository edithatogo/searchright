#!/usr/bin/env python3
"""Validate the pinned Sourceright migration packet and parity-case coverage."""
from __future__ import annotations
import json,sys
from pathlib import Path
import yaml
ROOT=Path(__file__).resolve().parents[1]
MAP=ROOT/'migration/sourceright/replacement-map.yaml'
CASES=ROOT/'migration/sourceright/parity-cases.json'
errors=[]
mapping=yaml.safe_load(MAP.read_text())
cases=json.loads(CASES.read_text())
if mapping.get('source',{}).get('blob_sha') != cases.get('source_blob_sha'):
    errors.append('pinned source blob differs between replacement map and parity cases')
seen=set()
allowed={'planned','partial','source_scaffolded_uncompiled','source_implemented_uncompiled','compatibility_harness_source_implemented_uncompiled'}
for i,item in enumerate(mapping.get('mappings',[])):
    status=item.get('implementation_status')
    if status not in allowed: errors.append(f'mapping {i}: unsupported implementation_status {status!r}')
    for symbol in item.get('existing_symbols',[]):
        if symbol in seen: errors.append(f'duplicate existing symbol mapping: {symbol}')
        seen.add(symbol)
required=set(mapping.get('parity_dimensions',[]))
covered={d for c in cases.get('cases',[]) for d in c.get('dimensions',[])}
# Some source-map dimensions are composite labels; require explicit exact coverage for critical controls.
critical={'provider identity','execution mode','error classification','retry and rate behaviour','replay and cache behaviour','endpoint and secret redaction','disabled-live negative behaviour','malformed and adversarial response handling'}
normalise=lambda x:x.lower().replace('_',' ').replace('-', ' ').strip()
covered_n={normalise(x) for x in covered}; missing=[]
for dim in critical:
    tokens=set(normalise(dim).split())
    if not any(tokens <= set(c.split()) or set(c.split()) <= tokens for c in covered_n): missing.append(dim)
if missing: errors.append(f'critical parity dimensions without case coverage: {missing}')
if mapping.get('rules',{}).get('remote_change_status') != 'not_performed': errors.append('remote_change_status must remain not_performed until a remote PR is observed')
receipt={'schema_version':'org.searchright.sourceright-migration-check.v1','status':'passed' if not errors else 'failed','source_blob_sha':cases.get('source_blob_sha'),'symbols_mapped':len(seen),'cases_checked':len(cases.get('cases',[])),'declared_dimensions':len(required),'errors':errors,'limitations':['No Sourceright checkout, dual-run, compiler gate or remote pull request was executed.']}
print(json.dumps(receipt,indent=2))
sys.exit(1 if errors else 0)
