<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Test layout

Integration tests for `noyalib`. Each top-level `*.rs` file is a separate
test binary per Cargo convention.

## Thematic suites

These map to a single module or feature area and are the canonical place
to add tests for that area.

| File                                 | Covers                                         |
| ------------------------------------ | ---------------------------------------------- |
| `alias_dual_label.rs`                | Anchor & alias edge cases                      |
| `anchor_shared_ser.rs`               | Shared-anchor serialisation                    |
| `anchors.rs`                         | Anchor / alias core behaviour                  |
| `comments.rs`                        | Comment retention paths                        |
| `competitive_features.rs`            | Feature parity vs other YAML libs              |
| `competitive_features_full.rs`       | Extended competitive parity                    |
| `competitor_bugs.rs`                 | Regressions for bugs found in competitors      |
| `cst_*.rs`                           | Concrete-syntax-tree (`src/cst/`)              |
| `official_suite.rs`                  | YAML 1.2 official test suite (392 cases)       |
| `phase*.rs`                          | Phase-staged integration milestones            |
| `spec.rs`                            | YAML 1.2.2 spec compliance                     |
| `coverage_borrowed.rs` / `_full.rs`  | `src/borrowed.rs`                              |
| `coverage_de.rs`                     | `src/de.rs` deserializer                       |
| `coverage_error.rs`                  | `src/error.rs`                                 |
| `coverage_fmt.rs`                    | `src/fmt.rs` wrappers                          |
| `coverage_loader.rs` / `_full.rs`    | Multi-document loader                          |
| `coverage_regression.rs`             | Pinned regressions                             |
| `coverage_scanner.rs`                | `src/parser/scanner.rs`                        |
| `coverage_ser.rs`                    | `src/ser.rs` serializer                        |
| `coverage_spanned_anchors.rs`        | `Spanned<T>` + anchor interaction              |
| `coverage_value.rs`                  | `src/value.rs` Value type                      |

## Sweep / padding suites *(scheduled for consolidation post-launch)*

These accumulated during the v0.0.1 push to 95.7 % line coverage. Each
new sweep was added without merging earlier ones, so the file count
overstates the conceptual surface. They are valid tests and contribute
real coverage; they read as noise only because of historical accretion.

`coverage_100.rs`, `coverage_boost.rs`, `coverage_final.rs`,
`coverage_final_sweep.rs`, `coverage_final_sweep2.rs`,
`coverage_final_sweep3.rs`, `coverage_final_sweep4.rs`,
`coverage_full.rs`, `coverage_gaps.rs`, `coverage_misc.rs`,
`coverage_remaining.rs`.

**Planned post-launch:** consolidate into `tests/regression/`
submodules under a single `tests/regression.rs` parent binary,
halving link-time in CI and presenting one entry instead of eleven
in `cargo test` output.

## Where to add a new test

1. If it covers an existing module, append to the matching thematic
   suite above.
2. If it covers a new feature, create a new thematic suite named after
   the feature (e.g. `tests/json_schema_2020_12.rs`).
3. **Do not** add new `coverage_*` sweep files — open a PR against the
   consolidation instead.
