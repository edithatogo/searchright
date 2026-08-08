#!/usr/bin/env python3
"""Static, network-free validation for the Searchright source repository.

This validator deliberately distinguishes checks that can be evidenced without a
Rust toolchain from checks that require Cargo, network access, credentials or an
external review. It aggregates failures so one run produces a useful repair list.
"""

from __future__ import annotations

import json
import re
import sys
import tomllib
from collections.abc import Iterable, Mapping
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:  # pragma: no cover - environment diagnostic
    yaml = None

try:
    from jsonschema import Draft202012Validator, FormatChecker
    from referencing import Registry, Resource
except ImportError:  # pragma: no cover - environment diagnostic
    Draft202012Validator = None
    FormatChecker = None
    Registry = None
    Resource = None

ROOT = Path(__file__).resolve().parents[1]
ERRORS: list[str] = []
CHECKS: dict[str, int] = {}

TEXT_SUFFIXES = {
    "",
    ".cff",
    ".css",
    ".html",
    ".in",
    ".json",
    ".json5",
    ".jsonc",
    ".md",
    ".ps1",
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".txt",
    ".wit",
    ".yaml",
    ".yml",
}
SKIP_PARTS = {".git", "target", "__pycache__", ".pytest_cache", ".mypy_cache"}
SCHEMA_CATALOG = ROOT / "contracts" / "schema-catalog.json"
EXPECTED_SCHEMA_VERSIONS = {
    "review-plan.yaml": "org.searchright.review-plan.v1",
    "search-strategy.yaml": "org.searchright.search-strategy.v1",
    "audit-event.json": "org.searchright.audit-event.v1",
    "prisma-flow.json": "org.searchright.prisma-flow.v1",
}
HEX_64 = re.compile(r"^[a-f0-9]{64}$")
ACTION_REF = re.compile(r"^([^@\s]+)@([0-9a-f]{40})$")
REQUIREMENT_PIN = re.compile(r"^[A-Za-z0-9_.-]+(?:\[[A-Za-z0-9_,.-]+\])?==[^\s;]+$")


def error(message: str) -> None:
    """Record one validation failure."""
    ERRORS.append(message)


def passed(name: str, count: int = 1) -> None:
    """Record completed static checks for the final receipt."""
    CHECKS[name] = CHECKS.get(name, 0) + count


def relative(path: Path) -> str:
    """Return a stable repository-relative path."""
    return path.relative_to(ROOT).as_posix()


def repository_files() -> Iterable[Path]:
    """Yield files in deterministic order, excluding generated/runtime trees."""
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or any(part in SKIP_PARTS for part in path.parts):
            continue
        yield path


def read_text(path: Path) -> str:
    """Read UTF-8 text and aggregate decode failures."""
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as exc:
        error(f"non-UTF-8 text file {relative(path)}: {exc}")
        return ""


def load_json(path: Path) -> Any:
    """Parse JSON while aggregating failures."""
    try:
        value = json.loads(read_text(path))
        passed("json_documents_parsed")
        return value
    except Exception as exc:  # noqa: BLE001 - aggregate diagnostics by design
        error(f"invalid JSON {relative(path)}: {exc}")
        return None


def load_yaml(path: Path) -> Any:
    """Parse YAML while aggregating failures."""
    if yaml is None:
        error("PyYAML is unavailable; YAML documents were not parsed")
        return None
    try:
        value = yaml.safe_load(read_text(path))
        passed("yaml_documents_parsed")
        return value
    except Exception as exc:  # noqa: BLE001 - aggregate diagnostics by design
        error(f"invalid YAML {relative(path)}: {exc}")
        return None


def load_toml(path: Path) -> Any:
    """Parse TOML while aggregating failures."""
    try:
        value = tomllib.loads(read_text(path))
        passed("toml_documents_parsed")
        return value
    except Exception as exc:  # noqa: BLE001 - aggregate diagnostics by design
        error(f"invalid TOML {relative(path)}: {exc}")
        return None


def load_data(path: Path) -> Any:
    """Load a JSON or YAML example."""
    if path.suffix == ".json":
        return load_json(path)
    if path.suffix in {".yaml", ".yml"}:
        return load_yaml(path)
    error(f"unsupported data format: {relative(path)}")
    return None


def validate_serialised_documents() -> None:
    """Parse every JSON, TOML and YAML document in the repository."""
    for path in repository_files():
        if path.suffix == ".json":
            load_json(path)
        elif path.suffix == ".toml":
            load_toml(path)
        elif path.suffix in {".yaml", ".yml"}:
            load_yaml(path)


def format_json_path(parts: Iterable[Any]) -> str:
    """Format a JSON Schema instance path."""
    rendered = "$"
    for part in parts:
        if isinstance(part, int):
            rendered += f"[{part}]"
        else:
            rendered += f".{part}"
    return rendered


