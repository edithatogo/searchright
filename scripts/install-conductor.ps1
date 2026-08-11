$ErrorActionPreference = "Stop"
$repo = "https://github.com/gemini-cli-extensions/conductor"
$releaseTag = "conductor-v0.3.0"
$releaseCommit = "32f44820450576e60a0685de602fff38bfd85609"
$geminiInstallDir = Join-Path $HOME ".gemini\extensions\conductor"

if (Get-Command gemini -ErrorAction SilentlyContinue) {
    if (Test-Path -LiteralPath $geminiInstallDir) {
        throw @"
Conductor is already installed at $geminiInstallDir. This installer will not
silently replace it or inherit an existing auto-update setting. Inspect the
installed version, uninstall it explicitly if replacement is intended, and
rerun this script to install the pinned $releaseTag baseline.
"@
    }
    Write-Host "Installing Conductor $releaseTag at exact commit $releaseCommit for Gemini CLI (auto-update disabled)..."
    & gemini extensions install $repo --ref $releaseCommit
    exit $LASTEXITCODE
}
if (Get-Command agy -ErrorAction SilentlyContinue) {
    throw @"
Antigravity was detected, but its plugin installer does not expose a verified
exact-ref and auto-update policy compatible with Searchright's pinned
$releaseTag baseline. No installation was attempted. Use Gemini CLI with the
exact-ref installer or document and verify equivalent host pin semantics first.
"@
}
if (Get-Command claude -ErrorAction SilentlyContinue) {
    throw @"
Claude Code was detected, but the marketplace commands available to this
repository do not expose a verified exact-ref and auto-update policy for the
pinned Conductor 0.3.0 baseline. No installation was attempted. Use Gemini CLI
with the exact-ref installer or document and verify equivalent host pin
semantics first. The Searchright Conductor artefacts remain checked in.
"@
}
throw "No supported Conductor host found. Install Gemini CLI, Antigravity, or Claude Code."
