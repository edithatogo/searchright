# 27: Accessibility, internationalisation and usability

## Objective

Ensure every public interface can provide stable, understandable diagnostics
without colour dependence and can evolve toward translated human-facing text
without changing audit semantics.

## Scope

- Stable diagnostic codes and severity/action fields.
- Plain text, JSON and JSONL rendering.
- Locale-neutral identifiers and separable messages.
- Keyboard, screen-reader and terminal-width design contracts.
- Information-specialist usability protocol and issue taxonomy.

## Requirements owned

See `conductor/requirements.md` rows whose Track owner includes `27`.

## Acceptance contract

Diagnostic contracts and renderers are source-implemented. Accessibility,
localisation and human-usability claims require executable snapshots and
participant evidence.

## Out of scope

No claim is made that every MCP client, terminal, IDE or future UI is accessible.