def validate_schemas_and_examples() -> dict[str, Any]:
    """Check the machine-readable schema catalogue and all canonical examples."""
    if any(value is None for value in (Draft202012Validator, FormatChecker, Registry, Resource)):
        error("jsonschema/referencing is unavailable; contracts were not validated")
        return {}

    catalog = load_json(SCHEMA_CATALOG)
    entries = catalog.get("entries") if isinstance(catalog, Mapping) else None
    if not isinstance(entries, list) or not entries:
        error("contracts/schema-catalog.json must contain a non-empty entries list")
        return {}

    schema_dir = ROOT / "contracts" / "json-schema"
    example_dir = ROOT / "contracts" / "examples"
    actual_schemas = {relative(path) for path in schema_dir.glob("*.schema.json")}
    actual_examples = {relative(path) for path in example_dir.iterdir() if path.is_file()}
    declared_schemas: set[str] = set()
    declared_examples: set[str] = set()
    entry_ids: set[str] = set()
    schema_id_values: set[str] = set()
    catalog_entries: list[tuple[Mapping[str, Any], Path, Path]] = []

    for index, entry in enumerate(entries):
        if not isinstance(entry, Mapping):
            error(f"schema catalogue entry {index} must be an object")
            continue
        identifier = entry.get("id")
        schema_value = entry.get("schema")
        example_value = entry.get("example")
        schema_id = entry.get("schema_id")
        if not all(nonempty_text(value) for value in (identifier, schema_value, example_value, schema_id)):
            error(f"schema catalogue entry {index} has missing identifiers or paths")
            continue
        if str(identifier) in entry_ids:
            error(f"duplicate schema catalogue id: {identifier}")
        entry_ids.add(str(identifier))
        if str(schema_id) in schema_id_values:
            error(f"duplicate schema catalogue schema_id: {schema_id}")
        schema_id_values.add(str(schema_id))
        schema_path = ROOT / str(schema_value)
        example_path = ROOT / str(example_value)
        declared_schemas.add(relative(schema_path))
        declared_examples.add(relative(example_path))
        if not schema_path.is_file():
            error(f"schema catalogue points to missing schema: {relative(schema_path)}")
            continue
        if not example_path.is_file():
            error(f"schema catalogue points to missing example: {relative(example_path)}")
            continue
        catalog_entries.append((entry, schema_path, example_path))

    if actual_schemas != declared_schemas:
        error(
            "schema catalogue mismatch: "
            f"undeclared={sorted(actual_schemas - declared_schemas)}, "
            f"missing={sorted(declared_schemas - actual_schemas)}"
        )
    if actual_examples != declared_examples:
        error(
            "schema-example catalogue mismatch: "
            f"undeclared={sorted(actual_examples - declared_examples)}, "
            f"missing={sorted(declared_examples - actual_examples)}"
        )

    schemas: dict[str, Any] = {}
    registry = Registry()
    for entry, path, _ in catalog_entries:
        schema = load_json(path)
        if not isinstance(schema, Mapping):
            continue
        if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
            error(f"schema does not declare Draft 2020-12: {relative(path)}")
        schema_id = schema.get("$id")
        if schema_id != entry.get("schema_id"):
            error(f"schema $id disagrees with catalogue: {relative(path)}")
            continue
        if not isinstance(schema_id, str) or not schema_id.startswith("https://schemas.searchright.dev/"):
            error(f"schema has invalid or missing $id: {relative(path)}")
            continue
        try:
            Draft202012Validator.check_schema(schema)
            registry = registry.with_resource(schema_id, Resource.from_contents(schema))
            schemas[relative(path)] = schema
            passed("draft_2020_12_schemas_checked")
        except Exception as exc:  # noqa: BLE001
            error(f"invalid Draft 2020-12 schema {relative(path)}: {exc}")

    examples: dict[str, Any] = {}
    for _entry, schema_path, example_path in catalog_entries:
        schema = schemas.get(relative(schema_path))
        example = load_data(example_path)
        examples[example_path.name] = example
        if schema is None or example is None:
            continue
        try:
            validator = Draft202012Validator(
                schema,
                registry=registry,
                format_checker=FormatChecker(),
            )
            failures = sorted(
                validator.iter_errors(example),
                key=lambda item: tuple(str(part) for part in item.absolute_path),
            )
            for failure in failures:
                error(
                    f"schema example mismatch {relative(example_path)} "
                    f"at {format_json_path(failure.absolute_path)}: {failure.message}"
                )
            if not failures:
                passed("schema_examples_validated")
        except Exception as exc:  # noqa: BLE001
            error(f"could not validate {relative(example_path)}: {exc}")
    passed("schema_catalogue_entries_checked", len(catalog_entries))
    return examples

def nonempty_text(value: Any) -> bool:
    """Return whether a value is non-empty text."""
    return isinstance(value, str) and bool(value.strip())


