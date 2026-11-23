#!/usr/bin/env python3
"""Check the non-executable source-selection inventory without network access."""

from __future__ import annotations

from datetime import date
import json
from pathlib import Path
import re
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
CATALOGUE = ROOT / "data/source-selection/catalogue.v1.json"
SLUG = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
DISPOSITIONS = {"under_review", "assessed", "deferred", "retired"}
VERIFICATION = {"first_party_identity_checked", "requires_review", "conflict_unresolved"}
EVIDENCE = {"not_claimed", "existing_contract_unverified", "manual_method_documented"}
ACCESS = {"free", "licensed", "mixed", "unknown"}
KINDS = {"bibliographic_database", "review_database", "trial_register", "discovery_index"}
ROOT_KEYS = {"schema_version", "authority", "observed_on", "sources", "source_directory_reference", "limitations"}
SOURCE_KEYS = {"source_id", "source_name", "source_kind", "aliases", "first_party_url", "observed_on", "verification_state", "disposition", "issue_number", "limitations", "platforms"}
PLATFORM_KEYS = {"platform_id", "platform_name", "access", "execution_evidence", "existing_contract_path"}


def _valid_date(value: object) -> bool:
    if not isinstance(value, str):
        return False
    try:
        return date.fromisoformat(value).isoformat() == value
    except ValueError:
        return False


