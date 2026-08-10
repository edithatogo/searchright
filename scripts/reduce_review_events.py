#!/usr/bin/env python3
"""Deterministically reduce a pre-verified Searchright audit stream.

This reducer does not recompute BLAKE3 event hashes because it is deliberately
stdlib-only. Callers must supply the verified audit head produced by the Rust
ledger verifier; the reducer checks linkage, review identity, event identity,
and human-authority invariants before deriving a disposable snapshot.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any, Iterable

SCHEMA_VERSION = "org.searchright.review-state-snapshot.v1"
GENESIS = "GENESIS"
HEX64 = set("0123456789abcdef")


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
        payload = event.get("payload") if isinstance(event.get("payload"), dict) else {}
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
        else:
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
    return {
        "schema_version": "org.searchright.review-state-reducer-self-test.v1",
        "status": "failed" if errors else "passed",
        "tests": ["deterministic_reduction", "human_final_authority", "verified_head_binding"],
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
