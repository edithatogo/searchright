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
 bindings=data.get('contract_bindings',{})
 if bindings.get('status')!='generated_compiler_checked': errors.append('contract bindings are not compiler checked')
 if bindings.get('domain_logic') is not False: errors.append('contract bindings must contain no domain logic')
 for path in [bindings.get('generator'),bindings.get('check'),bindings.get('manifest')]:
  if not isinstance(path,str) or not (ROOT/path).is_file(): errors.append(f'missing contract binding path {path!r}')
 for path in ['examples/quickstart/README.md','docs/sdk-and-adoption.md']:
  if not (ROOT/path).is_file(): errors.append(f'missing {path}')
 receipt={'schema_version':'org.searchright.sdk-examples-receipt.v1','status':'failed' if errors else 'passed','targets':len(langs),'contract_bindings':bindings.get('status'),'errors':errors,'limitations':['Contract-only bindings are generated and compiler-checked; Python/TypeScript clients, package installation, publication and downstream adoption remain separate Track 35 evidence.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
