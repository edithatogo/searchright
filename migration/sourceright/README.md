# Sourceright shared-core migration packet

## Observed source boundary

The requested repository name `edithatogo/sourcerightlibrary` was not resolved
on 2026-08-05. The active repository observed during this design pass was
`edithatogo/sourceright`. The inspected `main`-branch blob for
`src/live_providers.rs` was
`57bc071c6afc7d5a4cb8ead12112919a446ebd24`.

That file currently combines four different concerns:

1. Sourceright-specific conversion from provider payloads into
   `AcademicProviderResult` and `CslItem` comparisons.
2. Provider endpoint construction and source-specific response parsing.
3. Generic execution policy: enablement, timeout, rate interval, retries,
   caching and environment configuration.
4. Smoke-test orchestration and its result/report model.

Only concerns 2 and 3 belong in the shared search infrastructure. Concern 1
remains in Sourceright. Concern 4 becomes a thin Sourceright compatibility
workflow over `evidence-search-core` rather than a second runtime.

## Target boundary

| Existing Sourceright responsibility | Target | Cutover state |
| --- | --- | --- |
| `LiveProviderConfig` environment parsing | Sourceright compatibility facade translating into `ExecutionPolicy` and provider manifests | Planned |
| `LiveProviderRuntimeControls` | `searchright_contracts::ExecutionPolicy` | Contract available; parity unproven |
| `LiveProviderExecution` | `evidence_search_core::ProviderMode` | Source available; parity unproven |
| `fetch_json`, `fetch_text`, retry and interval control | `evidence_search_core::ProviderRegistry` | Source available; compile/integration unproven |
| Endpoint builders | `searchright-connectors` provider adapters | Europe PMC and PubMed scaffolded; other providers planned |
| File cache helpers | Content-addressed replay/cache service in shared core | Not implemented; retain temporarily |
| Fixture payload parsing | Provider adapter parser, then Sourceright-specific result translation | Europe PMC partially scaffolded; other providers planned |
| `AcademicProviderResult` production and CSL comparison | Sourceright | Retained |
| `LiveProviderSmokeState` public report | Sourceright facade backed by shared receipts plus compatibility fields | Planned |

The machine-readable mapping is in `replacement-map.yaml`.

## Migration contract

No existing Sourceright code is deleted solely because Searchright contains a
similarly named abstraction. Replacement requires all of the following:

1. Pin the source Sourceright commit and copy its live-provider fixtures into a
   licence/provenance-aware parity harness.
2. Add `evidence-search-core` behind a disabled-by-default Sourceright feature,
   for example `shared-search-core`.
3. Produce old-runtime and shared-runtime outputs for every fixture. Compare
   provider classification, identifiers, candidate fields, error codes,
   execution mode, retry behaviour and redacted endpoint evidence.
4. Test disabled-live behaviour, zero/maximum budgets, malformed responses,
   retryable and non-retryable failures, cache/replay corruption, redirect
   handling, undeclared hosts and secret redaction.
5. Resolve intentional differences in an approved parity waiver. Security
   hardening may change behaviour, but it must never be hidden as equivalence.
6. Switch the default only after Sourceright's full test, lint, documentation,
   fixture and smoke gates pass on the shared implementation.
7. Retain the old runtime behind a rollback feature for at least one compatible
   release. Remove it only in a bounded, separately reviewable change.

## Required compatibility facade

The Sourceright-side facade should remain small and product-specific:

```rust,ignore
pub struct SourcerightProviderFacade {
    registry: evidence_search_core::ProviderRegistry,
}

impl SourcerightProviderFacade {
    pub async fn verify_candidate(
        &self,
        canonical: &CslItem,
        provider: AcademicProvider,
        policy: searchright_contracts::ExecutionPolicy,
    ) -> Result<AcademicProviderResult, SourcerightError> {
        // Build a bounded shared-core request, execute it, then translate the
        // returned bibliographic record into Sourceright's citation-verification
        // model. The shared receipt is retained in result provenance.
    }
}
```

This sketch is deliberately not compiled in either workspace. Its exact public
shape must be derived from the pinned Sourceright commit and reviewed as a
Sourceright API change.

## Release and rollback gates

- No coordinated crate release before `Cargo.lock`, package dry-runs and licence
  checks exist in both repositories.
- No claim that custom code has been replaced before a merged Sourceright pull
  request and a parity receipt identify the removed symbols.
- No caching cutover until the shared core has equivalent or safer replay/cache
  semantics.
- No endpoint may be accepted from arbitrary runtime input; provider manifests
  declare allowed HTTPS hosts and redirects remain disabled by default.
- Rollback must be one feature/configuration change, not a source-code restore.

## Evidence to attach to the future Sourceright pull request

- pinned before/after commits;
- fixture inventory and licence/provenance record;
- old/new parity matrix, including approved differences;
- shared-core execution and redaction receipts;
- security/adversarial results;
- benchmark and performance comparison;
- semver and deprecation decision;
- rollback exercise receipt;
- documentation migration and release notes.

No remote Sourceright branch or pull request was created by this repository
generation.
