#!/usr/bin/env python3
"""Validate SDK intent and runnable-example source contracts."""
from __future__ import annotations
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main()->int:
 errors=[]; data=json.loads((ROOT/'sdk/manifest.json').read_text())
 for path in data.get('source_contracts',[]):
  if not (ROOT/path).is_file(): errors.append(f'missing SDK source contract {path}')
 langs=[x.get('language') for x in data.get('targets',[]) if isinstance(x,dict)]
 if set(langs)!={'rust','python','typescript'}: errors.append('SDK target set differs')
 if data.get('automatic_publication') is not False: errors.append('SDK publication must require approval')
 if 'must_not_reimplement' not in data.get('generation_policy',''): errors.append('SDKs must remain thin/generated')
 for path in ['examples/quickstart/README.md','docs/sdk-and-adoption.md']:
  if not (ROOT/path).is_file(): errors.append(f'missing {path}')
 receipt={'schema_version':'org.searchright.sdk-examples-receipt.v1','status':'failed' if errors else 'passed','targets':len(langs),'errors':errors,'limitations':['Static SDK intent only; Python/TypeScript packages were not generated, compiled or published.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
