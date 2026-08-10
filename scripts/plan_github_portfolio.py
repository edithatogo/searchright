#!/usr/bin/env python3
"""Render a non-mutating GitHub Project/issue plan for the evidence portfolio."""
from __future__ import annotations
import argparse,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main()->int:
 ap=argparse.ArgumentParser(); ap.add_argument('--output',type=Path); args=ap.parse_args()
 d=json.loads((ROOT/'integration/github/portfolio-project.json').read_text())
 plan={
  'schema_version':'org.searchright.github-portfolio-plan.v1','status':'prepared_not_applied','owner':d['owner'],'project_title':d['title'],
  'field_actions':[{'action':'ensure_field',**field} for field in d['fields']],
  'view_actions':[{'action':'ensure_view',**view} for view in d['views']],
  'item_actions':[{'action':'ensure_draft_or_linked_issue','key':item['key'],'title':item['title'],'repository':item['repository'],'source_path':item['source_path'],'fields':{k:item[k] for k in ('repository','kind','implementation_state','evidence_level','compatibility_state','licence_state','release_train','producer','consumer','contract_family','next_proof','linked_track')}} for item in d['items']],
  'destructive_actions':0,'automatic_promotions':0,'claim_boundary':d['claim_boundary']
 }
 text=json.dumps(plan,indent=2,sort_keys=True)+'\n'
 if args.output: args.output.write_text(text)
 print(text,end=''); return 0
if __name__=='__main__': raise SystemExit(main())
