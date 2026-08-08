#!/usr/bin/env python3
"""Check CiteWeft's pinned, one-way and non-canonical integration boundary."""
from __future__ import annotations
import json, re, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
errors=[]
manifest=json.loads((ROOT/'integration/citeweft-compatibility.json').read_text())
root=tomllib.loads((ROOT/'Cargo.toml').read_text())
dep=root['workspace']['dependencies'].get('citeweft')
if not isinstance(dep,dict) or dep.get('rev')!=manifest['upstream_commit']: errors.append('workspace CiteWeft revision differs from compatibility manifest')
members=root['workspace']['members']
if 'crates/searchright-citeweft' not in members: errors.append('searchright-citeweft is not a workspace member')
for member in members:
    cargo=tomllib.loads((ROOT/member/'Cargo.toml').read_text())
    deps={}
    for section in ('dependencies','dev-dependencies','build-dependencies'): deps.update(cargo.get(section,{}))
    if 'citeweft' in deps and member!='crates/searchright-citeweft': errors.append(f'{member} directly depends on CiteWeft')
text=(ROOT/'crates/searchright-contracts/src/document.rs').read_text()
for invariant in ('canonical_write_permitted','retained_full_text','DocumentEvidence'):
    if invariant not in text: errors.append(f'missing document evidence invariant {invariant}')
adapter=(ROOT/'crates/searchright-citeweft/src/lib.rs').read_text()
for symbol in ('from_scholarly_document','from_reference_model_report'):
    if not re.search(rf'pub fn {symbol}\b',adapter): errors.append(f'missing adapter {symbol}')
if 'canonical_write_permitted: false' not in adapter: errors.append('adapter does not explicitly deny canonical writes')
receipt={'schema_version':'org.searchright.citeweft-integration-receipt.v1','status':'failed' if errors else 'passed','upstream_commit':manifest['upstream_commit'],'checks':7,'errors':errors,'limitations':['Source-level compatibility only; the pinned git dependency and adapter tests were not compiled in this environment.']}
print(json.dumps(receipt,indent=2,sort_keys=True))
raise SystemExit(1 if errors else 0)
