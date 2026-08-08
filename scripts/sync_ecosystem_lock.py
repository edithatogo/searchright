#!/usr/bin/env python3
"""Generate the non-self-referential evidence-infrastructure component lock."""
from __future__ import annotations
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'integration/ecosystem-lock.json'
def sha(path:Path)->str: return hashlib.sha256(path.read_bytes()).hexdigest()
def build()->dict:
 cargo=(ROOT/'Cargo.toml').read_text()
 version=re.search(r'\[workspace\.package\].*?\nversion = "([^"]+)"',cargo,re.S).group(1)
 rust=re.search(r'\[workspace\.package\].*?\nrust-version = "([^"]+)"',cargo,re.S).group(1)
 rmcp=re.search(r'^rmcp = \{ version = "([^"]+)"',cargo,re.M).group(1)
 index=json.loads((ROOT/'integration/passports/index.json').read_text())
 passports={x['repository']:x for x in index['passports']}
 components=[
  {'id':'searchright','kind':'product','repository':'edithatogo/searchright','version':version,'revision':'record_in_release_receipt','source_contracts_sha256':sha(ROOT/'contracts/schema-catalog.json'),'public_package_policy_sha256':sha(ROOT/'release/public-packages.json'),'rust_toolchain':rust},
  {'id':'citeweft','kind':'product','repository':'edithatogo/citeweft','version':'0.1.0','revision':passports['edithatogo/citeweft']['revision'],'passport':passports['edithatogo/citeweft']['path']},
  {'id':'sourceright','kind':'product','repository':'edithatogo/sourceright','version':'0.1.20-observed','revision':passports['edithatogo/sourceright']['revision'],'passport':passports['edithatogo/sourceright']['path']},
  {'id':'mcp-rust-sdk','kind':'protocol_sdk','package':'rmcp','version':rmcp,'protocol_policy':'MCP protocol negotiation and transcript evidence required'},
  {'id':'reporting-standard-packs','kind':'content_pack','repository':'edithatogo/standards_check','revision':passports['edithatogo/standards_check']['revision'],'passport':passports['edithatogo/standards_check']['path'],'licence_gate':'review_required'},
  {'id':'repository-policy','kind':'policy_pack','repository':'edithatogo/repository-standards','revision':passports['edithatogo/repository-standards']['revision'],'passport':passports['edithatogo/repository-standards']['path']},
  {'id':'screening-benchmark','kind':'benchmark','canonical_repository':'asreview/synergy-dataset','local_fork':'edithatogo/synergy-dataset','local_revision':passports['edithatogo/synergy-dataset']['revision'],'run_pin':'required_per_benchmark_receipt'}
 ]
 return {'schema_version':'org.searchright.ecosystem-lock.v1','source_epoch':'2026-08-09','release_train':'searchright-evidence-infrastructure','automatic_updates':False,'components':components,'promotion_order':['contracts','citeweft','searchright','sourceright','standard-packs-and-benchmarks'],'required_receipts':['compiler','consumer-fixture','downstream-canary','licence','SBOM','reproducibility','attestation'],'claim_boundary':'The lock fixes observed component identities and local contract digests; the Searchright release commit and external corpus release must be added to the executed release receipt.'}
def main()->int:
 ap=argparse.ArgumentParser(); ap.add_argument('--check',action='store_true'); args=ap.parse_args(); d=build(); text=json.dumps(d,indent=2)+"\n"
 if args.check:
  if not OUT.is_file() or OUT.read_text()!=text:
   print(json.dumps({'status':'failed','error':'ecosystem lock is stale'},indent=2)); return 1
  print(json.dumps({'status':'passed','components':len(d['components']),'automatic_updates':False},indent=2)); return 0
 OUT.write_text(text); print(json.dumps({'status':'updated','components':len(d['components'])},indent=2)); return 0
if __name__=='__main__': raise SystemExit(main())