def unique_nonempty_ids(values: Any, field: str) -> set[str]:
    """Validate a list of mappings with non-empty, unique `id` fields."""
    if not isinstance(values, list):
        error(f"{field} must be a list")
        return set()
    seen: set[str] = set()
    for index, item in enumerate(values):
        if not isinstance(item, Mapping) or not nonempty_text(item.get("id")):
            error(f"{field}[{index}].id must be non-empty")
            continue
        identifier = str(item["id"])
        if identifier in seen:
            error(f"duplicate identifier `{identifier}` in {field}")
        seen.add(identifier)
    return seen


def validate_query_expression(node: Any, location: str = "query") -> None:
    """Apply semantic checks to the portable query AST."""
    if not isinstance(node, Mapping):
        error(f"{location} must be an object")
        return
    operation = node.get("op")
    if operation == "term":
        term = node.get("term")
        if not isinstance(term, Mapping) or not nonempty_text(term.get("text")):
            error(f"{location}.term.text must be non-empty")
    elif operation in {"and", "or"}:
        children = node.get("children")
        if not isinstance(children, list) or len(children) < 2:
            error(f"{location}.children must contain at least two expressions")
            return
        for index, child in enumerate(children):
            validate_query_expression(child, f"{location}.children[{index}]")
    elif operation == "not":
        validate_query_expression(node.get("include"), f"{location}.include")
        validate_query_expression(node.get("exclude"), f"{location}.exclude")
    elif operation == "proximity":
        distance = node.get("distance")
        if not isinstance(distance, int) or isinstance(distance, bool) or distance < 1:
            error(f"{location}.distance must be a positive integer")
        validate_query_expression(node.get("left"), f"{location}.left")
        validate_query_expression(node.get("right"), f"{location}.right")
    else:
        error(f"{location}.op is unsupported: {operation!r}")


def validate_review_plan(plan: Any) -> tuple[set[str], set[str]]:
    """Check cross-field invariants in the canonical review plan example."""
    if not isinstance(plan, Mapping):
        error("review-plan example must be an object")
        return set(), set()
    if plan.get("schema_version") != EXPECTED_SCHEMA_VERSIONS["review-plan.yaml"]:
        error("review-plan example has the wrong schema_version")
    objectives = plan.get("objectives")
    if not isinstance(objectives, list) or not objectives or any(not nonempty_text(item) for item in objectives):
        error("review-plan objectives must contain non-empty values")
    framework = ((plan.get("question") or {}).get("framework") if isinstance(plan.get("question"), Mapping) else None)
    elements = framework.get("elements") if isinstance(framework, Mapping) else None
    if not isinstance(elements, Mapping) or not elements or any(
        not nonempty_text(key) or not nonempty_text(value) for key, value in elements.items()
    ):
        error("review-plan question framework must contain named, non-empty elements")
    eligibility = plan.get("eligibility")
    criterion_ids: set[str] = set()
    if isinstance(eligibility, Mapping):
        include = eligibility.get("include")
        exclude = eligibility.get("exclude")
        include_ids = unique_nonempty_ids(include, "eligibility.include")
        exclude_ids = unique_nonempty_ids(exclude, "eligibility.exclude")
        overlap = include_ids & exclude_ids
        if overlap:
            error(f"eligibility identifiers overlap across include/exclude: {sorted(overlap)}")
        criterion_ids = include_ids | exclude_ids
    else:
        error("review-plan eligibility must be an object")
    source_ids = unique_nonempty_ids(plan.get("information_sources"), "information_sources")
    strategy_values = plan.get("strategy_ids")
    strategy_ids: set[str] = set()
    if not isinstance(strategy_values, list) or not strategy_values:
        error("review-plan strategy_ids must not be empty")
    else:
        for value in strategy_values:
            if not nonempty_text(value):
                error("review-plan strategy_ids contain an empty value")
            elif value in strategy_ids:
                error(f"duplicate strategy identifier `{value}`")
            else:
                strategy_ids.add(value)
    protocol = plan.get("protocol")
    if isinstance(protocol, Mapping):
        registry = protocol.get("registry")
        identifier = protocol.get("identifier")
        if (registry is None) != (identifier is None):
            error("protocol registry and identifier must be supplied together")
    else:
        error("review-plan protocol must be an object")
    if source_ids and strategy_ids and criterion_ids:
        passed("review_plan_semantic_invariants")
    return source_ids, strategy_ids


def validate_search_strategy(strategy: Any, source_ids: set[str], strategy_ids: set[str]) -> None:
    """Check strategy semantics and cross-document references."""
    if not isinstance(strategy, Mapping):
        error("search-strategy example must be an object")
        return
    if strategy.get("schema_version") != EXPECTED_SCHEMA_VERSIONS["search-strategy.yaml"]:
        error("search-strategy example has the wrong schema_version")
    if strategy.get("source_id") not in source_ids:
        error("search-strategy source_id is absent from the review plan")
    if strategy.get("strategy_id") not in strategy_ids:
        error("search-strategy strategy_id is absent from the review plan")
    validate_query_expression(strategy.get("query"))
    limits = strategy.get("limits")
    if not isinstance(limits, Mapping):
        error("search-strategy limits must be an object")
        return
    date_limit = limits.get("publication_date")
    if isinstance(date_limit, Mapping):
        start = date_limit.get("from_year")
        end = date_limit.get("to_year")
        if isinstance(start, int) and isinstance(end, int) and start > end:
            error("search-strategy publication-date start exceeds end")
    restricted = date_limit is not None or any(
        bool(limits.get(field)) for field in ("languages", "publication_types", "filters")
    )
    rationale = limits.get("rationale")
    if restricted and (not isinstance(rationale, list) or not rationale):
        error("search restrictions require an explicit rationale")
    passed("search_strategy_semantic_invariants")


