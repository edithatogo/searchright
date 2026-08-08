#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

command -v cargo >/dev/null 2>&1 || {
  echo "cargo is required; install Rust via rustup" >&2
  exit 2
}

rustup show active-toolchain >/dev/null
cargo generate-lockfile
cargo fetch --locked
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-features --locked
python3 scripts/validate_repository.py

echo "Searchright bootstrap and deterministic smoke gates completed."
