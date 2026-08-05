#!/usr/bin/env bash
set -euo pipefail

repo="https://github.com/gemini-cli-extensions/conductor"

if command -v gemini >/dev/null 2>&1; then
  echo "Installing/updating Conductor for Gemini CLI with auto-update enabled..."
  gemini extensions install "$repo" --auto-update
  exit 0
fi

if command -v agy >/dev/null 2>&1; then
  echo "Installing Conductor for Antigravity..."
  agy plugins install "$repo"
  exit 0
fi

if command -v claude >/dev/null 2>&1; then
  cat <<'EOF'
Claude Code was detected. Run these commands inside Claude Code:
  /plugin marketplace add gemini-cli-extensions/conductor
  /plugin install conductor
Then run:
  /conductor:conductor-setup
The Searchright Conductor artefacts are already checked in under conductor/.
EOF
  exit 0
fi

cat >&2 <<'EOF'
No supported Conductor host was found. Install Gemini CLI, Antigravity, or
Claude Code, then rerun this script. The repository context/tracks are already
present, but this is not a successful plugin installation.
EOF
exit 2
