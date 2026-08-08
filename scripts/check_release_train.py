#!/usr/bin/env python3
"""Validate the non-promoting cross-repository contract release train."""
from __future__ import annotations
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PATH=ROOT/'integration/release-train.json'

def main()->int:
 errors=[]; data=json.loads(PATH.read_text())
 if data.get('schema_version')!='org.searchright.integration-release-train.v1': errors.append('unexpected schema version')
 if data.get('automatic_promotion') is not False: errors.append('release train must require human promotion')
 for key in ('ecosystem_lock','contract_surface','public_package_policy'):
  value=data.get(key)
  if not isinstance(value,str) or not (ROOT/value).is_file(): errors.append(f'missing release-train control surface {key}: {value}')
 components=data.get('components',[]); orders=[x.get('promotion_order') for x in components if isinstance(x,dict)]
 if orders!=list(range(1,len(orders)+1)): errors.append('component promotion order must be contiguous')
 repos=[x.get('repository') for x in components if isinstance(x,dict)]
 if len(repos)!=len(set(repos)) or 'edithatogo/searchright' not in repos: errors.append('release-train repositories must be unique and include Searchright')
 for item in components:
  passport=item.get('passport')
  if passport is not None and not (ROOT/passport).is_file(): errors.append(f'missing passport {passport}')
 stages=data.get('stages',[]); ids=[x.get('id') for x in stages if isinstance(x,dict)]
 if ids!=['contract','consumer_fixture','compiler','downstream_canary','release_candidate','promotion']: errors.append('release stages differ from contract')
 if any(x.get('automatic') is not False for x in stages): errors.append('no release stage may promote automatically')
 if len(data.get('rollback',[]))<3: errors.append('release train requires rollback steps')
 if not errors:
  lock=json.loads((ROOT/data['ecosystem_lock']).read_text())
  if lock.get('automatic_updates') is not False: errors.append('ecosystem lock must not update automatically')
  packages=json.loads((ROOT/data['public_package_policy']).read_text())
  if any(item.get('publish_ready') for item in packages.get('packages',[])): errors.append('release train cannot proceed while a package is marked publish-ready without compiler evidence')
 receipt={'schema_version':'org.searchright.integration-release-train-receipt.v1','status':'failed' if errors else 'passed','components':len(components),'stages':len(stages),'automatic_promotions':0,'control_surfaces':3,'errors':errors,'limitations':['Static release-train consistency only; producer and consumer repositories were not built or released.']}
 print(json.dumps(receipt,indent=2,sort_keys=True)); return 1 if errors else 0
if __name__=='__main__': raise SystemExit(main())
