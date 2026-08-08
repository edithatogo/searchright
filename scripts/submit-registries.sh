#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ "${SEARCHRIGHT_ALLOW_EXTERNAL_SUBMISSION:-}" != "YES" ]]; then
  echo "External submission is disabled. Set SEARCHRIGHT_ALLOW_EXTERNAL_SUBMISSION=YES after reviewing registry/status.json." >&2
  exit 2
fi

test -f Cargo.lock || { echo "Cargo.lock missing" >&2; exit 3; }
command -v mcp-publisher >/dev/null 2>&1 || { echo "mcp-publisher missing" >&2; exit 4; }
command -v smithery >/dev/null 2>&1 || { echo "smithery CLI missing" >&2; exit 5; }

python3 scripts/validate_repository.py
cargo test --workspace --all-features --locked
mcp-publisher validate server.json

echo "Validation passed. Publication remains split by target to preserve explicit approval."
echo "Official MCP Registry: mcp-publisher publish server.json"
echo "Smithery MCPB: smithery mcp publish dist/searchright.mcpb -n @edithatogo/searchright"
echo "Glama: authenticate with GitHub and submit https://github.com/edithatogo/searchright"
