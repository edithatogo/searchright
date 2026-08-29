<!-- searchright-issue-key: track-10-phase-4-task-03 -->
# Track 10 / Phase 4 / Task 03

Parent phase key: `track-10-phase-4`
Conductor plan: `conductor/tracks/10-mcp-mvp/plan.md`
Canonical task state: **source task complete**.

## Canonical task

- [x] Run compiler-backed Conductor review and append review fixes after Cargo gates execute.
  - Review fix `232003620e3a087f4025ca7a8e957f92cb169b51`: Close authority-boundary, immutable idempotency, exact source-preservation, structured-output, resource and prompt review findings while restoring architecture, source-structure and generated-state gates.
  - Review fix `8912bf8440d5b073a9aa7e1f673acbed5ce674d8`: Fix exact official-client receipt invocation through libtest and make interrupted receipt generation safely retryable without weakening the clean source-tree gate.
  - Review fix `945694cc1004d0dacf09f3bd60a42107f1d46f84`: Close the workspace-wide strict Clippy gate by keeping the screening decision enum import scoped to store tests.
  - Review fix `896e88ac39456be6d535eefc61b725c60c0a16b3`: Make advanced MCP pagination and authority-spoof assertions panic-safe under the workspace Clippy policy.
  - Review fix `c72fc1e76d2100116e0ab2f4e260889c5c5c04ba`: Remove the unused searchright facade UUID declaration reported by the hosted cargo-machete gate and regenerate the source SBOM and hash manifest.
  - Review fix `c1414d5cb592c67a544f0c9390fa0e9e7fd41411`: Refresh the locked searchright package dependency list after removing the redundant UUID declaration so locked builds remain reproducible.
  - Review fix `e1cbf61522c065f0743c4f3186c9c96de2859499`: Scope the GitHub CLI authentication preflight to github.com so unrelated configured enterprise-host credentials cannot block the required read-only control-plane audit.
  - Review fix `366da722d751e27ec4328757f382650537ab7cb8`: Bind trusted-host authority to exact bytes in every canonical local-store family, including same-length changes and nested state, while rejecting symbolic links.

## Completion and evidence contract

- This issue mirrors one top-level checklist item in the Conductor plan.
- Nested checklist entries remain acceptance details inside this issue.
- Closing a source-complete task does not promote the parent track's evidence level.
- Reopening or closing is synchronised only from the canonical Conductor checklist.
- Higher-evidence, downstream, human and registry gates require their own receipts.
