#!/usr/bin/env python3
"""Render the Conductor roadmap into deterministic GitHub issue bodies."""
from __future__ import annotations
import argparse
import json
import re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
COVERAGE=ROOT/'conductor/roadmap-coverage.json'
OUT=ROOT/'conductor/github/issues'
HIERARCHY=ROOT/'conductor/github/issue-hierarchy.json'
PHASE_RE=re.compile(r'^## Phase (\d+): (.+)$',re.MULTILINE)

def phase_sections(plan:str)->list[tuple[int,str,str]]:
    matches=list(PHASE_RE.finditer(plan)); result=[]
    for i,m in enumerate(matches):
        start=m.end(); end=matches[i+1].start() if i+1<len(matches) else len(plan)
        body=plan[start:end].strip()
        result.append((int(m.group(1)),m.group(2).strip(),body))
    return result

def marker(key:str)->str: return f'<!-- searchright-issue-key: {key} -->'

def epic_body(entries:list[dict])->str:
    lines=[marker('roadmap-epic'),'# Searchright roadmap epic','',
      'This issue is generated from `conductor/roadmap-coverage.json`. Conductor remains canonical; remote status cannot promote repository evidence.','',
      '## Tracks','']
    for e in entries:
        lines.append(f"- [ ] `{e['track_id']}` — {e['title']} (`track-{e['track_id']}`)")
    lines += ['', '## Synchronisation contract','',
      '- Dry-run is the default.','- Apply requires an explicit CLI flag, environment opt-in and GitHub write permission.',
      '- Track issues are native subissues of this epic; phase issues are native subissues of their track.',
      '- Stable markers preserve idempotency and a portable hierarchy fallback.','']
    return '\n'.join(lines)

def track_body(e:dict)->str:
    tid=e['track_id']; lines=[marker(f'track-{tid}'),f"# Track {tid}: {e['title']}",'',e['outcome'],'',
      '## Source of truth','',f"- Spec: `conductor/tracks/{tid}-{e['slug']}/spec.md`",f"- Plan: `conductor/tracks/{tid}-{e['slug']}/plan.md`",f"- Evidence: `conductor/tracks/{tid}-{e['slug']}/evidence.json`",'',
      '## Contract','',f"- Horizon: `{e['horizon']}`",f"- Status: `{e['status']}`",f"- Evidence: `{e['evidence_level']}`",f"- Dependencies: `{', '.join(e.get('dependencies',[])) or 'none'}`",f"- Requirements: `{', '.join(e.get('requirements',[])) or 'none'}`",f"- External approval required: `{str(bool(e.get('external_approval_required'))).lower()}`",'',
      '## Phase subissues','']
    for n,title in ((1,'Source implementation'),(2,'Source-level verification'),(3,'Higher-evidence gates'),(4,'Review and closeout')):
        lines.append(f'- [ ] Phase {n}: {title} (`track-{tid}-phase-{n}`)')
    lines += ['', '## Claim boundary','', e['claim_boundary'],'',
      '> Closing this GitHub issue cannot by itself promote evidence. The Conductor evidence record and applicable runtime/external receipts remain authoritative.','']
    return '\n'.join(lines)

def phase_body(e:dict,n:int,title:str,section:str)->str:
    tid=e['track_id']; key=f'track-{tid}-phase-{n}'
    return '\n'.join([marker(key),f"# Track {tid} / Phase {n}: {title}",'',
      f"Parent track key: `track-{tid}`",f"Conductor plan: `conductor/tracks/{tid}-{e['slug']}/plan.md`",'',
      '## Phase tasks','',section,'',
      '## Evidence rule','',
      'Remote completion is a planning signal only. Evidence is promoted only through the track evidence record and a reproducible receipt at the claimed level.',''])

def build()->tuple[dict,dict[Path,str]]:
    entries=json.loads(COVERAGE.read_text())['tracks']; outputs={}; nodes=[]
    outputs[OUT/'roadmap-epic.md']=epic_body(entries)
    nodes.append({'key':'roadmap-epic','title':'Searchright roadmap','kind':'epic','parent_key':None,'body_path':'conductor/github/issues/roadmap-epic.md','labels':['kind:epic','conductor'],'status':'prepared_not_synced'})
    for e in entries:
        tid=e['track_id']; track_key=f'track-{tid}'; tpath=OUT/f'{track_key}.md'
        outputs[tpath]=track_body(e)
        labels=['kind:track','conductor',f"horizon:{e['horizon']}", 'evidence:external-required' if e.get('external_approval_required') else 'evidence:source-verified']
        nodes.append({'key':track_key,'title':f"Track {tid}: {e['title']}",'kind':'track','parent_key':'roadmap-epic','body_path':tpath.relative_to(ROOT).as_posix(),'labels':labels,'status':'prepared_not_synced'})
        plan=(ROOT/f"conductor/tracks/{tid}-{e['slug']}/plan.md").read_text()
        phases=phase_sections(plan)
        if [x[0] for x in phases] != [1,2,3,4]:
            raise ValueError(f'track {tid} must contain phases 1-4 exactly')
        for n,title,section in phases:
            key=f'{track_key}-phase-{n}'; path=OUT/f'{key}.md'; outputs[path]=phase_body(e,n,title,section)
            nodes.append({'key':key,'title':f'Track {tid} / Phase {n}: {title}','kind':'phase','parent_key':track_key,'body_path':path.relative_to(ROOT).as_posix(),'labels':['kind:phase','conductor',f'track:{tid}'],'status':'prepared_not_synced'})
    hierarchy={'schema_version':'org.searchright.github-issue-hierarchy.v1','repository':'edithatogo/searchright','epic_key':'roadmap-epic','nodes':nodes,'generated_at':'2026-08-06','apply_permitted':False}
    outputs[HIERARCHY]=json.dumps(hierarchy,indent=2)+"\n"
    return hierarchy,outputs

def main()->int:
    ap=argparse.ArgumentParser(); ap.add_argument('--check',action='store_true'); args=ap.parse_args()
    hierarchy,outputs=build(); stale=[]
    for path,content in outputs.items():
        if args.check:
            if not path.is_file() or path.read_text()!=content: stale.append(path.relative_to(ROOT).as_posix())
        else:
            path.parent.mkdir(parents=True,exist_ok=True); path.write_text(content)
    expected={p for p in outputs if p.parent==OUT}
    extras={p for p in OUT.glob('*.md')} - expected
    if args.check and extras: stale += [f'extra:{p.relative_to(ROOT).as_posix()}' for p in sorted(extras)]
    elif not args.check:
        for p in extras: p.unlink()
    status='failed' if stale else 'passed'
    print(json.dumps({'schema_version':'org.searchright.github-issue-render-receipt.v1','status':status,'nodes':len(hierarchy['nodes']),'issue_bodies':len(expected),'stale':stale},indent=2,sort_keys=True))
    return 1 if stale else 0
if __name__=='__main__': raise SystemExit(main())
