#!/usr/bin/env python3
"""Validate the release-candidate and pilot rehearsal contract."""
from __future__ import annotations
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main()->int:
 errors=[]; data=json.loads((ROOT/'release/rehearsal.json').read_text())
 if data.get('status')!='prepared_not_executed': errors.append('source rehearsal must not claim execution')
 if data.get('automatic_release') is not False or data.get('automatic_registry_submission') is not False: errors.append('release and registry actions require approval')
 if len(data.get('required_gates',[]))<10: errors.append('rehearsal gate set is incomplete')
 if set(data.get('pilot_profiles',[]))!={'local_researcher','institution_self_hosted','remote_single_tenant'}: errors.append('pilot profiles differ')
 if data.get('rollback_required') is not True: errors.append('rollback rehearsal is mandatory')
 for path in ['docs/releases/release-candidate.md','docs/pilots/pilot-protocol.md']:
  if not (ROOT/path).is_file(): errors.append(f'missing {path}')
 receipt={'schema_version':'org.searchright.release-rehearsal-receipt.v1','status':'failed' if errors else 'passed','gates':len(data.get('required_gates',[])),'pilot_profiles':len(data.get('pilot_profiles',[])),'errors':errors,'limitations':['Static rehearsal contract only; no release candidate, pilot or rollback was executed.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
