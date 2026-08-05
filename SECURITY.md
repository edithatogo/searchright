# Security Policy

## Supported versions

Only the latest tagged technical-preview release receives security fixes until
1.0. Pre-release interfaces may change with migration notes.

## Reporting

Do not open public issues for vulnerabilities, leaked credentials, private review
records or licensed full text. Use GitHub private vulnerability reporting on the
canonical repository. Include impact, affected versions, reproduction steps and
whether live provider credentials or review data were exposed.

## Security invariants

- No secret may be persisted in a review artifact or audit event.
- Network egress is denied unless a provider is enabled by the review plan and
  runtime policy.
- Licensed databases are bring-your-own-access and are never scraped by default.
- Tool write operations are dry-run first and require explicit application.
- Agents cannot silently alter eligibility criteria, final inclusion decisions or
  protocol history.
- Provider responses are untrusted input and must be size-limited, parsed without
  code execution and retained only under the configured data policy.
- Experimental WebAssembly connectors run with capability-scoped WASI permissions.

See `docs/security/threat-model.md` and `docs/security/egress-policy.md`.