def validate_provider_manifest(manifest: Any) -> None:
    """Check provider security semantics beyond JSON Schema."""
    if not isinstance(manifest, Mapping):
        error("provider-manifest example must be an object")
        return
    hosts = manifest.get("allowed_hosts")
    if not isinstance(hosts, list):
        error("provider allowed_hosts must be a list")
        return
    if len(hosts) != len(set(hosts)):
        error("provider allowed_hosts contains duplicates")
    for host in hosts:
        if not nonempty_text(host) or "://" in host or "/" in host or "@" in host:
            error(f"provider allowed host is not a bare hostname: {host!r}")
        if str(host).lower() in {"localhost", "localhost.localdomain"}:
            error("provider manifest must not allow localhost")
    passed("provider_manifest_semantic_invariants")


def validate_screening_decision(decision: Any) -> None:
    """Check exclusion and agent-provenance invariants."""
    if not isinstance(decision, Mapping):
        error("screening-decision example must be an object")
        return
    excluded = decision.get("decision") == "exclude"
    reason = decision.get("exclusion_reason")
    if excluded and not isinstance(reason, Mapping):
        error("excluded screening decision requires a structured exclusion reason")
    if not excluded and reason is not None:
        error("non-exclusion screening decision must not carry an exclusion reason")
    if decision.get("reviewer_kind") == "agent" and not nonempty_text(decision.get("agent_provenance")):
        error("agent screening decision requires agent provenance")
    if decision.get("round") == "full_text" and excluded and isinstance(reason, Mapping):
        if not nonempty_text(reason.get("criterion_id")):
            error("full-text exclusion requires a criterion identifier")
    passed("screening_semantic_invariants")


def validate_audit_event(event: Any) -> None:
    """Check hash-chain representation and contract version."""
    if not isinstance(event, Mapping):
        error("audit-event example must be an object")
        return
    if event.get("schema_version") != EXPECTED_SCHEMA_VERSIONS["audit-event.json"]:
        error("audit-event example has the wrong schema_version")
    previous = event.get("previous_hash")
    if previous != "GENESIS" and (not isinstance(previous, str) or HEX_64.fullmatch(previous) is None):
        error("audit previous_hash must be GENESIS or a lower-case 64-character hash")
    current = event.get("event_hash")
    if not isinstance(current, str) or HEX_64.fullmatch(current) is None:
        error("audit event_hash must be a lower-case 64-character hash")
    if previous == current:
        error("audit event_hash must not equal previous_hash")
    passed("audit_event_semantic_invariants")


def validate_prisma_flow(flow: Any) -> None:
    """Check PRISMA 2020 flow arithmetic and identifiers."""
    if not isinstance(flow, Mapping):
        error("prisma-flow example must be an object")
        return
    if flow.get("schema_version") != EXPECTED_SCHEMA_VERSIONS["prisma-flow.json"]:
        error("prisma-flow example has the wrong schema_version")
    numeric_fields = (
        "records_databases",
        "records_registers",
        "records_other",
        "duplicates_removed",
        "automation_removed",
        "other_removed",
        "records_screened",
        "records_excluded",
        "reports_sought",
        "reports_not_retrieved",
        "reports_assessed",
        "studies_included",
        "reports_included",
    )
    if any(not isinstance(flow.get(field), int) or isinstance(flow.get(field), bool) for field in numeric_fields):
        error("PRISMA arithmetic fields must be integers")
        return
    identified = flow["records_databases"] + flow["records_registers"] + flow["records_other"]
    removed = flow["duplicates_removed"] + flow["automation_removed"] + flow["other_removed"]
    expected_screened = identified - removed
    if expected_screened != flow["records_screened"]:
        error(f"PRISMA records_screened mismatch: expected {expected_screened}")
    expected_sought = flow["records_screened"] - flow["records_excluded"]
    if expected_sought != flow["reports_sought"]:
        error(f"PRISMA reports_sought mismatch: expected {expected_sought}")
    expected_assessed = flow["reports_sought"] - flow["reports_not_retrieved"]
    if expected_assessed != flow["reports_assessed"]:
        error(f"PRISMA reports_assessed mismatch: expected {expected_assessed}")
    exclusions = flow.get("full_text_exclusions")
    exclusion_total = 0
    reason_ids: set[str] = set()
    if not isinstance(exclusions, list):
        error("PRISMA full_text_exclusions must be a list")
    else:
        for index, reason in enumerate(exclusions):
            if not isinstance(reason, Mapping):
                error(f"PRISMA exclusion {index} must be an object")
                continue
            identifier = reason.get("reason_id")
            if identifier in reason_ids:
                error(f"duplicate PRISMA exclusion reason `{identifier}`")
            if isinstance(identifier, str):
                reason_ids.add(identifier)
            count = reason.get("count")
            if isinstance(count, int) and not isinstance(count, bool):
                exclusion_total += count
    if exclusion_total + flow["reports_included"] != flow["reports_assessed"]:
        error("PRISMA assessed reports do not reconcile with exclusions plus included reports")
    if flow["studies_included"] > flow["reports_included"]:
        error("PRISMA included studies cannot exceed included reports")
    passed("prisma_arithmetic_invariants")


