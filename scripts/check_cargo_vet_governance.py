#!/usr/bin/env python3
"""Fail closed on unapproved cargo-vet imports, trust and exemptions."""

from __future__ import annotations

import datetime as dt
import json
import re
import subprocess
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
TRACK06_ID = "CVX-0259"
TRACK06_ISSUE = "https://github.com/edithatogo/searchright/issues/89"
TRACK06_DECIDED_AT = "2026-08-30T09:01:04Z"
TRACK06_EXPIRY = dt.date(2026, 9, 29)
TRACK06_RATIONALE = (
    "Track 06 locked quick-xml 0.41.0 default-only feature configuration; "
    "owner explicitly accepts unresolved package-wide audit risk until 2026-09-29; "
    "not an audit or safety certification"
)
TRACK06_RECEIPT = "verification/receipts/track-06-dependency-risk-approval.json"
TRACK06_CHECKSUM = "e660451e55124f798a69a5af3f49ccfbefbd41910eefd25caf2393e1f3473ec1"


def check_track06_scope(errors: list[str]) -> None:
    """The exception cannot silently follow a version or resolved-feature change."""
    packages = load_toml(ROOT / "Cargo.lock").get("package", [])
    selected = [p for p in packages if p.get("name") == "quick-xml"]
    if len(selected) != 1 or any(
        p.get("version") != "0.41.0" or p.get("checksum") != TRACK06_CHECKSUM
        or p.get("source") != "registry+https://github.com/rust-lang/crates.io-index"
        for p in selected
    ):
        errors.append("Track 06 quick-xml locked identity differs from owner approval")
        return
    try:
        for feature_flags in ([], ["--all-features"]):
            result = subprocess.run(
                ["cargo", "metadata", "--locked", "--offline", "--format-version", "1", *feature_flags],
                cwd=ROOT, capture_output=True, text=True, check=True, timeout=120,
            )
            metadata = json.loads(result.stdout)
            ids = {p["id"] for p in metadata["packages"] if p["name"] == "quick-xml"}
            nodes = [n for n in metadata["resolve"]["nodes"] if n["id"] in ids]
            if len(nodes) != 1 or nodes[0]["features"] != ["default"]:
                errors.append("Track 06 quick-xml resolved features differ from owner approval")
    except (OSError, subprocess.SubprocessError, ValueError, KeyError, TypeError) as error:
        errors.append(f"cannot verify Track 06 resolved feature scope: {type(error).__name__}")


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
            track06 = proposal_id == TRACK06_ID
            if proposal.get("crate") == "quick-xml" and not track06:
                errors.append("quick-xml requires the exact Track 06 owner decision id")
            required = ("owner", "rationale", "risk_summary", "replacement_plan", "linked_issue", "decided_at", "decision_evidence")
            if any(not proposal.get(field) for field in required):
                errors.append(f"approved proposal {proposal_id} lacks accountable decision evidence")
            if expires_at < today:
                errors.append(f"approved proposal {proposal_id} is expired")
            if proposal.get("owner") != REQUIRED_OWNER:
                errors.append(f"approved proposal {proposal_id} has the wrong accountable owner")
            if proposal.get("linked_issue") != (TRACK06_ISSUE if track06 else REQUIRED_ISSUE):
                errors.append(f"approved proposal {proposal_id} has the wrong linked issue")
            if expires_at != (TRACK06_EXPIRY if track06 else REQUIRED_REVIEW_BY):
                errors.append(f"approved proposal {proposal_id} has the wrong review deadline")
            if proposal.get("rationale") != (TRACK06_RATIONALE if track06 else REQUIRED_RATIONALE):
                errors.append(f"approved proposal {proposal_id} has altered rationale")
            if track06:
                if (proposal.get("crate"), version, proposal.get("criteria")) != ("quick-xml", "0.41.0", "safe-to-deploy"):
                    errors.append("Track 06 approval has altered dependency or criterion")
                if proposal.get("decided_at") != TRACK06_DECIDED_AT or proposal.get("decision_evidence") != [TRACK06_RECEIPT]:
                    errors.append("Track 06 approval has altered decision evidence")
                try:
                    receipt = load_json(ROOT / TRACK06_RECEIPT)
                except (OSError, ValueError):
                    receipt = {}
                if receipt.get("linked_issue") != TRACK06_ISSUE:
                    errors.append("Track 06 receipt has the wrong linked issue")
                if (receipt.get("decision"), receipt.get("owner"), receipt.get("checksum"), receipt.get("expires_at"), receipt.get("features"), receipt.get("crate"), receipt.get("version"), receipt.get("criterion"), receipt.get("recorded_at"), receipt.get("track_id")) != (
                    "approved", REQUIRED_OWNER, TRACK06_CHECKSUM, "2026-09-29", ["default"], "quick-xml", "0.41.0", "safe-to-deploy", TRACK06_DECIDED_AT, "06"
                ):
                    errors.append("Track 06 risk approval receipt differs from approved scope")
                check_track06_scope(errors)
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
            if crate == "quick-xml" and entry.get("version") == "0.41.0":
                expected_note = (
                    f"accountable_owner={REQUIRED_OWNER}; issue={TRACK06_ISSUE}; "
                    f"approved_on={TRACK06_DECIDED_AT}; review_by={TRACK06_EXPIRY}; "
                    f"rationale={TRACK06_RATIONALE}"
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
