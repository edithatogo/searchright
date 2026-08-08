#!/usr/bin/env python3
"""Check the generated epic -> track -> phase GitHub issue hierarchy."""
from __future__ import annotations
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
H=ROOT/'conductor/github/issue-hierarchy.json'; COVERAGE=ROOT/'conductor/roadmap-coverage.json'

def main()->int:
 errors=[]; data=json.loads(H.read_text()); tracks=json.loads(COVERAGE.read_text())['tracks']; nodes=data.get('nodes',[]); by={n.get('key'):n for n in nodes}
 expected_track={f"track-{e['track_id']}" for e in tracks}; expected_phase={f"track-{e['track_id']}-phase-{n}" for e in tracks for n in range(1,5)}
 epic=[n for n in nodes if n.get('kind')=='epic']; actual_track={n['key'] for n in nodes if n.get('kind')=='track'}; actual_phase={n['key'] for n in nodes if n.get('kind')=='phase'}
 if len(nodes)!=1+len(tracks)*5: errors.append(f'expected {1+len(tracks)*5} nodes, found {len(nodes)}')
 if len(epic)!=1 or epic[0].get('key')!='roadmap-epic' or epic[0].get('parent_key') is not None: errors.append('invalid roadmap epic')
 if actual_track!=expected_track: errors.append('track issue keys differ from roadmap')
 if actual_phase!=expected_phase: errors.append('phase issue keys differ from four phases per track')
 for key,n in by.items():
  path=ROOT/n.get('body_path','')
  if not path.is_file(): errors.append(f'missing body for {key}'); continue
  body=path.read_text()
  if f'<!-- searchright-issue-key: {key} -->' not in body: errors.append(f'missing stable marker for {key}')
  if n.get('status')!='prepared_not_synced': errors.append(f'{key} overclaims remote status')
  if n.get('kind')=='track' and n.get('parent_key')!='roadmap-epic': errors.append(f'{key} wrong epic parent')
  if n.get('kind')=='phase':
   m=re.fullmatch(r'(track-\d{2})-phase-([1-4])',key)
   if not m or n.get('parent_key')!=m.group(1) or m.group(1) not in by: errors.append(f'{key} wrong track parent')
 if data.get('apply_permitted') is not False: errors.append('local hierarchy must not authorise remote apply')
 # Metadata and plan markers must agree.
 for e in tracks:
  tid=e['track_id']; d=ROOT/f"conductor/tracks/{tid}-{e['slug']}"; meta=json.loads((d/'metadata.json').read_text()); plan=(d/'plan.md').read_text()
  gh=meta.get('github',{})
  if gh.get('track_issue_key')!=f'track-{tid}' or gh.get('phase_issue_keys')!=[f'track-{tid}-phase-{n}' for n in range(1,5)]: errors.append(f'track {tid} metadata issue keys differ')
  for n in range(1,5):
   if f'<!-- github-subissue-key: track-{tid}-phase-{n} -->' not in plan: errors.append(f'track {tid} plan lacks phase {n} marker')
 receipt={'schema_version':'org.searchright.github-issue-hierarchy-receipt.v1','status':'failed' if errors else 'passed','nodes_checked':len(nodes),'track_issues':len(actual_track),'phase_subissues':len(actual_phase),'errors':errors,'limitations':['No remote GitHub repository, issue number or subissue relationship is claimed.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
