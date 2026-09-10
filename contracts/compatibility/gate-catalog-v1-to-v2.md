# Gate catalogue v1 to v2 compatibility

## Change

`org.searchright.gate-catalog.v2` is an additive successor to the v1 source-only
catalogue. Version 2 represents repository scripts and native compiler/tool
commands in one evidence inventory without treating catalogue presence as
execution evidence.

Version 2 adds:

- `command_kind`;
- `program`;
- nullable `script` for non-script commands;
- boolean network and compiler capabilities derived from the command profile;
- the `compiler_verified` evidence ceiling;
- stable digest-suffixed identifiers for native commands.

The v1 schema and example remain immutable and readable. The canonical generated
catalogue moves to v2; consumers requiring v1 must continue reading the retained
v1 example or explicitly down-convert only `repository_script` entries.

## Migration

A v1 entry maps to v2 as follows:

```text
command_kind = repository_script
program = python
script = existing script
network = false
external_writes = false
compiler_required = false
evidence_ceiling = existing ceiling
```

Native commands have no valid v1 representation and must not be silently dropped
when a consumer claims complete gate coverage.

## Evidence boundary

The catalogue records the maximum evidence a command may establish after a
successful, receipt-bound execution. A listed Cargo, Python, Node or external
command is not a passing test. Version 2 does not promote live-provider,
methodological, remote-state, downstream or publication claims.
