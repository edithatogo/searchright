#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

static_only=false
if [[ "${1:-}" == "--static-only" ]]; then
  static_only=true
fi

python3 scripts/validate_repository.py

if $static_only; then
  exit 0
fi

command -v cargo >/dev/null 2>&1 || {
  echo "cargo is unavailable; rerun with --static-only or install Rust" >&2
  exit 2
}

test -f Cargo.lock || {
  echo "Cargo.lock is required for deterministic verification" >&2
  exit 3
}

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo nextest run --workspace --all-features --locked
cargo test --workspace --all-features --doc --locked
cargo doc --workspace --all-features --no-deps --locked
cargo deny check
cargo machete
