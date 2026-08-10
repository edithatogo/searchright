#!/usr/bin/env python3
"""Validate the strategic cross-repository portfolio projection."""
from __future__ import annotations
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PATH=ROOT/'integration/github/portfolio-project.json'
def main()->int:
 d=json.loads(PATH.read_text()); errors=[]
 if d.get('schema_version')!='org.searchright.evidence-infrastructure-portfolio.v1': errors.append('unexpected schema version')
 if d.get('remote_mutation_permitted') is not False: errors.append('canonical portfolio must not authorise remote mutation')
 fields=d.get('fields',[]); names=[x.get('name') for x in fields]
 required={'Repository','Work kind','Implementation state','Evidence level','Compatibility state','Licence state','Release train','Producer','Consumer','Contract family','Next proof','Searchright track'}
 if not required.issubset(names): errors.append(f'missing fields: {sorted(required-set(names))}')
 if len(names)!=len(set(names)): errors.append('portfolio field names must be unique')
 views=d.get('views',[])
 if len(views)<5 or len({x.get('name') for x in views})!=len(views): errors.append('portfolio requires at least five unique views')
 items=d.get('items',[]); keys=[]; repos=set()
 for item in items:
  keys.append(item.get('key')); repos.add(item.get('repository'))
  for key in ('key','repository','kind','title','source_path','producer','consumer','contract_family','implementation_state','evidence_level','compatibility_state','licence_state','release_train','next_proof','linked_track'):
   if item.get(key) in (None,''): errors.append(f"{item.get('key')}: missing {key}")
  source=ROOT/item.get('source_path','')
  if not source.exists(): errors.append(f"{item.get('key')}: missing source path {item.get('source_path')}")
  if item.get('issue_url') is not None: errors.append(f"{item.get('key')}: issue URL must remain null before remote sync")
 if len(keys)!=len(set(keys)): errors.append('portfolio item keys must be unique')
 required_repos={'edithatogo/searchright','edithatogo/sourceright','edithatogo/citeweft','edithatogo/repository-standards','edithatogo/standards_check'}
 if required_repos-repos: errors.append(f'missing core repositories: {sorted(required_repos-repos)}')
 sync=d.get('sync_policy',{})
 if sync.get('delete')!='never' or sync.get('automatic_promotion') is not False: errors.append('portfolio sync must be non-destructive and non-promoting')
 receipt={'schema_version':'org.searchright.portfolio-project-receipt.v1','status':'failed' if errors else 'passed','fields':len(fields),'views':len(views),'items':len(items),'repositories':len(repos),'remote_mutations':0,'errors':errors,'limitations':['Static portfolio projection only; no GitHub Project or cross-repository issue was created.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
