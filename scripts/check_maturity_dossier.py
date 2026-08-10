#!/usr/bin/env python3
"""Validate the evidence-scaled maturity dossier and release decision."""
from __future__ import annotations
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
EXPECTED={
 'contracts','compiler','determinism','providers','methodology','security','interfaces','migration','usability','operations',
 'github_control_plane','downstream_compatibility','access_and_tenancy','backup_restore_incidents','sdk_and_adoption','pilots','registries'
}
READY_STATES={'passed','externally_validated','publicly_accepted'}
def main()->int:
 errors=[]; data=json.loads((ROOT/'conductor/maturity-dossier.json').read_text())
 domains=data.get('domains',[]); names=[x.get('domain') for x in domains if isinstance(x,dict)]
 if set(names)!=EXPECTED or len(names)!=len(set(names)): errors.append(f'maturity domains differ: {sorted(set(names)^EXPECTED)}')
 blockers=[]
 for domain in domains:
  if not domain.get('state') or 'critical_blocker' not in domain: errors.append(f'invalid domain {domain}'); continue
  if domain.get('critical_blocker'): blockers.append(domain.get('domain'))
 if blockers and data.get('decision')!='not_ready': errors.append('critical blockers require not_ready decision')
 if not blockers and any(domain.get('state') not in READY_STATES for domain in domains): errors.append('non-ready domain state exists without a critical blocker')
 if data.get('decision')=='ready' and blockers: errors.append('ready decision cannot contain blockers')
 for path in ['docs/maturity/1.0-gate.md','docs/maturity/gap-register.md','docs/maturity/release-decision.md']:
  if not (ROOT/path).is_file(): errors.append(f'missing {path}')
 receipt={'schema_version':'org.searchright.maturity-dossier-receipt.v1','status':'failed' if errors else 'passed','decision':data.get('decision'),'domains':len(domains),'critical_blockers':sorted(blockers),'errors':errors,'limitations':['Static dossier consistency only; it cannot generate missing compiler, live, human or external evidence.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
