#!/usr/bin/env bash
set -euo pipefail

readonly repo="https://github.com/gemini-cli-extensions/conductor"
readonly release_tag="conductor-v0.3.0"
readonly release_commit="32f44820450576e60a0685de602fff38bfd85609"
readonly gemini_install_dir="${HOME}/.gemini/extensions/conductor"

if command -v gemini >/dev/null 2>&1; then
  if [[ -e "$gemini_install_dir" ]]; then
    cat >&2 <<EOF
Conductor is already installed at $gemini_install_dir. This installer will not
silently replace it or inherit an existing auto-update setting. Inspect the
installed version, uninstall it explicitly if replacement is intended, and
rerun this script to install the pinned $release_tag baseline.
EOF
    exit 3
  fi
  echo "Installing Conductor $release_tag at exact commit $release_commit for Gemini CLI (auto-update disabled)..."
  gemini extensions install "$repo" --ref "$release_commit"
  exit 0
fi

if command -v agy >/dev/null 2>&1; then
  cat >&2 <<EOF
Antigravity was detected, but its plugin installer does not expose a verified
exact-ref and auto-update policy compatible with Searchright's pinned
$release_tag baseline. No installation was attempted. Use Gemini CLI with the
exact-ref installer or document and verify equivalent host pin semantics first.
EOF
  exit 3
fi

if command -v claude >/dev/null 2>&1; then
  cat <<'EOF'
Claude Code was detected, but the marketplace commands available to this
repository do not expose a verified exact-ref and auto-update policy for the
pinned Conductor 0.3.0 baseline. No installation was attempted. Use Gemini CLI
with the exact-ref installer or document and verify equivalent host pin
semantics first. The Searchright Conductor artefacts remain checked in.
EOF
  exit 3
fi

cat >&2 <<'EOF'
No supported Conductor host was found. Install Gemini CLI, Antigravity, or
Claude Code, then rerun this script. The repository context/tracks are already
present, but this is not a successful plugin installation.
EOF
exit 2