def validate_example_semantics(examples: Mapping[str, Any]) -> None:
    """Apply cross-field and cross-document checks to canonical examples."""
    source_ids, strategy_ids = validate_review_plan(examples.get("review-plan.yaml"))
    validate_search_strategy(examples.get("search-strategy.yaml"), source_ids, strategy_ids)
    validate_query_expression(examples.get("query-ast.yaml"), "query-ast")
    validate_provider_manifest(examples.get("provider-manifest.yaml"))
    validate_screening_decision(examples.get("screening-decision.yaml"))
    validate_audit_event(examples.get("audit-event.json"))
    validate_prisma_flow(examples.get("prisma-flow.json"))


def validate_workspace() -> None:
    """Check workspace membership, package uniqueness and crate-root policy."""
    cargo = load_toml(ROOT / "Cargo.toml")
    if not isinstance(cargo, Mapping):
        return
    workspace = cargo.get("workspace")
    members = workspace.get("members") if isinstance(workspace, Mapping) else None
    if not isinstance(members, list):
        error("Cargo workspace members must be a list")
        return
    declared = {str(member) for member in members}
    actual = {
        path.parent.relative_to(ROOT).as_posix()
        for path in (ROOT / "crates").glob("*/Cargo.toml")
    }
    if declared != actual:
        error(
            "workspace member parity failure: "
            f"undeclared={sorted(actual - declared)}, missing={sorted(declared - actual)}"
        )
    names: set[str] = set()
    for member in sorted(declared):
        directory = ROOT / member
        manifest_path = directory / "Cargo.toml"
        if not manifest_path.is_file():
            error(f"workspace member missing Cargo.toml: {member}")
            continue
        manifest = load_toml(manifest_path)
        package = manifest.get("package") if isinstance(manifest, Mapping) else None
        name = package.get("name") if isinstance(package, Mapping) else None
        if not nonempty_text(name):
            error(f"workspace member has no package name: {member}")
        elif name in names:
            error(f"duplicate Cargo package name: {name}")
        else:
            names.add(str(name))
        lints = manifest.get("lints") if isinstance(manifest, Mapping) else None
        if not isinstance(lints, Mapping) or lints.get("workspace") is not True:
            error(f"workspace lints are not inherited by {member}")
        roots = [directory / "src" / "lib.rs", directory / "src" / "main.rs"]
        existing_roots = [path for path in roots if path.is_file()]
        if not existing_roots:
            error(f"workspace member has no Rust crate root: {member}")
        for root in existing_roots:
            if "#![forbid(unsafe_code)]" not in read_text(root):
                error(f"crate root does not forbid unsafe code: {relative(root)}")
    if len(names) == len(declared):
        passed("workspace_crates_checked", len(declared))