def validate_catalogue(data: object) -> list[str]:
    """Return deterministic fail-closed errors; no access or runtime certification."""
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["catalogue must be an object"]
    if set(data) != ROOT_KEYS:
        errors.append("catalogue fields must match the versioned internal inventory contract")
    if data.get("schema_version") != "org.searchright.source-selection-catalogue.v1":
        errors.append("unsupported schema_version")
    if data.get("authority") != "advisory_only":
        errors.append("catalogue authority must remain advisory_only")
    if not _valid_date(data.get("observed_on")):
        errors.append("invalid catalogue observed_on")
    if data.get("source_directory_reference") != "https://refhunter.org/en/":
        errors.append("source_directory_reference must be link-only RefHunter entrypoint")
    if not isinstance(data.get("limitations"), list) or not data["limitations"]:
        errors.append("catalogue requires limitations")
    sources = data.get("sources")
    if not isinstance(sources, list) or not sources:
        return errors + ["sources must be a nonempty list"]

    seen_sources: set[str] = set()
    seen_aliases: dict[str, str] = {}
    platform_definitions: dict[str, tuple[str, str]] = {}
    encountered_ids: list[str] = []
    for i, source in enumerate(sources):
        label = f"sources[{i}]"
        if not isinstance(source, dict):
            errors.append(f"{label}: source must be an object")
            continue
        if set(source) != SOURCE_KEYS:
            errors.append(f"{label}: unrecognised or missing source field")
        source_id = source.get("source_id")
        if not isinstance(source_id, str) or not SLUG.fullmatch(source_id):
            errors.append(f"{label}: invalid source_id")
            continue
        encountered_ids.append(source_id)
        if source_id in seen_sources:
            errors.append(f"{label}: duplicate source_id {source_id}")
        seen_sources.add(source_id)
        if not isinstance(source.get("source_name"), str) or not source["source_name"].strip():
            errors.append(f"{source_id}: source_name is required")
        if source.get("source_kind") not in KINDS:
            errors.append(f"{source_id}: invalid source_kind")
        if source.get("verification_state") not in VERIFICATION:
            errors.append(f"{source_id}: invalid verification_state")
        if source.get("disposition") not in DISPOSITIONS:
            errors.append(f"{source_id}: invalid disposition")
        if source.get("disposition") == "assessed" and source.get("verification_state") != "first_party_identity_checked":
            errors.append(f"{source_id}: unverified source cannot be assessed")
        issue = source.get("issue_number")
        if issue is not None and (type(issue) is not int or issue <= 0):
            errors.append(f"{source_id}: invalid issue_number")
        url = source.get("first_party_url")
        if not isinstance(url, str) or urlsplit(url).scheme != "https" or not urlsplit(url).hostname:
            errors.append(f"{source_id}: invalid first_party_url")
        elif urlsplit(url).hostname in {"refhunter.org", "www.refhunter.org"}:
            errors.append(f"{source_id}: RefHunter is secondary, not first-party evidence")
        if not _valid_date(source.get("observed_on")):
            errors.append(f"{source_id}: invalid observed_on")
        limitations = source.get("limitations")
        if not isinstance(limitations, list) or not limitations or any(not isinstance(v, str) or not v.strip() for v in limitations):
            errors.append(f"{source_id}: substantive limitations required")
        aliases = source.get("aliases")
        if not isinstance(aliases, list) or any(not isinstance(a, str) or not a.strip() for a in aliases):
            errors.append(f"{source_id}: invalid aliases")
            aliases = []
        for name in [source.get("source_name"), *aliases]:
            if not isinstance(name, str):
                continue
            normalized = name.casefold().strip()
            other = seen_aliases.get(normalized)
            if other is not None and other != source_id:
                errors.append(f"{source_id}: alias collides with {other}: {name}")
            seen_aliases[normalized] = source_id

        platforms = source.get("platforms")
        if not isinstance(platforms, list) or not platforms:
            errors.append(f"{source_id}: platforms must be a nonempty list")
            continue
        seen_pairs: set[tuple[str, str]] = set()
        for j, platform in enumerate(platforms):
            if not isinstance(platform, dict):
                errors.append(f"{source_id}: platform {j} must be an object")
                continue
            if set(platform) != PLATFORM_KEYS:
                errors.append(f"{source_id}: unrecognised or missing platform field")
            platform_id = platform.get("platform_id")
            if not isinstance(platform_id, str) or not SLUG.fullmatch(platform_id):
                errors.append(f"{source_id}: invalid platform_id")
                continue
            pair = (source_id, platform_id)
            if pair in seen_pairs:
                errors.append(f"{source_id}: duplicate source/platform pair {platform_id}")
            seen_pairs.add(pair)
            name, access = platform.get("platform_name"), platform.get("access")
            if not isinstance(name, str) or not name.strip() or access not in ACCESS:
                errors.append(f"{source_id}: invalid platform name/access")
                continue
            current = (name, access)
            earlier = platform_definitions.get(platform_id)
            if earlier is not None and earlier != current:
                errors.append(f"{source_id}: inconsistent platform {platform_id}")
            platform_definitions[platform_id] = current
            evidence = platform.get("execution_evidence")
            if evidence not in EVIDENCE:
                errors.append(f"{source_id}: invalid execution_evidence {evidence}")
            path = platform.get("existing_contract_path")
            if path is not None and (
                not isinstance(path, str)
                or not path.strip()
                or path.startswith("/")
                or ".." in Path(path).parts
            ):
                errors.append(f"{source_id}: invalid existing_contract_path")
            if evidence in {"existing_contract_unverified", "manual_method_documented"} and not path:
                errors.append(f"{source_id}: existing contract status requires exact path")
            if evidence == "not_claimed" and path is not None:
                errors.append(f"{source_id}: not_claimed must not imply an existing contract")

    if encountered_ids != sorted(encountered_ids):
        errors.append("sources must sort by source_id")
    return errors


def main() -> int:
    data = json.loads(CATALOGUE.read_text(encoding="utf-8"))
    errors = validate_catalogue(data)
    sources = data.get("sources", []) if isinstance(data, dict) else []
    print(json.dumps({
        "schema_version": "org.searchright.source-catalogue-check.v1",
        "status": "failed" if errors else "passed",
        "sources_checked": len(sources) if isinstance(sources, list) else 0,
        "network_operations": 0,
        "external_writes": 0,
        "errors": errors,
        "claim_boundary": "Advisory metadata identity and fixture invariants only; no current API, licence, methodological adequacy or live-provider certification.",
    }, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
