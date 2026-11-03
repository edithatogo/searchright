#!/usr/bin/env python3
"""Validate dataset provenance, upstream boundaries, and licence firewalls."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SYNERGY_PASSPORT = ROOT / "integration/passports/synergy-screening-benchmark.json"
MANIFEST = ROOT / "benchmarks/methodology/manifest.json"


def check_dataset_provenance() -> dict[str, Any]:
    """Execute dataset provenance and licence firewall verification."""
    errors: list[str] = []
    passports_checked = 0
    upstreams_checked = 0

    if not SYNERGY_PASSPORT.is_file():
        errors.append("missing SYNERGY integration passport")
    else:
        passports_checked += 1
        passport = json.loads(SYNERGY_PASSPORT.read_text(encoding="utf-8"))
        if passport.get("schema_version") != "org.searchright.integration-passport.v1":
            errors.append("unexpected passport schema version")
        if passport.get("integration_id") != "synergy-screening-benchmark":
            errors.append("unexpected integration_id in SYNERGY passport")
        if passport.get("repository") != "edithatogo/synergy-dataset":
            errors.append("passport repository must identify the personal mirror")

        upstream = passport.get("canonical_upstream", {})
        if not isinstance(upstream, dict) or upstream.get("repository") != "asreview/synergy-dataset":
            errors.append("canonical upstream repository must be asreview/synergy-dataset")
        if upstream.get("default_branch") != "master":
            errors.append("canonical upstream default branch must be master")

        role = passport.get("local_fork_role")
        if role not in {"mirror", "mirror_or_patch_carrier_only"}:
            errors.append(f"local fork role must be mirror or patch carrier, got {role!r}")

        for lic_field in ("code_license", "content_license"):
            if passport.get(lic_field) != "CC0-1.0":
                errors.append(f"passport {lic_field} must be CC0-1.0")

        for flag in ("default_network", "default_external_writes", "default_telemetry", "automatic_revision_updates"):
            if passport.get(flag) is not False:
                errors.append(f"passport {flag} must be false")

        caps = passport.get("allowed_capabilities", [])
        for req_cap in ("dataset_read", "benchmark_report_write"):
            if req_cap not in caps:
                errors.append(f"passport missing allowed capability {req_cap}")

        claim = str(passport.get("claim_boundary", "")).lower()
        if "benchmark upstream" not in claim or "no performance" not in claim:
            errors.append("passport claim boundary must disclaim performance claims before versioned publication")

    if not MANIFEST.is_file():
        errors.append("missing methodology benchmark manifest")
    else:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        upstreams = manifest.get("external_upstreams", [])
        synergy = next((u for u in upstreams if u.get("name") == "SYNERGY"), None)
        if not synergy:
            errors.append("missing SYNERGY entry in external_upstreams of methodology manifest")
        else:
            upstreams_checked += 1
            if synergy.get("canonical_repository") != "asreview/synergy-dataset":
                errors.append("manifest SYNERGY canonical repository must be asreview/synergy-dataset")
            if synergy.get("local_fork_role") != "mirror_or_patch_carrier_only":
                errors.append("manifest SYNERGY local fork role must be mirror_or_patch_carrier_only")
            if synergy.get("labels_may_enter_prompts") is not False:
                errors.append("manifest SYNERGY labels_may_enter_prompts must be false")

    return {
        "schema_version": "org.searchright.dataset-provenance-receipt.v1",
        "status": "failed" if errors else "passed",
        "passports_checked": passports_checked,
        "upstreams_checked": upstreams_checked,
        "errors": errors,
        "claim_boundary": (
            "Dataset provenance checks verify canonical upstream boundaries and mirror roles; "
            "no benchmark data were executed."
        ),
    }


def main() -> int:
    receipt = check_dataset_provenance()
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
