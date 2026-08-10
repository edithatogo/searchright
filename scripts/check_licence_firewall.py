#!/usr/bin/env python3
"""Enforce integration origin, licence and redistribution boundaries."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INDEX = ROOT / "integration/passports/index.json"


def main() -> int:
    errors: list[str] = []
    warnings: list[str] = []
    index = json.loads(INDEX.read_text(encoding="utf-8"))
    checked = 0
    forks = 0
    reference_only = 0
    for entry in index.get("passports", []):
        path = ROOT / entry["path"]
        passport = json.loads(path.read_text(encoding="utf-8"))
        checked += 1
        role = passport.get("local_fork_role")
        upstream = passport.get("canonical_upstream")
        status = passport.get("licence_review_status")
        code = passport.get("code_license")
        content = passport.get("content_license")
        redistribution = passport.get("redistribution")
        drift = passport.get("drift_policy")
        if role == "original":
            if upstream is not None:
                errors.append(f"{entry['integration_id']}: original repository declares canonical upstream")
        else:
            forks += 1
            if not isinstance(upstream, dict) or not upstream.get("repository"):
                errors.append(f"{entry['integration_id']}: fork lacks canonical upstream")
            if upstream and upstream.get("repository") == passport.get("repository"):
                errors.append(f"{entry['integration_id']}: fork and canonical upstream are identical")
        if not code or not content or not redistribution or not drift:
            errors.append(f"{entry['integration_id']}: incomplete licence firewall metadata")
        if "NOASSERTION" in {code, content} and status not in {"reference_only", "review_required"}:
            errors.append(f"{entry['integration_id']}: NOASSERTION material must be reference-only or review-required")
        if status == "reference_only":
            reference_only += 1
            if passport.get("mode") == "rust_dependency":
                errors.append(f"{entry['integration_id']}: reference-only integration cannot be a Rust dependency")
        if status == "review_required":
            warnings.append(f"{entry['integration_id']}: licence review still required")
        if passport.get("automatic_revision_updates") is not False:
            errors.append(f"{entry['integration_id']}: revision updates must remain review-gated")
    receipt = {
        "schema_version": "org.searchright.licence-firewall-receipt.v1",
        "status": "failed" if errors else "passed",
        "passports_checked": checked,
        "forks_classified": forks,
        "reference_only_integrations": reference_only,
        "warnings": warnings,
        "errors": errors,
        "claim_boundary": "This static firewall records declared licence and origin boundaries; it is not legal advice or a substitute for rights-holder review.",
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
