# Resolved dependency scan maintenance — 20 November 2026

Primary track: 16. This bounded maintenance slice corrects the manifest inventory
claim and adds a separate exact Cargo.lock inventory for Grype. It does not reopen
the archived track or advance source/native/provider/release evidence ceilings.

Manifest requirements no longer populate installed-version or package-URL fields.
Inherited workspace licences resolve to strings. Focused regressions enforce this
boundary and dependency-reference integrity. The release verification job enforces
critical findings before binary assembly/publication; the separate source workflow
is advisory and records the checked-out commit.

Relevant tests, baseline failures and unavailable native/hosted gates are recorded
in the scoped PR. Required candidate and hosted gates remain mandatory.