def validate_conductor() -> None:
    """Check complete ordered Conductor planning and evidence metadata."""
    conductor = ROOT / "conductor"
    required = (
        "product.md",
        "product-guidelines.md",
        "tech-stack.md",
        "workflow.md",
        "requirements.md",
        "design.md",
        "tracks.md",
        "roadmap-coverage.json",
        "maturity-dossier.json",
        "upstream.lock.json",
        "upstream-capabilities.md",
    )
    for filename in required:
        if not (conductor / filename).is_file():
            error(f"missing conductor/{filename}")

    coverage = load_json(conductor / "roadmap-coverage.json")
    coverage_entries = coverage.get("tracks") if isinstance(coverage, Mapping) else None
    if not isinstance(coverage_entries, list) or not coverage_entries:
        error("conductor/roadmap-coverage.json must contain tracks")
        return
    declared_ids = [str(entry.get("track_id")) for entry in coverage_entries if isinstance(entry, Mapping)]

    tracks_dir = conductor / "tracks"
    tracks = sorted(path for path in tracks_dir.glob("[0-9][0-9]-*") if path.is_dir())
    expected_ids = [f"{number:02d}" for number in range(len(tracks))]
    actual_ids = [path.name[:2] for path in tracks]
    if actual_ids != expected_ids:
        error(f"Conductor track IDs are not contiguous: {actual_ids}")
    if actual_ids != declared_ids:
        error(f"Conductor coverage order differs from track directories: {declared_ids}")

    allowed_statuses = {
        "source_implemented",
        "contracted",
        "scaffolded",
        "partially_implemented",
        "source_implemented_unverified",
        "integration_prepared",
        "release_prepared",
        "submission_prepared",
        "external_evidence_required",
    }
    allowed_evidence = {
        "contracted",
        "source_verified",
        "compiler_verified",
        "fixture_proven",
        "live_proven",
        "externally_validated",
        "published",
    }
    for track in tracks:
        for filename in ("spec.md", "plan.md", "metadata.json", "evidence.json"):
            if not (track / filename).is_file():
                error(f"missing {filename} in {relative(track)}")
        metadata = load_json(track / "metadata.json")
        evidence_record = load_json(track / "evidence.json")
        if not isinstance(metadata, Mapping) or not isinstance(evidence_record, Mapping):
            continue
        track_id = track.name[:2]
        if metadata.get("track_id") != track_id or evidence_record.get("track_id") != track_id:
            error(f"track identity mismatch in {relative(track)}")
        if metadata.get("slug") != track.name[3:]:
            error(f"track metadata slug mismatch in {relative(track)}")
        status = metadata.get("status")
        evidence = metadata.get("evidence_level")
        if status not in allowed_statuses:
            error(f"invalid Conductor status {status!r} in {relative(track)}")
        if evidence not in allowed_evidence:
            error(f"invalid evidence level {evidence!r} in {relative(track)}")
        if evidence_record.get("status") != status or evidence_record.get("evidence_level") != evidence:
            error(f"track evidence differs from metadata in {relative(track)}")
        plan = read_text(track / "plan.md")
        if not re.search(r"^- \[x\]", plan, flags=re.MULTILINE | re.IGNORECASE):
            error(f"track has no source-evidenced completed task: {relative(track / 'plan.md')}")
        blockers = evidence_record.get("blockers")
        if isinstance(blockers, list) and blockers and not re.search(r"^- \[ \]", plan, flags=re.MULTILINE):
            error(f"track blockers have no open plan task: {relative(track / 'plan.md')}")
        if "## Phase 4: Review and closeout" not in plan:
            error(f"track lacks review/closeout phase: {relative(track / 'plan.md')}")

    requirements = read_text(conductor / "requirements.md")
    priorities = set(re.findall(r"\|\s*(Must|Should|Could|Won[’']t now)\s*\|", requirements))
    required_priorities = {"Must", "Should", "Could"}
    if not required_priorities.issubset(priorities) or not any(item.startswith("Won") for item in priorities):
        error(f"MoSCoW matrix is incomplete: found {sorted(priorities)}")
    design = read_text(conductor / "design.md")
    mermaid_blocks = re.findall(r"```mermaid\s*\n(.*?)```", design, flags=re.DOTALL)
    if not mermaid_blocks or any(not block.strip() for block in mermaid_blocks):
        error("conductor/design.md must contain non-empty Mermaid diagrams")
    else:
        passed("mermaid_design_blocks_checked", len(mermaid_blocks))
    if len(tracks) == len(coverage_entries):
        passed("conductor_tracks_checked", len(tracks))


def validate_action_and_tool_pins() -> None:
    """Require immutable Action references and exact Cargo tool versions."""
    workflow_dir = ROOT / ".github" / "workflows"
    uses_count = 0
    for path in sorted(workflow_dir.glob("*.yml")):
        text = read_text(path)
        for line_number, line in enumerate(text.splitlines(), start=1):
            match = re.search(r"\buses:\s*([^\s#]+)", line)
            if match:
                reference = match.group(1).strip("'\"")
                if reference.startswith("./") or reference.startswith("docker://"):
                    continue
                uses_count += 1
                if ACTION_REF.fullmatch(reference) is None:
                    error(f"GitHub Action is not commit-pinned at {relative(path)}:{line_number}: {reference}")
        for line_number, line in enumerate(text.splitlines(), start=1):
            if "cargo install " not in line:
                continue
            command = line.split("cargo install ", 1)[1]
            local_path_install = " --path " in line
            if " --locked" not in line or (not local_path_install and " --version " not in line):
                error(f"Cargo-installed CI tool lacks exact version/lock at {relative(path)}:{line_number}")
            if not local_path_install:
                before_version = command.split(" --version ", 1)[0].strip()
                if " " in before_version:
                    error(f"install Cargo CI tools one at a time for independent pins at {relative(path)}:{line_number}")
    passed("github_action_commit_pins_checked", uses_count)


