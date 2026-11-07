# CSV dependency decision

## Scope

Use the exact `csv = "=1.4.0"` dependency for bibliographic CSV reading and
writing, and for Covidence/Rayyan CSV reading. This removes the two local CSV
parsers and the interchange field encoder. Screening exports keep their existing
`String` API and small escaping helper; changing that API is outside this patch.

This is a bounded dependency substitution. Record mapping, review authority,
decision values, quarantine rules and stable record identifiers stay in
Searchright. No parser dependency grants authority to apply screening decisions.

## Selection evidence

Evidence was checked on 2026-11-07. The observation of a release or repository
push is maintenance evidence, not a security audit or a promise of future support.

| Locked package | Published | Licence | Primary evidence |
| --- | --- | --- | --- |
| `csv` 1.4.0 | 2025-10-17 | MIT or Unlicense | [Version history](https://docs.rs/crate/csv/1.4.0); [tagged manifest](https://github.com/BurntSushi/rust-csv/blob/1.4.0/Cargo.toml) |
| `csv-core` 0.1.13 | 2025-10-17 | MIT or Unlicense | [Registry entry](https://github.com/rust-lang/crates.io-index/blob/master/cs/v-/csv-core); [tagged manifest](https://github.com/BurntSushi/rust-csv/blob/1.4.0/csv-core/Cargo.toml) |
| `ryu` 1.0.23 | 2026-02-08 | Apache-2.0 or BSL-1.0 | [Registry entry](https://github.com/rust-lang/crates.io-index/blob/master/3/r/ryu); [tagged manifest](https://github.com/dtolnay/ryu/blob/1.0.23/Cargo.toml) |

The [upstream CSV repository](https://github.com/BurntSushi/rust-csv) was not
archived and its GitHub metadata reported a last push of 2026-08-04. Version 1.4.0
followed 1.3.1 from 2024-11-09. Its declared minimum Rust version is 1.73, below
Searchright's pinned toolchain. `itoa`, `memchr` and `serde_core` already occur in
the locked workspace graph; the table lists the three added registry packages.
The exact checksums in `Cargo.lock` come from their official crates.io index
entries. Cargo must confirm resolution before a compiler-backed claim.

Existing licence, advisory and dependency review checks apply. The three new
exact versions have agent-authored source reviews recorded in `cargo vet` after
checking the published source identity. No exemption or publisher trust is added.

## Exact-source review

Reviewed on 2026-11-07 by OpenAI ChatGPT coding agents. The existing approved peer
registries did not supply exact-version audit coverage for these additions; the
reviews here are locally authored and retain that attribution.

The temporary [source-identity job](https://github.com/edithatogo/searchright/actions/runs/42357467565/job/128358607285)
downloaded the official published archives, verified all three SHA256 values
against `Cargo.lock`, and reported their source-file hashes and manifests without
executing package code. Every published `src/` file and original Cargo manifest
matched the reviewed upstream files; no additional source file was present.
All three normalized manifests set `build = false`. The temporary workflow was
removed after this comparison.

| Package | Reviewed source identity | Substantive checks |
| --- | --- | --- |
| `csv` 1.4.0 | [Release commit](https://github.com/BurntSushi/rust-csv/tree/4a3997e91d668ea1d8595bdef15625a77cf2308a); 12 source/manifest files matched | Four unchecked UTF-8 conversions and the private `StringRecord` validation/clearing invariants; reader/writer progress and caller-selected I/O |
| `csv-core` 0.1.13 | Four source/manifest files matched that release tree; archive VCS commit is `a0a3c9e4b493c216fb6d8284c9ac918444ba67f3` | No unsafe code or allocation; state-machine and caller-buffer bounds; builder creates the DFA used by the higher-level reader |
| `ryu` 1.0.23 | [Release commit](https://github.com/dtolnay/ryu/tree/f0b52bb194befe6fd242154f2182fafd43a819b8); 17 source files, original manifest and both licences matched | Sealed float API, initialized 24-byte output, pointer/index bounds; full power-table constants and all exponent-derived indices independently checked |

No unexpected network, process execution or ambient filesystem access was found
in the deployed paths. CSV's explicit file-opening APIs remain caller-directed;
Searchright passes in-memory readers and writers. Parser permissiveness and
input-dependent memory use remain documented limitations. Direct cloning or
default construction of a low-level core reader does not preserve/build its DFA
like the builder does; Searchright uses the higher-level builder path and does
not expose or clone core readers.

The review also identified repeated scans of a large quoted field when the core
writer exhausts its fixed-size output buffer. Searchright sizes the writer buffer
to at least the largest fully encoded field, using checked arithmetic. A field
therefore fits a fresh buffer, limiting continuation to one retry and keeping
scanning linear in field bytes without imposing a new size cap. Memory remains
proportional to the largest field and the materialized output.

These reviews support the recorded criterion for the deployed configurations;
they are not formal proofs, upstream attestations or exhaustive tests. No local
Rust/Miri/fuzz execution is claimed. Changes to enabled features or direct core
API use require reassessing these boundaries. Native integration tests and
advisory checks run separately in CI.

## Compatibility and corrected behaviour

- Bibliographic exports retain the unquoted header, always-quoted data fields,
  doubled quotes, LF record terminators and column order. Empty exports remain
  header-only. Public function signatures stay unchanged. Writer errors propagate
  through the new `InterchangeError::Csv` variant; downstream exhaustive matches
  on this enum may need updating.
- The writer buffer adapts to the largest output field, including joined authors
  and quote expansion, avoiding repeated scans through large fields. Checked
  length overflow becomes an input error rather than wrapping.
- Both readers use `has_headers(false)` so Searchright retains its field mapping,
  and `flexible(true)` so existing short rows and extra columns remain accepted.
  Bibliographic imports retain their filtering of records with only empty fields.
- A UTF-8 byte-order mark at the start of bibliographic input is now recognised,
  rather than becoming part of the first header name.
- Screening imports now read logical records. Quoted multiline rationale and
  exclusion-reason fields round-trip through the existing exporters. Error
  messages use the CSV record's starting physical line instead of counting split
  text lines. Whitespace-only records remain ignorable; an empty decision in a
  comma-delimited row remains an error.
- CSV quoting follows the upstream reader's deliberately permissive rules.
  Literal quotes inside an unquoted field are retained instead of being silently
  removed. An unclosed quoted field remains tolerated, as before. This is not a
  new strict CSV-validation guarantee. An unclosed screening rationale can absorb
  subsequent physical rows into that field; malformed input has no guarantee of
  preserving the previous line-based parser's interpretation.

The [upstream reader configuration documentation](https://docs.rs/csv/1.4.0/csv/struct.ReaderBuilder.html)
defines these parsing options. Regression cases cover LF, CRLF and CR input,
multiline fields, doubled and literal quotes, Unicode, BOM, empty input, variable
row widths, quarantine errors, deterministic output and the screening failure
paths. Existing interchange and screening fixtures remain part of the native
test run. Native tests and formatting must run with the pinned Rust toolchain;
static checks alone do not establish execution or downstream compatibility.
