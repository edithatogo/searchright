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
 gates=data.get('required_gates')
 if not isinstance(gates,list) or not gates or any(not isinstance(gate,str) or not gate.strip() for gate in gates): errors.append('rehearsal requires named checks for the shipped profile')
 elif len(gates)!=len(set(gates)): errors.append('rehearsal checks must be unique')
 profiles=data.get('pilot_profiles')
 supported={'local_researcher','institution_self_hosted','remote_single_tenant'}
 if not isinstance(profiles,list) or not profiles or any(not isinstance(profile,str) or profile not in supported for profile in profiles): errors.append('pilot profiles must name supported deployment profiles')
 elif len(profiles)!=len(set(profiles)): errors.append('pilot profiles must be unique')
 if data.get('rollback_required') is not True: errors.append('rollback rehearsal is mandatory')
 for path in ['docs/releases/release-candidate.md','docs/pilots/pilot-protocol.md']:
  if not (ROOT/path).is_file(): errors.append(f'missing {path}')
 receipt={'schema_version':'org.searchright.release-rehearsal-receipt.v1','status':'failed' if errors else 'passed','gates':len(gates) if isinstance(gates,list) else 0,'pilot_profiles':len(profiles) if isinstance(profiles,list) else 0,'errors':errors,'limitations':['Static rehearsal contract only; no release candidate, pilot or rollback was executed.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
