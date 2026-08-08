# ADR 0010: Accessible diagnostics and institutional governance are core concerns

- Status: accepted
- Date: 2026-08-06

## Context

Research software often treats accessibility, privacy, retention and deployment
policy as UI or hosting details. In Searchright they affect every interface and
whether sensitive operations may occur.

## Decision

Diagnostics use stable codes and support plain text, JSON and JSONL without
colour dependence. Locale-neutral identifiers are separated from message text.
Institutional policies classify data and evaluate purpose, retention, storage,
export, deletion and approval before sensitive actions. Decisions are auditable
and deny by default when required information is missing.

## Consequences

- CLI, MCP and embedded hosts share accessible machine-readable failures.
- Institutional deployments can layer approved policy without forking the core.
- Policy evaluation is not represented as legal or privacy compliance advice.
