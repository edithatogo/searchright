#!/usr/bin/env python3
"""Check the source-level CLI/MCP/facade operation catalogue without compilation."""
from __future__ import annotations
import json,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CATALOG=ROOT/'contracts/interface-catalog.json'
CLI=(ROOT/'crates/searchright-cli/src/main.rs').read_text()
MCP=(ROOT/'crates/searchright-mcp/src/lib.rs').read_text()
ENGINE=(ROOT/'crates/searchright/src/engine.rs').read_text()
errors=[]
data=json.loads(CATALOG.read_text())
entries=data.get('entries',[])
seen=set()
for i,e in enumerate(entries):
    op=e.get('operation','')
    if not op or op in seen: errors.append(f'entry {i}: duplicate/empty operation {op!r}')
    seen.add(op)
    cli=e.get('cli_variant',''); mcp=e.get('mcp_tool',''); facade=e.get('facade_method','')
    if not re.search(rf'^\s*{re.escape(cli)}\b',CLI,re.M): errors.append(f'{op}: missing CLI variant {cli}')
    if not re.search(rf'^\s*(?:async\s+)?fn\s+{re.escape(mcp)}\s*\(',MCP,re.M): errors.append(f'{op}: missing MCP tool {mcp}')
    if not re.search(rf'^\s*pub\s+(?:async\s+)?fn\s+{re.escape(facade)}\s*\(',ENGINE,re.M): errors.append(f'{op}: missing facade method {facade}')
# Ensure every public facade method is catalogued except constructors/helpers represented elsewhere.
public=set(re.findall(r'^\s*pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(',ENGINE,re.M))
declared={e['facade_method'] for e in entries}
uncatalogued=sorted(public-declared)
if uncatalogued: errors.append(f'uncatalogued facade methods: {uncatalogued}')
receipt={'schema_version':'org.searchright.interface-parity-receipt.v1','status':'passed' if not errors else 'failed','operations_checked':len(entries),'errors':errors,'limitations':['Source-level parity only; no CLI binary or MCP protocol transcript was executed.']}
print(json.dumps(receipt,indent=2))
sys.exit(1 if errors else 0)
