# ADR 0002: Contract-first model and append-only ledger

Status: accepted

Canonical schemas precede adapters. Material state changes append versioned,
hash-chained events. Derived snapshots may be rebuilt from events and cannot be
used as the sole audit source.
