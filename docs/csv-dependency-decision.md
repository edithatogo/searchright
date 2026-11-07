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

Existing licence, advisory, dependency review and `cargo vet` admission gates
apply to these versions. This decision adds no audit assertion or exemption.

## Compatibility and corrected behaviour

- Bibliographic exports retain the unquoted header, always-quoted data fields,
  doubled quotes, LF record terminators and column order. Empty exports remain
  header-only. Public function signatures stay unchanged. Writer errors propagate
  through the new `InterchangeError::Csv` variant; downstream exhaustive matches
  on this enum may need updating.
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
