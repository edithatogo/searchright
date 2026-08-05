$ErrorActionPreference = "Stop"
$repo = "https://github.com/gemini-cli-extensions/conductor"

if (Get-Command gemini -ErrorAction SilentlyContinue) {
    Write-Host "Installing/updating Conductor for Gemini CLI with auto-update enabled..."
    & gemini extensions install $repo --auto-update
    exit $LASTEXITCODE
}
if (Get-Command agy -ErrorAction SilentlyContinue) {
    Write-Host "Installing Conductor for Antigravity..."
    & agy plugins install $repo
    exit $LASTEXITCODE
}
if (Get-Command claude -ErrorAction SilentlyContinue) {
    Write-Host @"
Claude Code was detected. Run these commands inside Claude Code:
  /plugin marketplace add gemini-cli-extensions/conductor
  /plugin install conductor
Then run /conductor:conductor-setup.
The Searchright Conductor artefacts are already checked in under conductor/.
"@
    exit 0
}
throw "No supported Conductor host found. Install Gemini CLI, Antigravity, or Claude Code."
