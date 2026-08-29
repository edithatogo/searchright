# CLI compatibility policy

Searchright's command-line interface is an alpha, scriptable adapter over
`SearchrightEngine`. The grouped `plan`, `source`, `strategy`, `run`, `import`,
`screen` and `report` hierarchy is the preferred interface. Existing top-level
operation aliases remain available during the alpha series so automation can
migrate without a flag day.

## Stable surfaces

- Successful machine operations emit JSON unless the command explicitly
  requests a textual artefact such as Mermaid, diagnostics, shell completions or
  a manual page.
- Usage failures exit with code `2`; operation failures exit with code `3`.
  Both use the versioned `org.searchright.cli-error.v1` JSON envelope on standard
  error and never describe partial output as safe.
- `init` is a dry run unless `--apply` is supplied. It refuses to overwrite an
  existing file. Other current commands are read-only, advisory or generate
  bytes on standard output.
- Command names, help text, dry-run JSON and usage-error JSON are checked against
  executable snapshots. Bash, Elvish, Fish, PowerShell and Zsh completions are
  generated with `searchright completions <shell>`; the roff manual is generated
  with `searchright manpage`.

## Change policy

Additive commands and optional fields may be introduced within the alpha
series. Removing or renaming a command, changing an exit code, changing a
versioned JSON envelope, or making a read-only command write requires a
compatibility review, snapshot update and migration note. A legacy alias is not
removed until its grouped replacement has executable parity and the release
notes announce the removal window.

CLI and MCP operation parity is governed by `contracts/interface-catalog.json`.
Presentation-only completion and manual-page generators are not facade
operations and do not confer network, write, screening or publication
authority.

## Evidence boundary

Checked-in tests and local receipts establish only the operating systems and
binaries on which they ran. Cross-platform support requires successful hosted
Linux, macOS and Windows receipts for the exact committed revision. Installation
and release evidence remain separate from source completeness.
