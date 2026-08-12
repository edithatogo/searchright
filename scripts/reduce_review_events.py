#!/usr/bin/env python3
"""Deterministically reduce a pre-verified Searchright audit stream.

This reducer does not recompute BLAKE3 event hashes because it is deliberately
stdlib-only. Callers must supply the verified audit head produced by the Rust
ledger verifier; the reducer checks linkage, review identity, event identity,
and human-authority invariants before deriving a disposable snapshot.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import unicodedata
from collections import Counter
from collections.abc import Iterable
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "org.searchright.review-state-snapshot.v1"
GENESIS = "GENESIS"
HEX64 = set("0123456789abcdef")
ROOT = Path(__file__).resolve().parents[1]
EVENT_REGISTRY = ROOT / "contracts" / "events" / "registry.json"


class ReductionError(ValueError):
    """Raised when an event stream cannot be safely reduced."""


def canonical_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def valid_hash(value: object) -> bool:
    return isinstance(value, str) and len(value) == 64 and set(value) <= HEX64


def load_events(path: Path) -> list[dict[str, Any]]:
    events: list[dict[str, Any]] = []
    for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw.strip():
            continue
        try:
            value = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise ReductionError(f"line {number} is not JSON: {exc}") from exc
        if not isinstance(value, dict):
            raise ReductionError(f"line {number} must be a JSON object")
        events.append(value)
    return events


def load_event_registry() -> tuple[dict[str, dict[str, Any]], int, set[str]]:
    """Load and structurally validate the fail-closed event registry."""
    raw = json.loads(EVENT_REGISTRY.read_text(encoding="utf-8"))
    if raw.get("schema_version") != "org.searchright.audit-event-registry.v1":
        raise ReductionError("event registry has an unsupported schema_version")
    if raw.get("unknown_event_type_policy") != "reject":
        raise ReductionError("event registry must reject unknown event types")
    if raw.get("unknown_payload_version_policy") != "reject":
        raise ReductionError("event registry must reject unknown payload versions")
    rows: dict[str, dict[str, Any]] = {}
    for row in raw.get("event_types", []):
        event_type = row.get("event_type")
        if not isinstance(event_type, str) or not event_type or event_type in rows:
            raise ReductionError("event registry contains an invalid or duplicate event type")
        allowed_keys = row.get("allowed_payload_keys")
        field_types = row.get("payload_field_types")
        if (
            not isinstance(allowed_keys, list)
            or not all(isinstance(key, str) and key for key in allowed_keys)
            or not isinstance(field_types, dict)
            or set(field_types) != set(allowed_keys)
            or not all(value in {"boolean", "integer", "string"} for value in field_types.values())
        ):
            raise ReductionError(f"event registry field types are invalid for {event_type}")
        rows[event_type] = row
    maximum_payload_bytes = raw.get("maximum_payload_bytes")
    prohibited_payload_keys = raw.get("prohibited_payload_keys")
    if not isinstance(maximum_payload_bytes, int) or maximum_payload_bytes < 1:
        raise ReductionError("event registry maximum_payload_bytes is invalid")
    if not isinstance(prohibited_payload_keys, list) or not all(
        isinstance(key, str) and key for key in prohibited_payload_keys
    ):
        raise ReductionError("event registry prohibited_payload_keys is invalid")
    return rows, maximum_payload_bytes, set(prohibited_payload_keys)


def nested_keys(value: Any) -> set[str]:
    """Return object keys at every depth for sensitive-field denial."""
    if isinstance(value, dict):
        return set(value) | set().union(*(nested_keys(item) for item in value.values()))
    if isinstance(value, list):
        return set().union(*(nested_keys(item) for item in value))
    return set()


def normalize_payload(
    event_type: str,
    payload: dict[str, Any],
    registry: dict[str, dict[str, Any]],
    maximum_payload_bytes: int,
    prohibited_payload_keys: set[str],
) -> dict[str, Any]:
    """Project a payload copy to its current version without rewriting the event."""
    row = registry.get(event_type)
    if row is None:
        raise ReductionError(f"unregistered event_type {event_type or '[empty]'}")
    if len(canonical_bytes(payload)) > maximum_payload_bytes:
        raise ReductionError(f"event_type {event_type} payload exceeds the configured size limit")
    denied = nested_keys(payload) & prohibited_payload_keys
    if denied:
        raise ReductionError(
            f"event_type {event_type} payload contains prohibited keys {sorted(denied)}"
        )
    allowed_keys = set(row.get("allowed_payload_keys", []))
    unexpected_keys = set(payload) - allowed_keys
    if unexpected_keys:
        raise ReductionError(
            f"event_type {event_type} payload contains unregistered keys {sorted(unexpected_keys)}"
        )
    current = row.get("current_payload_version")
    version = payload.get("_schema_version", row.get("legacy_unversioned_payload_version"))
    if not isinstance(version, int) or not isinstance(current, int):
        raise ReductionError(f"event_type {event_type} has a non-integer payload version")
    known_versions = {
        item.get("version") for item in row.get("versions", []) if isinstance(item, dict)
    }
    if version not in known_versions:
        raise ReductionError(f"event_type {event_type} has unsupported payload version {version}")

    normalized = copy.deepcopy(payload)
    while version != current:
        matching_plan: dict[str, Any] | None = None
        for relative in row.get("migrations", []):
            plan = json.loads((ROOT / relative).read_text(encoding="utf-8"))
            if (
                plan.get("event_type") == event_type
                and plan.get("from_payload_version") == version
            ):
                matching_plan = plan
                break
        if matching_plan is None:
            raise ReductionError(
                f"event_type {event_type} has no migration from payload version {version}"
            )
        if (
            matching_plan.get("destructive") is not False
            or matching_plan.get("original_event_immutable") is not True
        ):
            raise ReductionError(f"event_type {event_type} migration is not preservation-safe")
        for transformation in matching_plan.get("transformations", []):
            operation = transformation.get("operation")
            if operation == "rename":
                source = str(transformation.get("from", "")).removeprefix("/")
                target = str(transformation.get("to", "")).removeprefix("/")
                if source not in normalized or not source or not target or target in normalized:
                    raise ReductionError(f"event_type {event_type} rename precondition failed")
                normalized[target] = normalized.pop(source)
            elif operation == "replace":
                path = str(transformation.get("path", "")).removeprefix("/")
                if not path:
                    raise ReductionError(f"event_type {event_type} replace path is invalid")
                normalized[path] = transformation.get("value")
            else:
                raise ReductionError(f"event_type {event_type} migration operation is unsupported")
        next_version = matching_plan.get("to_payload_version")
        if not isinstance(next_version, int) or next_version <= version:
            raise ReductionError(f"event_type {event_type} migration does not advance")
        version = next_version
    field_types = row["payload_field_types"]
    for key, value in normalized.items():
        expected = field_types[key]
        valid = (
            (expected == "boolean" and isinstance(value, bool))
            or (expected == "integer" and isinstance(value, int) and not isinstance(value, bool) and value >= 0)
            or (
                expected == "string"
                and isinstance(value, str)
                and bool(value.strip())
                and len(value) <= 512
                and not any(unicodedata.category(character) == "Cc" for character in value)
            )
        )
        if not valid:
            raise ReductionError(
                f"event_type {event_type} payload key {key} is not a valid {expected}"
            )
    return normalized


def validate_linkage(events: Iterable[dict[str, Any]], verified_head: str) -> tuple[str, str]:
    review_id: str | None = None
    previous = GENESIS
    ids: set[str] = set()
    last_id = ""
    count = 0
    for index, event in enumerate(events):
        count += 1
        if event.get("schema_version") != "org.searchright.audit-event.v1":
            raise ReductionError(f"event {index} has an unsupported schema_version")
        event_id = event.get("event_id")
        current_review = event.get("review_id")
        event_hash = event.get("event_hash")
        if not isinstance(event_id, str) or not event_id:
            raise ReductionError(f"event {index} has no event_id")
        if event_id in ids:
            raise ReductionError(f"duplicate event_id {event_id}")
        ids.add(event_id)
        if not isinstance(current_review, str) or not current_review:
            raise ReductionError(f"event {index} has no review_id")
        if review_id is None:
            review_id = current_review
        elif current_review != review_id:
            raise ReductionError(f"event {index} mixes review {current_review} into {review_id}")
        if event.get("previous_hash") != previous:
            raise ReductionError(f"event {index} does not point to the preceding event")
        if not valid_hash(event_hash):
            raise ReductionError(f"event {index} has a non-canonical event_hash")
        previous = event_hash
        last_id = event_id
    if count == 0:
        raise ReductionError("at least one event is required")
    if verified_head != previous:
        raise ReductionError("verified head does not match the supplied event stream")
    return review_id or "", last_id


def reduce_events(events: list[dict[str, Any]], verified_head: str) -> dict[str, Any]:
    review_id, last_event_id = validate_linkage(events, verified_head)
    event_registry, maximum_payload_bytes, prohibited_payload_keys = load_event_registry()
    source_counts: Counter[str] = Counter()
    search_runs: dict[str, dict[str, Any]] = {}
    final_decisions: dict[str, dict[str, Any]] = {}
    recommendations: dict[str, list[dict[str, Any]]] = {}
    amendments: list[str] = []
    rejected_authority_events: list[str] = []
    unknown_event_types: set[str] = set()
    status = "created"
    plan_validated = False

    for event in events:
        event_type = str(event.get("event_type", ""))
        source_counts[event_type] += 1
        raw_payload = event.get("payload") if isinstance(event.get("payload"), dict) else {}
        payload = normalize_payload(
            event_type,
            raw_payload,
            event_registry,
            maximum_payload_bytes,
            prohibited_payload_keys,
        )
        actor = event.get("actor") if isinstance(event.get("actor"), dict) else {}
        actor_type = str(actor.get("actor_type", ""))
        event_id = str(event["event_id"])

        if event_type == "review_plan_validated":
            plan_validated = True
            status = "planned"
        elif event_type == "search_run_completed":
            run_id = payload.get("run_id")
            if isinstance(run_id, str) and run_id:
                search_runs[run_id] = {
                    "run_id": run_id,
                    "source_id": payload.get("source_id"),
                    "record_count": payload.get("record_count"),
                    "event_id": event_id,
                }
                status = "searching"
        elif event_type == "protocol_amended":
            amendment_id = payload.get("amendment_id")
            if isinstance(amendment_id, str) and amendment_id and amendment_id not in amendments:
                amendments.append(amendment_id)
        elif event_type == "review_status_changed":
            next_status = payload.get("status")
            if isinstance(next_status, str) and next_status:
                status = next_status
        elif event_type == "screening_decision_recorded":
            record_id = payload.get("record_id")
            decision = payload.get("decision")
            final_authority = payload.get("final_authority") is True
            if not isinstance(record_id, str) or not record_id:
                unknown_event_types.add("malformed:screening_decision_recorded")
                continue
            decision_entry = {
                "record_id": record_id,
                "stage": payload.get("stage"),
                "decision": decision,
                "reviewer_id": payload.get("reviewer_id") or actor.get("actor_id"),
                "event_id": event_id,
            }
            if final_authority:
                if actor_type != "human":
                    rejected_authority_events.append(event_id)
                else:
                    final_decisions[record_id] = decision_entry
                    status = "screening"
            else:
                recommendations.setdefault(record_id, []).append(decision_entry)
        else:  # pragma: no cover - the registry and reducer handlers must stay aligned
            unknown_event_types.add(event_type or "[empty]")

    decision_counts = Counter(
        str(item.get("decision", "unknown")) for item in final_decisions.values()
    )
    state: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "review_id": review_id,
        "state_version": 1,
        "source_event_count": len(events),
        "source_head_hash": verified_head,
        "last_event_id": last_event_id,
        "status": status,
        "plan_validated": plan_validated,
        "search_runs": [search_runs[key] for key in sorted(search_runs)],
        "protocol_amendments": sorted(amendments),
        "screening": {
            "final_decisions": [final_decisions[key] for key in sorted(final_decisions)],
            "final_decision_counts": dict(sorted(decision_counts.items())),
            "advisory_recommendation_count": sum(len(items) for items in recommendations.values()),
            "rejected_final_authority_event_ids": sorted(rejected_authority_events),
        },
        "event_type_counts": dict(sorted(source_counts.items())),
        "unknown_event_types": sorted(unknown_event_types),
        "claim_boundary": (
            "Derived disposable state from an audit stream whose BLAKE3 head was verified externally; "
            "the snapshot is not the canonical audit ledger and non-human final decisions are rejected."
        ),
    }
    state["state_sha256"] = hashlib.sha256(canonical_bytes(state)).hexdigest()
    return state


def self_test() -> dict[str, Any]:
    h1 = "1" * 64
    h2 = "2" * 64
    h3 = "3" * 64
    h4 = "4" * 64
    events = [
        {
            "schema_version": "org.searchright.audit-event.v1",
            "event_id": "e1",
            "review_id": "review-demo",
            "event_type": "review_plan_validated",
            "occurred_at": "2026-08-08T00:00:00Z",
            "actor": {"actor_id": "librarian-1", "actor_type": "human", "provenance": None},
            "payload": {"plan_id": "review-demo"},
            "previous_hash": GENESIS,
            "event_hash": h1,
        },
        {
            "schema_version": "org.searchright.audit-event.v1",
            "event_id": "e2",
            "review_id": "review-demo",
            "event_type": "search_run_completed",
            "occurred_at": "2026-08-08T00:01:00Z",
            "actor": {"actor_id": "searchright", "actor_type": "cli", "provenance": "0.1"},
            "payload": {"run_id": "run-1", "source_id": "pubmed", "record_count": 42},
            "previous_hash": h1,
            "event_hash": h2,
        },
        {
            "schema_version": "org.searchright.audit-event.v1",
            "event_id": "e3",
            "review_id": "review-demo",
            "event_type": "screening_decision_recorded",
            "occurred_at": "2026-08-08T00:02:00Z",
            "actor": {"actor_id": "ranker", "actor_type": "agent", "provenance": "model:test"},
            "payload": {"record_id": "record-1", "stage": "title_abstract", "decision": "exclude", "final_authority": True},
            "previous_hash": h2,
            "event_hash": h3,
        },
        {
            "schema_version": "org.searchright.audit-event.v1",
            "event_id": "e4",
            "review_id": "review-demo",
            "event_type": "screening_decision_recorded",
            "occurred_at": "2026-08-08T00:03:00Z",
            "actor": {"actor_id": "reviewer-1", "actor_type": "human", "provenance": None},
            "payload": {"record_id": "record-1", "stage": "title_abstract", "decision": "include", "final_authority": True},
            "previous_hash": h3,
            "event_hash": h4,
        },
    ]
    first = reduce_events(events, h4)
    second = reduce_events(events, h4)
    errors: list[str] = []
    if canonical_bytes(first) != canonical_bytes(second):
        errors.append("reduction was not deterministic")
    if first["screening"]["rejected_final_authority_event_ids"] != ["e3"]:
        errors.append("non-human final authority was not rejected")
    decisions = first["screening"]["final_decisions"]
    if len(decisions) != 1 or decisions[0].get("decision") != "include":
        errors.append("human final decision was not retained")
    try:
        reduce_events(events, "f" * 64)
        errors.append("mismatched verified head was accepted")
    except ReductionError:
        pass
    registry, maximum_payload_bytes, prohibited_payload_keys = load_event_registry()
    registry_example = json.loads(
        (ROOT / "contracts/examples/audit-event-registry.json").read_text(encoding="utf-8")
    )
    registry_source = json.loads(EVENT_REGISTRY.read_text(encoding="utf-8"))
    if registry_source != registry_example:
        errors.append("catalogued event registry example drifted from runtime registry")
    migration_input = json.loads(
        (ROOT / "contracts/events/fixtures/search-run-completed-v0.json").read_text(
            encoding="utf-8"
        )
    )
    migration_expected = json.loads(
        (ROOT / "contracts/events/fixtures/search-run-completed-v1.json").read_text(
            encoding="utf-8"
        )
    )
    migration_actual = normalize_payload(
        "search_run_completed",
        migration_input,
        registry,
        maximum_payload_bytes,
        prohibited_payload_keys,
    )
    if migration_actual != migration_expected:
        errors.append("event payload migration fixture did not match")
    if migration_input.get("provider") != "pubmed":
        errors.append("event payload migration mutated the original payload")
    unknown = copy.deepcopy(events[0])
    unknown["event_type"] = "unregistered_event"
    try:
        reduce_events([unknown], h1)
        errors.append("unregistered event type was accepted")
    except ReductionError:
        pass
    prohibited = copy.deepcopy(events[0])
    prohibited["payload"]["token"] = "must-not-persist"
    try:
        reduce_events([prohibited], h1)
        errors.append("prohibited payload key was accepted")
    except ReductionError:
        pass
    wrong_authority_type = copy.deepcopy(events[2])
    wrong_authority_type["payload"]["final_authority"] = "human"
    wrong_authority_type["previous_hash"] = GENESIS
    try:
        reduce_events([wrong_authority_type], h3)
        errors.append("string final_authority was accepted")
    except ReductionError:
        pass
    return {
        "schema_version": "org.searchright.review-state-reducer-self-test.v1",
        "status": "failed" if errors else "passed",
        "tests": [
            "deterministic_reduction",
            "human_final_authority",
            "verified_head_binding",
            "event_payload_migration_fixture",
            "unknown_event_type_rejection",
            "event_registry_catalogue_parity",
            "prohibited_payload_key_rejection",
            "typed_final_authority_rejection",
        ],
        "errors": errors,
        "example": first,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("events", nargs="?", type=Path)
    parser.add_argument("--verified-head")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        receipt = self_test()
        print(json.dumps(receipt, indent=2, sort_keys=True))
        return 1 if receipt["errors"] else 0
    if args.events is None or not args.verified_head:
        parser.error("events and --verified-head are required unless --self-test is used")
    try:
        state = reduce_events(load_events(args.events), args.verified_head)
    except ReductionError as exc:
        print(json.dumps({"status": "failed", "error": str(exc)}, indent=2))
        return 1
    rendered = json.dumps(state, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
