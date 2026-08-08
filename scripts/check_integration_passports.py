#!/usr/bin/env python3
"""Validate pinned, default-deny integration passports and lock parity."""
from __future__ import annotations
import json
import re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
INDEX=ROOT/'integration/passports/index.json'
LOCKS=ROOT/'integration/locks.json'
HEX40=re.compile(r'^[a-f0-9]{40}$')

def main()->int:
    errors=[]
    index=json.loads(INDEX.read_text())
    locks=json.loads(LOCKS.read_text())
    entries=index.get('passports',[])
    lock_by={x['repository']:x for x in locks.get('repositories',[])}
    ids=set(); repos=set(); checked=0
    for item in entries:
        path=ROOT/item['path']
        if not path.is_file(): errors.append(f"missing passport {item['path']}"); continue
        passport=json.loads(path.read_text())
        checked+=1
        iid=passport.get('integration_id'); repo=passport.get('repository'); rev=passport.get('revision')
        if iid in ids: errors.append(f'duplicate integration_id {iid}')
        if repo in repos: errors.append(f'duplicate active repository {repo}')
        ids.add(iid); repos.add(repo)
        if item != {'integration_id':iid,'repository':repo,'revision':rev,'path':item['path']}:
            errors.append(f'index mismatch for {item["path"]}')
        if not isinstance(rev,str) or not HEX40.fullmatch(rev): errors.append(f'invalid revision for {iid}')
        for key in ('default_network','default_external_writes','default_telemetry','automatic_revision_updates'):
            if passport.get(key) is not False: errors.append(f'{iid} must set {key}=false')
        if not passport.get('verification'): errors.append(f'{iid} has no verification gates')
        if not passport.get('rollback'): errors.append(f'{iid} has no rollback')
        if not passport.get('claim_boundary'): errors.append(f'{iid} has no claim boundary')
        lock=lock_by.get(repo)
        if not lock or lock.get('revision')!=rev or lock.get('passport')!=item['path']:
            errors.append(f'lock mismatch for {repo}')
    if set(lock_by)!=repos: errors.append('integration locks and active passport repositories differ')
    if locks.get('automatic_updates') is not False: errors.append('integration locks must deny automatic updates')
    receipt={'schema_version':'org.searchright.integration-passport-receipt.v1','status':'failed' if errors else 'passed','passports_checked':checked,'candidate_integrations':len(index.get('candidate_integrations',[])),'errors':errors,'limitations':['Source-level validation only; downstream compilation and consumer-driven tests remain separate evidence.']}
    print(json.dumps(receipt,indent=2,sort_keys=True))
    return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
