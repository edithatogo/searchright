#!/usr/bin/env python3
"""Check the source-preserving native-query conformance corpus."""
from __future__ import annotations
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
INDEX=ROOT/'contracts/query-corpus/index.json'
DIALECTS={'pub_med','ovid_medline','embase','cinahl_ebsco','psyc_info_ovid','scopus','web_of_science'}

def main()->int:
    errors=[]; data=json.loads(INDEX.read_text()); seen=set(); checked=0
    for fixture in data.get('fixtures',[]):
        fid=fixture.get('id'); dialect=fixture.get('dialect'); path=ROOT/str(fixture.get('path',''))
        if not fid or fid in seen: errors.append(f'duplicate or empty fixture id {fid!r}')
        seen.add(fid)
        if dialect not in DIALECTS: errors.append(f'{fid} has unsupported dialect {dialect!r}')
        if not path.is_file(): errors.append(f'{fid} missing {path.relative_to(ROOT)}'); continue
        text=path.read_text(encoding='utf-8')
        if '\x00' in text: errors.append(f'{fid} contains NUL')
        lines=text.splitlines()
        if len(lines)<fixture.get('minimum_lines',1): errors.append(f'{fid} has fewer lines than declared')
        if any(line.rstrip()!=line for line in lines): errors.append(f'{fid} has trailing whitespace')
        if not text.endswith('\n'): errors.append(f'{fid} must end with a newline for byte-stable line spans')
        checked+=1
    if set(DIALECTS)!={item.get('dialect') for item in data.get('fixtures',[])}:
        errors.append('corpus must cover each declared MVP native dialect exactly at least once')
    receipt={'schema_version':'org.searchright.native-query-corpus-receipt.v1','status':'failed' if errors else 'passed','fixtures_checked':checked,'dialects':sorted(DIALECTS),'errors':errors,'limitations':['Lexical source corpus only; semantic equivalence requires independently reviewed golden parses and translation-loss assessments.']}
    print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
