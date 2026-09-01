## Contract and track

- Conductor track: `36`
- Multi-track exception: `none`
- Pilot: CHHHS institutional publication-intelligence demonstration

## Purpose

Add a fixture-first demonstration that uses SearchRight as the provider-execution boundary for recurring discovery of publications potentially attributable to Cairns and Hinterland Hospital and Health Service (CHHHS).

The demonstration conservatively links provider records, retains source overlap and source identifiers, separates confirmed, probable, review-required and insufficient institutional evidence, applies versioned explainable themes and study-type rules, persists incremental local state, and renders deterministic HTML, JSON and CSV monthly reports.

## Repair of the previous branch

The earlier branch contained only a temporary payload-applicator workflow. This revision replaces that delivery mechanism with the actual reviewed source files on a clean base from current `main`. It does not weaken workflow hardening, source-integrity, compiler, lint, coverage, security or one-track scope checks.

## Verification contract

Before leaving draft, the exact PR head must pass:

- Python fixture regression tests and deterministic monthly report generation;
- repository validation and the complete static harness;
- workspace tests with all features;
- all-target, all-feature Clippy with warnings denied;
- `scripts/verify.sh`; and
- the complete hosted check matrix.

## Claim boundary

This is a high-recall candidate-monitor demonstration. It does not establish exhaustive CHHHS publication coverage, validated institutional-attribution accuracy, current live-provider compatibility, an authoritative research register, or production storage and governance readiness. The institutional identity register and any staff or service comparisons require accountable review.
