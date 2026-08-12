#!/usr/bin/env python3
"""Fail closed on unapproved cargo-vet imports, trust and exemptions."""

from __future__ import annotations

import datetime as dt
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
STORE = ROOT / "supply-chain"
APPROVED_IMPORTS = {
    "bytecode-alliance": "https://raw.githubusercontent.com/bytecodealliance/wasmtime/main/supply-chain/audits.toml",
    "embark-studios": "https://raw.githubusercontent.com/EmbarkStudios/rust-ecosystem/main/audits.toml",
    "mozilla": "https://raw.githubusercontent.com/mozilla/supply-chain/main/audits.toml",
    "zcash": "https://raw.githubusercontent.com/zcash/rust-ecosystem/main/supply-chain/audits.toml",
}
EXACT_VERSION = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")
REQUIRED_OWNER = "edithatogo"
REQUIRED_ISSUE = "https://github.com/edithatogo/searchright/issues/241"
REQUIRED_REVIEW_BY = dt.date(2026, 11, 10)
REQUIRED_RATIONALE = (
    "locked baseline dependency required by current workspace; no qualifying "
    "imported audit yet; temporary backlog exception, not an audit or safety certification"
)


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def main() -> int:
    errors: list[str] = []
    config = load_toml(STORE / "config.toml")
    ledger = load_json(STORE / "exemption-proposals.json")

    imports = config.get("imports", {})
    observed_imports = {name: value.get("url") for name, value in imports.items()}
    if observed_imports != APPROVED_IMPORTS:
        errors.append("cargo-vet imports differ from the exact approved peer registry allowlist")

    if config.get("trusted"):
        errors.append("cargo-vet publisher trust is prohibited")

    proposals = ledger.get("proposals", [])
    approved: set[tuple[str, str, str]] = set()
    seen_ids: set[str] = set()
    today = dt.date.today()
    maximum_days = ledger.get("policy", {}).get("maximum_duration_days")
    if maximum_days != 90:
        errors.append("maximum exemption duration must remain 90 days")

    for proposal in proposals:
        proposal_id = proposal.get("id")
        if proposal_id in seen_ids:
            errors.append(f"duplicate exemption proposal id: {proposal_id}")
        seen_ids.add(proposal_id)
        version = proposal.get("version", "")
        if not EXACT_VERSION.fullmatch(version):
            errors.append(f"proposal {proposal_id} does not use an exact crate version")
        try:
            proposed_at = dt.datetime.fromisoformat(proposal["proposed_at"].replace("Z", "+00:00")).date()
            expires_at = dt.date.fromisoformat(proposal["expires_at"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"proposal {proposal_id} has invalid dates")
            continue
        if expires_at <= proposed_at or (expires_at - proposed_at).days > 90:
            errors.append(f"proposal {proposal_id} exceeds the bounded exemption duration")
        if proposal.get("status") == "approved":
            required = ("owner", "rationale", "risk_summary", "replacement_plan", "linked_issue", "decided_at", "decision_evidence")
            if any(not proposal.get(field) for field in required):
                errors.append(f"approved proposal {proposal_id} lacks accountable decision evidence")
            if expires_at < today:
                errors.append(f"approved proposal {proposal_id} is expired")
            if proposal.get("owner") != REQUIRED_OWNER:
                errors.append(f"approved proposal {proposal_id} has the wrong accountable owner")
            if proposal.get("linked_issue") != REQUIRED_ISSUE:
                errors.append(f"approved proposal {proposal_id} has the wrong linked issue")
            if expires_at != REQUIRED_REVIEW_BY:
                errors.append(f"approved proposal {proposal_id} has the wrong review deadline")
            if proposal.get("rationale") != REQUIRED_RATIONALE:
                errors.append(f"approved proposal {proposal_id} has altered rationale")
            approved.add((proposal.get("crate", ""), version, proposal.get("criteria", "")))

    exemptions = config.get("exemptions", {})
    effective: set[tuple[str, str, str]] = set()
    for crate, entries in exemptions.items():
        for entry in entries:
            criteria = entry.get("criteria", [])
            if isinstance(criteria, str):
                criteria = [criteria]
            expected_note = (
                f"accountable_owner={REQUIRED_OWNER}; issue={REQUIRED_ISSUE}; "
                f"approved_on=2026-08-12T00:00:00+10:00; review_by={REQUIRED_REVIEW_BY}; "
                f"rationale={REQUIRED_RATIONALE}"
            )
            if entry.get("notes") != expected_note:
                errors.append(f"cargo-vet exemption {crate} {entry.get('version')} has missing or altered governance notes")
            if entry.get("suggest", True) is not True:
                errors.append(f"cargo-vet exemption {crate} {entry.get('version')} must remain suggested for review")
            for criterion in criteria:
                effective.add((crate, entry.get("version", ""), criterion))

    if effective != approved:
        missing_approvals = sorted(effective - approved)
        stale_approvals = sorted(approved - effective)
        if missing_approvals:
            errors.append(f"effective cargo-vet exemptions lack exact approvals: {missing_approvals}")
        if stale_approvals:
            errors.append(f"approved ledger entries lack exact effective exemptions: {stale_approvals}")

    receipt = {
        "schema_version": "org.searchright.cargo-vet-governance-check.v1",
        "status": "failed" if errors else "passed",
        "approved_imports": sorted(observed_imports),
        "proposal_count": len(proposals),
        "approved_proposal_count": sum(item.get("status") == "approved" for item in proposals),
        "effective_exemption_count": len(effective),
        "errors": errors,
        "limitations": [
            "This check validates governance metadata and exact peer import URLs; it does not audit dependency source code.",
            "Approved exemptions are temporary backlog risk acceptances, not dependency audits or safety certification.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
