#!/usr/bin/env python3
"""Run a bounded stdio MCP discovery transcript against a built server."""

from __future__ import annotations

import argparse
import json
import os
import queue
import subprocess
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, TextIO

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
_END_OF_STREAM = object()
CURRENT_PROTOCOL_VERSION = "2026-07-28"
PREVIOUS_PROTOCOL_VERSION = "2025-11-25"
CLIENT_ERAS = {
    CURRENT_PROTOCOL_VERSION: "current",
    PREVIOUS_PROTOCOL_VERSION: "previous",
}
REDACTED_TOOL_ERROR = (
    "operation_rejected: operation rejected by the shared Searchright facade"
)
LEAK_MARKER = "smoke-leak-marker-52743"

EFFECT_ANNOTATIONS = {
    "read_only": {
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
    "write_local_draft": {
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
    "write_local_review": {
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
    "network_and_local_write": {
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": True,
    },
    "local_write_preview": {
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
    "local_write": {
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
}


@dataclass(frozen=True)
class EraCapabilities:
    """Protocol-era capabilities that drive transcript assertions."""

    protocol_version: str
    client_era: str
    handshake: str
    result_metadata: bool
    cache_metadata: bool

    @property
    def assertion_families(self) -> list[str]:
        families = [
            f"{self.handshake}_handshake",
            "catalogue_parity",
            "deterministic_tool_order",
            "tool_schema_advertisements",
            "governed_tool_annotations",
            "structured_content_schema_validation",
            "governed_error_redaction",
            "protocol_error_shape",
        ]
        if self.result_metadata:
            families.append("result_type_metadata")
        if self.cache_metadata:
            families.append("cache_metadata")
        return families


ERA_CAPABILITIES = {
    CURRENT_PROTOCOL_VERSION: EraCapabilities(
        protocol_version=CURRENT_PROTOCOL_VERSION,
        client_era="current",
        handshake="server_discover",
        result_metadata=True,
        cache_metadata=True,
    ),
    PREVIOUS_PROTOCOL_VERSION: EraCapabilities(
        protocol_version=PREVIOUS_PROTOCOL_VERSION,
        client_era="previous",
        handshake="initialize",
        result_metadata=False,
        cache_metadata=False,
    ),
}


def send(process: subprocess.Popen[str], payload: dict[str, object]) -> None:
    if process.stdin is None:
        raise RuntimeError("MCP server stdin is unavailable")
    process.stdin.write(json.dumps(payload, separators=(",", ":")) + "\n")
    process.stdin.flush()


def start_reader(stream: TextIO) -> queue.Queue[str | object]:
    """Read newline-delimited protocol messages without blocking timeout logic."""
    lines: queue.Queue[str | object] = queue.Queue()

    def reader() -> None:
        try:
            for line in stream:
                lines.put(line)
        finally:
            lines.put(_END_OF_STREAM)

    threading.Thread(target=reader, name="searchright-mcp-smoke-reader", daemon=True).start()
    return lines


def receive(
    process: subprocess.Popen[str],
    lines: queue.Queue[str | object],
    request_id: int,
    timeout: float = 15.0,
) -> dict[str, object]:
    deadline = time.monotonic() + timeout
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f"timed out waiting for MCP response {request_id}")
        try:
            item = lines.get(timeout=remaining)
        except queue.Empty as exc:
            raise TimeoutError(f"timed out waiting for MCP response {request_id}") from exc
        if item is _END_OF_STREAM:
            raise RuntimeError(f"MCP server exited with {process.poll()}")
        if not isinstance(item, str):
            continue
        payload = json.loads(item)
        if not isinstance(payload, dict):
            continue
        if payload.get("id") == request_id:
            return payload


def resolve_server_command(command: list[str]) -> list[str]:
    """Resolve the server executable against the repository root."""
    executable = Path(command[0])
    candidate = executable if executable.is_absolute() else ROOT / executable
    tried = [candidate]
    if candidate.exists():
        return [str(candidate), *command[1:]]

    if os.name == "nt" and candidate.suffix.lower() != ".exe":
        exe_candidate = Path(f"{candidate}.exe")
        tried.append(exe_candidate)
        if exe_candidate.exists():
            return [str(exe_candidate), *command[1:]]

    tried_text = ", ".join(str(path) for path in tried)
    raise FileNotFoundError(
        "MCP server executable not found. "
        f"Tried: {tried_text}. Build the server or pass the correct executable path."
    )


def is_safely_callable(tool: dict[str, Any]) -> bool:
    """Return whether the advertised input schema has no required arguments."""
    schema = tool.get("inputSchema")
    if not isinstance(schema, dict):
        return False
    required = schema.get("required")
    return not isinstance(required, list) or len(required) == 0


def is_trivial_output_schema(schema: dict[str, Any]) -> bool:
    """Return whether the schema only constrains the root shape."""
    return "type" in schema and "properties" not in schema and "items" not in schema


def validate_structured_content(
    tool_name: str,
    result: object,
    output_schema: object,
    era: EraCapabilities,
    errors: list[str],
) -> bool:
    """Validate a tool result's structured content against its output schema."""
    if not isinstance(result, dict):
        errors.append(f"tools/call {tool_name} returned non-object result")
        return False
    if era.result_metadata and result.get("resultType") != "complete":
        errors.append(f"tools/call {tool_name} did not return resultType=complete")
        return False
    if result.get("isError") is not False:
        errors.append(f"tools/call {tool_name} did not return isError=false")
        return False
    if "structuredContent" not in result:
        errors.append(f"tools/call {tool_name} did not return structuredContent")
        return False
    if not isinstance(output_schema, dict):
        errors.append(f"tool {tool_name} lacks an object outputSchema for validation")
        return False

    structured_content = result["structuredContent"]
    try:
        validator = Draft202012Validator(output_schema)
        failures = sorted(
            validator.iter_errors(structured_content),
            key=lambda item: tuple(str(part) for part in item.absolute_path),
        )
    except Exception as exc:  # noqa: BLE001 - convert schema failures to receipt errors
        errors.append(f"tool {tool_name} outputSchema could not be applied: {exc}")
        return False
    for failure in failures:
        path = "$" + "".join(
            f"[{part}]" if isinstance(part, int) else f".{part}"
            for part in failure.absolute_path
        )
        errors.append(
            f"tool {tool_name} structuredContent failed outputSchema at {path}: {failure.message}"
        )
    return not failures


def text_blocks(result: dict[str, Any]) -> list[str]:
    """Extract text blocks from a tool result."""
    content = result.get("content")
    if not isinstance(content, list):
        return []
    blocks = []
    for block in content:
        if isinstance(block, dict) and isinstance(block.get("text"), str):
            blocks.append(block["text"])
    return blocks


def assert_protocol_error_shape(response: dict[str, object], errors: list[str]) -> bool:
    """Assert malformed input was reported as a JSON-RPC protocol error."""
    error = response.get("error")
    if not isinstance(error, dict):
        errors.append("malformed validate_plan did not return a JSON-RPC error")
        return False
    if "result" in response:
        errors.append("malformed validate_plan returned both result and JSON-RPC error")
        return False
    if not isinstance(error.get("message"), str):
        errors.append("malformed validate_plan JSON-RPC error lacks a message")
        return False
    return True


def assert_facade_error_shape(
    response: dict[str, object],
    era: EraCapabilities,
    errors: list[str],
) -> bool:
    """Assert facade rejection is represented as a redacted governed tool error."""
    if "error" in response:
        errors.append(f"facade rejection returned JSON-RPC error: {response['error']}")
        return False
    result = response.get("result")
    if not isinstance(result, dict):
        errors.append("facade rejection returned non-object result")
        return False
    if era.result_metadata and result.get("resultType") != "complete":
        errors.append("facade rejection did not return resultType=complete")
        return False
    if result.get("isError") is not True:
        errors.append("facade rejection did not return isError=true")
        return False
    messages = text_blocks(result)
    if messages != [REDACTED_TOOL_ERROR]:
        errors.append(f"facade rejection text was not the governed redaction: {messages}")
        return False
    transcript = json.dumps(response, sort_keys=True)
    leaked = [
        value
        for value in (LEAK_MARKER, "C:\\sensitive\\review-plan.json", "https://secret.example")
        if value in transcript
    ]
    if leaked:
        errors.append(f"facade rejection leaked user-controlled values: {leaked}")
        return False
    return True


def invalid_review_plan_document() -> str:
    """Return a deserialisable plan that the facade rejects semantically."""
    source = ROOT / "contracts" / "examples" / "review-plan.yaml"
    document = source.read_text(encoding="utf-8")
    document = document.replace(
        "review_id: demo-paediatric-metabolic-search",
        f"review_id: {LEAK_MARKER}",
    )
    document = document.replace(
        "title: Genomic testing in children with suspected inherited metabolic disease",
        "title: C:\\sensitive\\review-plan.json",
    )
    document = document.replace("strategy-medline-v1", LEAK_MARKER)
    document = document.replace("strategy-europe-pmc-v1", LEAK_MARKER)
    return document


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--receipt", type=Path)
    parser.add_argument(
        "--protocol-version",
        choices=sorted(CLIENT_ERAS),
        default=CURRENT_PROTOCOL_VERSION,
    )
    parser.add_argument("--strict-schemas", action="store_true")
    parser.add_argument("command", nargs="+", help="MCP server command")
    args = parser.parse_args()
    era = ERA_CAPABILITIES[args.protocol_version]

    expected = {
        entry["mcp_tool"]
        for entry in json.loads((ROOT / "contracts/interface-catalog.json").read_text())[
            "entries"
        ]
    }
    effect_by_tool = {name: "read_only" for name in expected}
    for entry in json.loads(
        (ROOT / "contracts/mcp/tool-catalog.json").read_text(encoding="utf-8")
    )["tools"]:
        effect_by_tool[entry["name"]] = entry["effect"]
    errors: list[str] = []
    observed: set[str] = set()
    callable_tools: dict[str, dict[str, Any]] = {}
    invoked_tools: list[str] = []
    schemas_validated = 0
    governed_errors_checked = 0
    trivial_output_schemas: set[str] = set()
    protocol_meta = {
        "io.modelcontextprotocol/protocolVersion": args.protocol_version,
        "io.modelcontextprotocol/clientInfo": {
            "name": "searchright-smoke",
            "version": "0.1.0",
        },
        "io.modelcontextprotocol/clientCapabilities": {},
    }

    def params(**values: object) -> dict[str, object]:
        return {**values, "_meta": protocol_meta}

    process: subprocess.Popen[str] | None = None
    try:
        command = resolve_server_command(args.command)
        process = subprocess.Popen(  # noqa: S603 - explicit command supplied by CI/local operator
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd=ROOT,
            bufsize=1,
        )
        if process.stdout is None:
            process.kill()
            raise RuntimeError("MCP server stdout is unavailable")
        lines = start_reader(process.stdout)

        if era.handshake == "server_discover":
            send(
                process,
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "server/discover",
                    "params": params(),
                },
            )
            discovery = receive(process, lines, 1)
            if "error" in discovery:
                errors.append(f"server/discover failed: {discovery['error']}")
            else:
                result = discovery.get("result", {})
                versions = result.get("supportedVersions", []) if isinstance(result, dict) else []
                if era.protocol_version not in versions:
                    errors.append(
                        "server/discover did not advertise MCP "
                        f"{era.protocol_version}"
                    )
                if not isinstance(result, dict):
                    errors.append("server/discover did not return an object result")
                else:
                    if era.result_metadata and result.get("resultType") != "complete":
                        errors.append("server/discover did not return resultType=complete")
                    if era.cache_metadata and (
                        not isinstance(result.get("ttlMs"), int)
                        or result.get("cacheScope") not in {"private", "public"}
                    ):
                        errors.append("server/discover omitted MCP cache metadata")
        else:
            send(
                process,
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": era.protocol_version,
                        "capabilities": {},
                        "clientInfo": {
                            "name": "searchright-smoke",
                            "version": "0.1.0",
                        },
                    },
                },
            )
            initialized = receive(process, lines, 1)
            if "error" in initialized:
                errors.append(f"initialize failed: {initialized['error']}")
            else:
                result = initialized.get("result", {})
                negotiated = result.get("protocolVersion") if isinstance(result, dict) else None
                if negotiated != era.protocol_version:
                    errors.append(
                        "initialize did not negotiate requested MCP "
                        f"{era.protocol_version}; got {negotiated!r}"
                    )
                send(
                    process,
                    {
                        "jsonrpc": "2.0",
                        "method": "notifications/initialized",
                        "params": params(),
                    },
                )

        send(
            process,
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": params()},
        )
        tools_response = receive(process, lines, 2)
        if "error" in tools_response:
            errors.append(f"tools/list failed: {tools_response['error']}")
        else:
            result = tools_response.get("result", {})
            tools = result.get("tools", []) if isinstance(result, dict) else []
            if not isinstance(result, dict):
                errors.append("tools/list did not return an object result")
            elif era.result_metadata and result.get("resultType") != "complete":
                errors.append("tools/list did not return resultType=complete")
            elif era.cache_metadata and (
                not isinstance(result.get("ttlMs"), int)
                or result.get("cacheScope") not in {"private", "public"}
            ):
                errors.append("tools/list omitted MCP cache metadata")
            names = [
                str(tool.get("name"))
                for tool in tools
                if isinstance(tool, dict) and isinstance(tool.get("name"), str)
            ]
            if names != sorted(names):
                errors.append("tools/list is not deterministically sorted by tool name")
            for tool in tools:
                if not isinstance(tool, dict):
                    errors.append("tools/list returned a non-object tool definition")
                    continue
                schema = tool.get("inputSchema")
                if not isinstance(schema, dict) or schema.get("type") != "object":
                    errors.append(f"tool {tool.get('name')} lacks an object inputSchema")
                output_schema = tool.get("outputSchema")
                if not isinstance(output_schema, dict) or output_schema.get("type") not in {
                    "array",
                    "object",
                }:
                    errors.append(f"tool {tool.get('name')} lacks a typed outputSchema")
                elif isinstance(tool.get("name"), str) and is_trivial_output_schema(output_schema):
                    trivial_output_schemas.add(tool["name"])
                annotations = tool.get("annotations")
                tool_name = tool.get("name")
                effect = effect_by_tool.get(tool_name)
                expected_annotations = EFFECT_ANNOTATIONS.get(effect)
                if not isinstance(annotations, dict) or any(
                    annotations.get(key) != value
                    for key, value in (expected_annotations or {}).items()
                ):
                    errors.append(
                        f"tool {tool_name} annotations do not match effect {effect!r}"
                    )
                if expected_annotations is None:
                    errors.append(f"tool {tool_name} has no governed effect mapping")
                if isinstance(tool.get("name"), str) and is_safely_callable(tool):
                    callable_tools[tool["name"]] = tool
            observed = {
                str(tool.get("name"))
                for tool in tools
                if isinstance(tool, dict) and isinstance(tool.get("name"), str)
            }
            missing = sorted(expected - observed)
            unexpected = sorted(observed - expected)
            if missing:
                errors.append(f"MCP tools missing from server: {missing}")
            if unexpected:
                errors.append(f"MCP server exposes tools absent from interface catalogue: {unexpected}")

        if args.strict_schemas and trivial_output_schemas:
            errors.append(
                "trivial output schemas are forbidden in strict mode: "
                f"{sorted(trivial_output_schemas)}"
            )

        request_id = 3
        for tool_name in sorted(callable_tools):
            tool = callable_tools[tool_name]
            send(
                process,
                {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "method": "tools/call",
                    "params": params(name=tool_name, arguments={}),
                },
            )
            call_response = receive(process, lines, request_id)
            invoked_tools.append(tool_name)
            if "error" in call_response:
                errors.append(f"tools/call {tool_name} failed: {call_response['error']}")
            else:
                if validate_structured_content(
                    tool_name,
                    call_response.get("result", {}),
                    tool.get("outputSchema"),
                    era,
                    errors,
                ):
                    schemas_validated += 1
            request_id += 1

        send(
            process,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "tools/call",
                "params": params(
                    name="validate_plan",
                    arguments={"document": "{", "format": "json"},
                ),
            },
        )
        malformed_response = receive(process, lines, request_id)
        if assert_protocol_error_shape(malformed_response, errors):
            governed_errors_checked += 1
        request_id += 1

        send(
            process,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "tools/call",
                "params": params(
                    name="validate_plan",
                    arguments={
                        "document": invalid_review_plan_document(),
                        "format": "yaml",
                    },
                ),
            },
        )
        facade_response = receive(process, lines, request_id)
        if assert_facade_error_shape(facade_response, era, errors):
            governed_errors_checked += 1

    except (OSError, RuntimeError, TimeoutError, json.JSONDecodeError) as exc:
        errors.append(str(exc))
    finally:
        if process is not None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)

    receipt = {
        "schema_version": "org.searchright.mcp-smoke-receipt.v1",
        "status": "failed" if errors else "passed",
        "protocol_version": era.protocol_version,
        "client_era": era.client_era,
        "rust_sdk": "rmcp 3.1.2",
        "expected_tools": len(expected),
        "observed_tools": len(observed),
        "tools_invoked": len(invoked_tools),
        "schemas_validated": schemas_validated,
        "governed_errors_checked": governed_errors_checked,
        "invoked_tool_names": invoked_tools,
        "trivial_output_schemas": sorted(trivial_output_schemas),
        "era_assertions": era.assertion_families,
        "limitations": [
            "Tools with required input arguments were not invoked by this smoke harness.",
        ],
        "errors": errors,
    }
    text = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    print(text, end="")
    if args.receipt:
        path = args.receipt if args.receipt.is_absolute() else ROOT / args.receipt
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
