#!/usr/bin/env python3
"""Idempotently sync generated issue bodies and native subissue relations.

Dry-run is unconditional unless both --apply and SEARCHRIGHT_GITHUB_APPLY=1 are
present. The script never creates a repository and never changes Conductor files.
"""
from __future__ import annotations
import argparse,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
HIERARCHY=ROOT/'conductor/github/issue-hierarchy.json'
LABELS=ROOT/'conductor/github/labels.json'

def run(args:list[str], *, capture=True, allow_failure=False)->subprocess.CompletedProcess[str]:
 p=subprocess.run(args,cwd=ROOT,text=True,capture_output=capture,check=False)
 if p.returncode and not allow_failure: raise RuntimeError(f"command failed ({p.returncode}): {' '.join(args)}\n{p.stderr}")
 return p

def existing_issues(repo:str)->dict[str,dict]:
 p=run(['gh','api','--paginate','--slurp',f'repos/{repo}/issues?state=all&per_page=100'])
 payload=json.loads(p.stdout or '[]')
 if payload and isinstance(payload[0],list): payload=[item for page in payload for item in page]
 result={}
 for issue in payload:
  if issue.get('pull_request'): continue
  body=issue.get('body') or ''
  for line in body.splitlines():
   if line.startswith('<!-- searchright-issue-key: ') and line.endswith(' -->'):
    result[line.removeprefix('<!-- searchright-issue-key: ').removesuffix(' -->')]=issue; break
 return result

def main()->int:
 ap=argparse.ArgumentParser(); ap.add_argument('--repo',default='edithatogo/searchright'); ap.add_argument('--apply',action='store_true'); ap.add_argument('--receipt-path',type=Path); args=ap.parse_args()
 data=json.loads(HIERARCHY.read_text()); nodes=data['nodes']; apply=args.apply and os.environ.get('SEARCHRIGHT_GITHUB_APPLY')=='1'
 plan={'schema_version':'org.searchright.github-issue-sync-plan.v1','repository':args.repo,'mode':'apply' if apply else 'dry_run','issues':len(nodes),'relationships':len(nodes)-1,'operations':[]}
 if not apply:
  for n in nodes: plan['operations'].append({'action':'upsert_issue','key':n['key'],'title':n['title'],'parent_key':n['parent_key']})
  print(json.dumps(plan,indent=2,sort_keys=True)); return 0
 if args.repo!=data['repository']: raise SystemExit('apply repository must match the generated hierarchy')
 if run(['git','status','--porcelain']).stdout.strip(): raise SystemExit('apply requires a clean Git working tree')
 run(['gh','repo','view',args.repo,'--json','nameWithOwner'])
 for label in json.loads(LABELS.read_text())['labels']:
  run(['gh','label','create',label['name'],'--repo',args.repo,'--color',label['color'],'--description',label['description'],'--force'])
 existing=existing_issues(args.repo); by_key={}; remote=[]
 for n in nodes:
  body=str(ROOT/n['body_path']); labels=','.join(n['labels']); old=existing.get(n['key'])
  if old:
   number=str(old['number']); run(['gh','issue','edit',number,'--repo',args.repo,'--title',n['title'],'--body-file',body,'--add-label',labels]); action='updated'
  else:
   p=run(['gh','issue','create','--repo',args.repo,'--title',n['title'],'--body-file',body,'--label',labels]); number=p.stdout.strip().rstrip('/').split('/')[-1]; action='created'
  issue=json.loads(run(['gh','api',f'repos/{args.repo}/issues/{number}']).stdout); by_key[n['key']]=issue
  remote.append({'key':n['key'],'number':issue['number'],'id':issue['id'],'action':action,'url':issue['html_url']})
 for n in nodes:
  if not n['parent_key']: continue
  parent=by_key[n['parent_key']]; child=by_key[n['key']]
  p=run(['gh','api','-X','POST',f"repos/{args.repo}/issues/{parent['number']}/sub_issues",'-F',f"sub_issue_id={child['id']}"],allow_failure=True)
  if p.returncode and 'already' not in (p.stderr+p.stdout).lower() and '422' not in (p.stderr+p.stdout): raise RuntimeError(p.stderr or p.stdout)
 receipt={'schema_version':'org.searchright.github-issue-sync-receipt.v1','repository':args.repo,'mode':'apply','issues':remote,'relationships_attempted':len(nodes)-1,'claim_boundary':'Remote numbers and relationships were observed during this explicit apply run; Conductor remains canonical.'}
 text=json.dumps(receipt,indent=2,sort_keys=True)+'\n'; print(text,end='')
 if args.receipt_path:
  path=args.receipt_path if args.receipt_path.is_absolute() else ROOT/args.receipt_path; path.parent.mkdir(parents=True,exist_ok=True); path.write_text(text)
 return 0
if __name__=='__main__':
 try: raise SystemExit(main())
 except RuntimeError as exc: print(str(exc),file=sys.stderr); raise SystemExit(1)
