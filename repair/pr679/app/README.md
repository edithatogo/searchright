# CHHHS research-intelligence demonstration

This fixture-first demonstration uses SearchRight as the search execution boundary for a recurring institutional publication monitor for Cairns and Hinterland Hospital and Health Service (CHHHS).

It is intentionally conservative about institutional attribution. A Cairns geographic mention is not treated as evidence of CHHHS authorship. Candidate records retain source overlap, matched affiliation evidence, classification terms and review status.

## Demonstration

```bash
cd apps/chhhs-research-intelligence
python -m unittest discover -s tests
python chhhs_research_demo.py run --fixture --month 2026-08 --output-dir output
```

The command creates a deterministic JSON state file and HTML, JSON and CSV monthly reports.

## SearchRight adapter

For live execution, set `CHHHS_SEARCHRIGHT_COMMAND` to an approved SearchRight CLI or MCP adapter command that accepts one JSON request on standard input and emits either a JSON object containing `records` or JSON Lines records on standard output.

```bash
CHHHS_SEARCHRIGHT_COMMAND='path/to/searchright-adapter' \
  python chhhs_research_demo.py update --state state.json
```

The request declares the approved institutional aliases, durable identifiers, free providers and previous update watermark. The adapter is invoked without a shell. The app does not directly reimplement provider pagination, retries, receipts or network policy.

Configured discovery providers are PubMed, Europe PMC, Crossref and OpenAlex. Live execution remains opt-in and must obey SearchRight provider policy and egress controls.

## Claim boundary

This is a high-recall candidate monitor, not an authoritative or exhaustive CHHHS publication register. The alias and identifier register requires accountable institutional review. Rule-based themes are advisory and versioned; they do not replace human classification. GitHub Actions artefacts are demonstration outputs, not production storage or backup.