def validate_python_pins() -> None:
    """Require exact validation-environment pins in the lock-style requirements file."""
    path = ROOT / "requirements" / "validation.txt"
    if not path.is_file():
        error("requirements/validation.txt is missing")
        return
    entries = [
        line.strip()
        for line in read_text(path).splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    if not entries:
        error("requirements/validation.txt has no packages")
    for line in entries:
        if REQUIREMENT_PIN.fullmatch(line) is None:
            error(f"validation dependency is not exact-pinned: {line}")
    passed("python_validation_dependency_pins_checked", len(entries))


def validate_registry_truthfulness() -> None:
    """Check that prepared packets do not claim submission or acceptance."""
    status_path = ROOT / "registry" / "status.json"
    status = load_json(status_path)
    if not isinstance(status, Mapping):
        return
    if status.get("overall_status") != "prepared_not_submitted":
        error("registry/status.json must remain prepared_not_submitted until external evidence exists")
    forbidden_states = {"accepted", "published", "submitted", "listed", "verified"}
    targets = status.get("targets")
    if not isinstance(targets, list) or not targets:
        error("registry status must contain targets")
        return
    names: set[str] = set()
    for target in targets:
        if not isinstance(target, Mapping):
            error("registry target must be an object")
            continue
        name = target.get("target")
        if name in names:
            error(f"duplicate registry target: {name}")
        if isinstance(name, str):
            names.add(name)
        state = target.get("status")
        if state in forbidden_states:
            error(f"registry target claims external state without publication evidence: {name}={state}")
        if state == "prepared" and not target.get("blockers"):
            error(f"prepared registry target lacks explicit blockers: {name}")
    for expected in {"github", "crates.io", "official-mcp-registry", "glama", "smithery"}:
        if expected not in names:
            error(f"registry catalogue is missing {expected}")
    passed("registry_targets_truthfulness_checked", len(names))


def validate_status_and_receipts() -> None:
    """Prevent static scaffolding from masquerading as compiled/published evidence."""
    status_path = ROOT / "PROJECT_STATUS.md"
    status = read_text(status_path)
    lock_exists = (ROOT / "Cargo.lock").exists()
    if not lock_exists and "Cargo.lock" not in status:
        error("missing Cargo.lock is not disclosed in PROJECT_STATUS.md")
    required_disclosures = (
        "Rust compilation",
        "Live provider calls",
        "GitHub repository creation/push",
        "Conductor plugin installation",
    )
    for disclosure in required_disclosures:
        if disclosure not in status:
            error(f"PROJECT_STATUS.md is missing disclosure: {disclosure}")
    receipt_path = ROOT / "verification" / "receipts" / "generation-environment.json"
    receipt = load_json(receipt_path)
    if not isinstance(receipt, Mapping):
        return
    environment = receipt.get("environment")
    if isinstance(environment, Mapping):
        if environment.get("cargo_available") is not False:
            error("generation receipt must record cargo_available=false for this runtime")
        if environment.get("rust_toolchain_available") is not False:
            error("generation receipt must record rust_toolchain_available=false for this runtime")
    if receipt.get("rust_compilation_performed") is True:
        error("generation receipt falsely claims Rust compilation")
    executed = receipt.get("executed")
    if isinstance(executed, list):
        false_claims = [
            item for item in executed
            if isinstance(item, str) and re.search(r"\bcargo\s+(check|test|clippy|fmt|build|doc)\b", item, re.IGNORECASE)
        ]
        if false_claims:
            error(f"generation receipt contains unsupported Cargo claims: {false_claims}")
    expected_claim = (
        "source_verified_not_compiler_verified"
        if receipt.get("schema_version") == "org.searchright.generation-environment.v3"
        else "contracted_and_scaffolded_not_compiled"
    )
    if receipt.get("claim") != expected_claim:
        error(f"generation receipt claim must remain {expected_claim}")
    passed("status_and_receipt_truthfulness_checked")


def strip_rust_noncode(source: str) -> str:
    """Replace Rust comments/string/character literals with spaces.

    The scanner handles nested block comments, normal/byte strings, raw strings
    and ordinary character literals. Newlines are retained for diagnostics.
    """
    output = list(source)
    length = len(source)
    index = 0

    def blank(start: int, end: int) -> None:
        for position in range(start, min(end, length)):
            if output[position] != "\n":
                output[position] = " "

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index)
            if end == -1:
                end = length
            blank(index, end)
            index = end
            continue
        if source.startswith("/*", index):
            start = index
            depth = 1
            index += 2
            while index < length and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            blank(start, index)
            continue

        raw_prefix_length = 0
        if source.startswith("br", index):
            raw_prefix_length = 2
        elif source.startswith("r", index):
            raw_prefix_length = 1
        if raw_prefix_length:
            cursor = index + raw_prefix_length
            hashes = 0
            while cursor < length and source[cursor] == "#":
                hashes += 1
                cursor += 1
            if cursor < length and source[cursor] == '"':
                terminator = '"' + ("#" * hashes)
                end = source.find(terminator, cursor + 1)
                end = length if end == -1 else end + len(terminator)
                blank(index, end)
                index = end
                continue

        quote_index = index
        if source.startswith('b"', index):
            quote_index = index + 1
        if quote_index < length and source[quote_index] == '"':
            cursor = quote_index + 1
            escaped = False
            while cursor < length:
                character = source[cursor]
                if escaped:
                    escaped = False
                elif character == "\\":
                    escaped = True
                elif character == '"':
                    cursor += 1
                    break
                cursor += 1
            blank(index, cursor)
            index = cursor
            continue

        if source[index] == "'":
            # A lifetime (`'a`) has no closing quote and must remain code. A
            # character literal closes within a small escaped sequence.
            cursor = index + 1
            if cursor < length and source[cursor] == "\\":
                cursor += 2
            else:
                cursor += 1
            if cursor < length and source[cursor] == "'":
                cursor += 1
                blank(index, cursor)
                index = cursor
                continue

        index += 1
    return "".join(output)


