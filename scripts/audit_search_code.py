#!/usr/bin/env python3
"""Find likely custom systematic-search code in one or more local repository roots."""
from __future__ import annotations
import argparse,hashlib,json,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TEXT_SUFFIXES={'.rs','.py','.jl','.r','.R','.js','.ts','.md','.csv','.yaml','.yml','.json','.toml','.sh','.ps1'}
SKIP={'.git','target','node_modules','.venv','venv','dist','build'}

def digest(path:Path)->str: return hashlib.sha256(path.read_bytes()).hexdigest()
def scan(root:Path,patterns:list[dict])->list[dict]:
 out=[]
 for path in sorted(root.rglob('*')):
  if not path.is_file() or path.suffix not in TEXT_SUFFIXES or any(p in SKIP for p in path.parts): continue
  try: text=path.read_text('utf-8')
  except UnicodeDecodeError: continue
  hits=[]
  for item in patterns:
   matches=list(re.finditer(item['regex'],text,re.I|re.M))
   if matches: hits.append({'pattern_id':item['id'],'count':len(matches),'replacement':item['replacement']})
  if hits: out.append({'path':path.relative_to(root).as_posix(),'sha256':digest(path),'hits':hits})
 return out

def self_test(patterns:list[dict])->bool:
 sample='https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi PubMed AND genome\nrecords_screened,12\n'
 found={p['id'] for p in patterns if re.search(p['regex'],sample,re.I|re.M)}
 return {'pubmed_endpoint','manual_prisma_log','search_strategy_string'} <= found

def main()->int:
 ap=argparse.ArgumentParser(); ap.add_argument('roots',nargs='*',type=Path); ap.add_argument('--output',type=Path); ap.add_argument('--self-test',action='store_true'); args=ap.parse_args()
 patterns=json.loads((ROOT/'migration/estate/patterns.json').read_text())['patterns']
 if args.self_test:
  ok=self_test(patterns); print(json.dumps({'status':'passed' if ok else 'failed','patterns':len(patterns)},indent=2)); return 0 if ok else 1
 roots=args.roots or [ROOT]
 repos=[]; errors=[]
 for root in roots:
  if not root.is_dir(): errors.append(f'not a directory: {root}'); continue
  repos.append({'root':str(root.resolve()),'findings':scan(root,patterns)})
 receipt={'schema_version':'org.searchright.estate-code-audit.v1','status':'passed' if not errors else 'failed','repositories':repos,'errors':errors,'limitations':['Pattern matches are candidates, not proof that code should be removed. Every replacement requires owner review and parity evidence.']}
 document=json.dumps(receipt,indent=2)+'\n'
 if args.output: args.output.write_text(document)
 print(document,end=''); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
