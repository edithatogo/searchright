# Repository standards alignment

Searchright inherits estate policy from
`edithatogo/repository-standards` through the exact revision and input paths in
`integration/passports/repository-standards-inheritance.json`.

## Local conformance evidence

- The pinned policy revision is
  `ad67bedaa0c4d0769bd54fd76354bac65b25b88c`.
- Searchright's deterministic static harness is the repository-local
  verification receipt required by the integration passport.
- Exact Rust 1.97.1 GNU compilation and all workspace tests passed locally on
  2026-08-12; the MSVC attempt was invalid because Git's POSIX `link.exe`
  shadowed the intended linker.
- Policy inheritance is read-only and revision-pinned. It does not imply that
  repository settings, badges, reusable workflows, or remote registry entries
  exist.

## Upstream registration state

Read-only GitHub API checks on 2026-08-12 found no
`edithatogo/searchright` entry in either `registry/repositories.json` or
`audits/latest/estate-conformance.json` at the pinned revision or current
`main`.

The prepared companion change packet is
`migration/companion-repositories/repository-standards.json`. Its
`remote_mutation_permitted` field is `false`, so applying registration and
running the upstream estate audit require separate explicit authority in the
repository-standards repository. Until an observed upstream commit and audit
receipt exist, Track 00 may claim local policy alignment only—not estate
registration or conformance.

## Completion evidence required

1. Add Searchright to the repository-standards registry under an approved
   research-infrastructure or Rust profile.
2. Run the repository-standards conformance audit against the exact Searchright
   revision.
3. Record the upstream commit, audit receipt, violations, exceptions, and
   rollback path in Searchright's Track 00 evidence.
