#!/usr/bin/env python3
"""Check the source-preserving native-query conformance corpus."""
from __future__ import annotations
import json
import hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
INDEX=ROOT/'contracts/query-corpus/index.json'
LOSS_MATRIX=ROOT/'contracts/query-corpus/loss-matrix.json'
DIALECTS={'pub_med','ovid_medline','embase','cinahl_ebsco','psyc_info_ovid','scopus','web_of_science'}

def main()->int:
    errors=[]; data=json.loads(INDEX.read_text()); seen=set(); checked=0
    provenance=data.get('provenance',{})
    for field in ('origin','rights_basis','redistribution_decision','external_methodological_review'):
        if not isinstance(provenance.get(field),str) or not provenance[field].strip():
            errors.append(f'corpus provenance requires non-empty {field}')
    for fixture in data.get('fixtures',[]):
        fid=fixture.get('id'); dialect=fixture.get('dialect'); path=ROOT/str(fixture.get('path',''))
        if not fid or fid in seen: errors.append(f'duplicate or empty fixture id {fid!r}')
        seen.add(fid)
        if dialect not in DIALECTS: errors.append(f'{fid} has unsupported dialect {dialect!r}')
        if not path.is_file(): errors.append(f'{fid} missing {path.relative_to(ROOT)}'); continue
        content=path.read_bytes(); text=content.decode('utf-8')
        declared=fixture.get('sha256')
        actual=hashlib.sha256(content).hexdigest()
        if declared != actual: errors.append(f'{fid} sha256 mismatch: expected {declared!r}, observed {actual}')
        if '\x00' in text: errors.append(f'{fid} contains NUL')
        lines=text.splitlines()
        if len(lines)<fixture.get('minimum_lines',1): errors.append(f'{fid} has fewer lines than declared')
        if any(line.rstrip()!=line for line in lines): errors.append(f'{fid} has trailing whitespace')
        if not text.endswith('\n'): errors.append(f'{fid} must end with a newline for byte-stable line spans')
        checked+=1
    if set(DIALECTS)!={item.get('dialect') for item in data.get('fixtures',[])}:
        errors.append('corpus must cover each declared MVP native dialect exactly at least once')
    matrix=json.loads(LOSS_MATRIX.read_text(encoding='utf-8'))
    verification=matrix.get('verification',{})
    for field in ('semantic_parser','compile_parse_compile_property','exact_source_preservation','scope'):
        if not isinstance(verification.get(field),str) or not verification[field].strip():
            errors.append(f'loss matrix verification requires non-empty {field}')
    rows=matrix.get('dialects',[])
    if len(rows) != len(DIALECTS) or {row.get('dialect') for row in rows} != DIALECTS:
        errors.append('loss matrix must cover each declared MVP native dialect exactly once')
    fixture_dialects={item.get('id'):item.get('dialect') for item in data.get('fixtures',[])}
    for row in rows:
        dialect=row.get('dialect')
        if row.get('fixture_id') not in fixture_dialects:
            errors.append(f'{dialect} loss matrix refers to an unknown fixture')
        elif fixture_dialects[row.get('fixture_id')] != dialect:
            errors.append(f'{dialect} loss matrix fixture belongs to a different dialect')
        if not row.get('supported_constructs'):
            errors.append(f'{dialect} loss matrix requires supported_constructs')
        if not row.get('known_losses'):
            errors.append(f'{dialect} loss matrix must fail closed with known_losses')
    receipt={'schema_version':'org.searchright.native-query-corpus-receipt.v1','status':'failed' if errors else 'passed','fixtures_checked':checked,'dialects':sorted(DIALECTS),'rights_basis':provenance.get('rights_basis'),'external_methodological_review':provenance.get('external_methodological_review'),'errors':errors,'limitations':['Project-authored bounded syntax fixtures only; an isolated methodology, safety and adversarial agent panel must review exact corpus and loss-matrix digests, preserve first-pass findings and dissent, and submit findings to the accountable owner for decision. Panel findings and owner decisions alone do not establish provider currency, empirical retrieval equivalence or topic-specific methodological adequacy.']}
    print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
