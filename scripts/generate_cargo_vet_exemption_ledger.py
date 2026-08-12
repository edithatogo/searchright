#!/usr/bin/env python3
"""Bind generated cargo-vet backlog exemptions to accountable decisions."""

from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "supply-chain" / "config.toml"
LEDGER = ROOT / "supply-chain" / "exemption-proposals.json"
OWNER = "edithatogo"
ISSUE = "https://github.com/edithatogo/searchright/issues/241"
APPROVED_ON = "2026-08-12T00:00:00+10:00"
REVIEW_BY = "2026-11-10"
RATIONALE = (
    "locked baseline dependency required by current workspace; no qualifying "
    "imported audit yet; temporary backlog exception, not an audit or safety certification"
)
RISK_SUMMARY = "The exact dependency version lacks qualifying safe-to-run or safe-to-deploy audit evidence."
REPLACEMENT_PLAN = "Replace the exemption with an imported or Searchright-authored audit before review_by."


def exemption_rows(config: dict) -> list[tuple[str, str, str]]:
    rows: list[tuple[str, str, str]] = []
    for crate, entries in config.get("exemptions", {}).items():
        for entry in entries:
            criteria = entry["criteria"]
            if isinstance(criteria, list):
                for criterion in criteria:
                    rows.append((crate, entry["version"], criterion))
            else:
                rows.append((crate, entry["version"], criteria))
    return sorted(rows)


def main() -> int:
    config_text = CONFIG.read_text(encoding="utf-8")
    config = tomllib.loads(config_text)
    rows = exemption_rows(config)
    note = (
        f"accountable_owner={OWNER}; issue={ISSUE}; approved_on={APPROVED_ON}; "
        f"review_by={REVIEW_BY}; rationale={RATIONALE}"
    )

    block = re.compile(
        r'(\[\[exemptions\.[^\]]+\]\]\nversion = "[^"]+"\ncriteria = (?:"[^"]+"|\[[^\]]+\]))'
        r'(?:\nnotes = (?:""".*?"""|"[^"\n]*"))?',
        re.DOTALL,
    )
    rendered, replacements = block.subn(lambda match: f'{match.group(1)}\nnotes = """{note}"""', config_text)
    if replacements != len(rows):
        raise SystemExit(
            f"refusing partial config rewrite: replaced {replacements} blocks for "
            f"{len(rows)} exemption rows"
        )
    CONFIG.write_text(rendered, encoding="utf-8", newline="\n")

    proposals = []
    for index, (crate, version, criteria) in enumerate(rows, start=1):
        proposals.append(
            {
                "id": f"CVX-{index:04d}",
                "crate": crate,
                "version": version,
                "criteria": criteria,
                "status": "approved",
                "owner": OWNER,
                "rationale": RATIONALE,
                "risk_summary": RISK_SUMMARY,
                "replacement_plan": REPLACEMENT_PLAN,
                "linked_issue": ISSUE,
                "proposed_at": APPROVED_ON,
                "decided_at": APPROVED_ON,
                "decision_evidence": [ISSUE],
                "expires_at": REVIEW_BY,
            }
        )
    ledger = {
        "schema_version": "org.searchright.cargo-vet-exemption-proposals.v1",
        "policy": {
            "default_status": "not_authorized",
            "maximum_duration_days": 90,
            "wildcard_publisher_trust": "prohibited",
            "claim_boundary": (
                "Each approved entry is a temporary backlog risk acceptance, not an audit, "
                "trust grant or safety certification."
            ),
        },
        "proposals": proposals,
    }
    LEDGER.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"exemptions": len(rows), "review_by": REVIEW_BY}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
