#!/usr/bin/env python3
"""Generate or check content hashes for the canonical context spine."""
from __future__ import annotations
import argparse,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]; MANIFEST=ROOT/'context/manifest.json'; LOCK=ROOT/'context/context-lock.json'
def digest(p:Path)->str: return hashlib.sha256(p.read_bytes()).hexdigest()
def build()->dict:
 m=json.loads(MANIFEST.read_text()); paths=[ROOT/x['path'] for x in m['required_context']]
 return {'schema_version':'org.searchright.context-lock.v1','generated_at':'source-epoch:2026-08-08','files':[{'path':p.relative_to(ROOT).as_posix(),'sha256':digest(p),'size':p.stat().st_size} for p in sorted(paths)],'claim_boundary':'Hash parity detects context drift; it does not establish semantic correctness or promote evidence.'}
def main()->int:
 ap=argparse.ArgumentParser(); ap.add_argument('--check',action='store_true'); a=ap.parse_args(); expected=json.dumps(build(),indent=2,sort_keys=True)+'\n'
 if a.check:
  ok=LOCK.is_file() and LOCK.read_text()==expected
  print(json.dumps({'schema_version':'org.searchright.context-lock-receipt.v1','status':'passed' if ok else 'failed','files':len(build()['files'])},indent=2)); return 0 if ok else 1
 LOCK.write_text(expected); print(expected,end=''); return 0
if __name__=='__main__': raise SystemExit(main())