def delimiter_error(source: str) -> str | None:
    """Return the first Rust delimiter mismatch, if any."""
    opening = {"(": ")", "[": "]", "{": "}"}
    closing = {value: key for key, value in opening.items()}
    stack: list[tuple[str, int]] = []
    line = 1
    for character in source:
        if character == "\n":
            line += 1
            continue
        if character in opening:
            stack.append((character, line))
        elif character in closing:
            if not stack or stack[-1][0] != closing[character]:
                return f"unexpected `{character}` at line {line}"
            stack.pop()
    if stack:
        character, opening_line = stack[-1]
        return f"unclosed `{character}` opened at line {opening_line}"
    return None


def validate_rust_sources() -> None:
    """Apply conservative lexical and policy checks without claiming compilation."""
    forbidden_fragments = (".unwrap(", ".expect(", "todo!", "unimplemented!", "dbg!")
    paths = sorted((ROOT / "crates").rglob("*.rs"))
    for path in paths:
        source = read_text(path)
        for fragment in forbidden_fragments:
            if fragment in source:
                error(f"forbidden Rust fragment `{fragment}` in {relative(path)}")
        stripped = strip_rust_noncode(source)
        mismatch = delimiter_error(stripped)
        if mismatch:
            error(f"Rust lexical delimiter failure in {relative(path)}: {mismatch}")
    passed("rust_sources_lexically_checked", len(paths))


def validate_text_hygiene() -> None:
    """Check placeholders, trailing whitespace and final newlines in text files."""
    forbidden_words = ("TO" + "DO", "FIX" + "ME", "T" + "BD", "X" + "XX")
    pattern = re.compile(r"\b(?:" + "|".join(re.escape(item) for item in forbidden_words) + r")\b")
    checked = 0
    for path in repository_files():
        if path.suffix not in TEXT_SUFFIXES and path.name not in {"Dockerfile", "Makefile", "NOTICE"}:
            continue
        text = read_text(path)
        checked += 1
        for line_number, line in enumerate(text.splitlines(), start=1):
            if line.rstrip(" \t") != line:
                error(f"trailing whitespace in {relative(path)}:{line_number}")
            if path != Path(__file__).resolve() and pattern.search(line):
                error(f"unresolved placeholder in {relative(path)}:{line_number}")
        if text and not text.endswith("\n"):
            error(f"text file lacks final newline: {relative(path)}")
    passed("text_files_hygiene_checked", checked)


def validate_publication_packets() -> None:
    """Check presence and conservative state of MCP/marketplace packets."""
    required = (
        "server.json",
        "glama.json",
        "registry/official-mcp/README.md",
        "registry/glama/README.md",
        "registry/smithery/README.md",
        "registry/smithery/mcpb-manifest.template.json",
        "registry/joss/paper.md",
    )
    for relative_path in required:
        if not (ROOT / relative_path).is_file():
            error(f"missing publication packet: {relative_path}")
    server = load_json(ROOT / "server.json")
    if isinstance(server, Mapping):
        if server.get("name") != "io.github.edithatogo/searchright":
            error("server.json MCP name mismatch")
        if not server.get("packages"):
            error("server.json has no package declaration")
    passed("publication_packets_checked", len(required))


def main() -> int:
    """Run every static gate and print a machine-readable summary."""
    validate_serialised_documents()
    examples = validate_schemas_and_examples()
    validate_example_semantics(examples)
    validate_workspace()
    validate_conductor()
    validate_action_and_tool_pins()
    validate_python_pins()
    validate_registry_truthfulness()
    validate_status_and_receipts()
    validate_rust_sources()
    validate_text_hygiene()
    validate_publication_packets()

    summary = {
        "status": "failed" if ERRORS else "passed",
        "repository": str(ROOT),
        "checks": dict(sorted(CHECKS.items())),
        "error_count": len(ERRORS),
        "limitations": [
            "No Rust parsing, compilation, tests, coverage, mutation or fuzzing were executed.",
            "No live provider, remote GitHub, Conductor host or registry operation was executed.",
        ],
    }
    print(json.dumps(summary, indent=2, sort_keys=True))
    if ERRORS:
        print("Static repository validation failed:", file=sys.stderr)
        for item in ERRORS:
            print(f"- {item}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
