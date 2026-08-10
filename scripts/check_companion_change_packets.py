#!/usr/bin/env python3
"""Validate prepared, non-mutating companion repository change packets."""
from __future__ import annotations
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
INDEX=ROOT/'migration/companion-repositories/index.json'
HEX40=re.compile(r'^[a-f0-9]{40}$')
def main()->int:
 errors=[]; index=json.loads(INDEX.read_text()); checked=0; changes=0; repos=set()
 for item in index.get('packets',[]):
  path=ROOT/item['path']
  if not path.is_file(): errors.append(f"missing packet {item['path']}"); continue
  d=json.loads(path.read_text()); checked+=1
  repo=d.get('repository'); rev=d.get('revision')
  if repo in repos: errors.append(f'duplicate companion repository {repo}')
  repos.add(repo)
  if d.get('schema_version')!='org.searchright.companion-change-packet.v1': errors.append(f'{repo}: schema version')
  if not isinstance(rev,str) or not HEX40.fullmatch(rev): errors.append(f'{repo}: exact revision required')
  if d.get('remote_mutation_permitted') is not False: errors.append(f'{repo}: remote mutation must be false')
  if not d.get('claim_boundary') or not d.get('completion_condition'): errors.append(f'{repo}: claim/completion boundary missing')
  entries=d.get('changes',[]); changes+=len(entries); ids=[]
  for change in entries:
   ids.append(change.get('id'))
   if change.get('automatic_apply') is not False: errors.append(f"{repo}:{change.get('id')}: automatic apply prohibited")
   for key in ('action','owner','target_paths','required_evidence'):
    if not change.get(key): errors.append(f"{repo}:{change.get('id')}: missing {key}")
  if len(ids)!=len(set(ids)): errors.append(f'{repo}: duplicate change IDs')
 expected={'edithatogo/citeweft','edithatogo/sourceright','edithatogo/repository-standards','edithatogo/standards_check','edithatogo/academic-research-skills','edithatogo/UOGTO','edithatogo/voiage','edithatogo/scholarly-publishing-agents','edithatogo/api-standards','edithatogo/PRISMA.jl','edithatogo/synergy-dataset'}
 if expected-repos: errors.append(f'missing required companion packets: {sorted(expected-repos)}')
 receipt={'schema_version':'org.searchright.companion-change-packet-receipt.v1','status':'failed' if errors else 'passed','packets_checked':checked,'changes_checked':changes,'remote_mutations':0,'errors':errors,'limitations':['No companion repository was modified or tested by this check.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
