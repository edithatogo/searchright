$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required; install Rust via rustup"
}
rustup show active-toolchain | Out-Null
cargo generate-lockfile
cargo fetch --locked
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-features --locked
python scripts/validate_repository.py
Write-Host "Searchright bootstrap and deterministic smoke gates completed."
