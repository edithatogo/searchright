#!/usr/bin/env python3
"""Run a bounded stdio MCP discovery transcript against a built server."""

from __future__ import annotations

import argparse
import json
import queue
import subprocess
import threading
import time
from pathlib import Path
from typing import TextIO

ROOT = Path(__file__).resolve().parents[1]
_END_OF_STREAM = object()


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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("command", nargs="+", help="MCP server command")
    args = parser.parse_args()

    expected = {
        entry["mcp_tool"]
        for entry in json.loads((ROOT / "contracts/interface-catalog.json").read_text())[
            "entries"
        ]
    }
    process = subprocess.Popen(  # noqa: S603 - explicit command supplied by CI/local operator
        args.command,
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
    errors: list[str] = []
    observed: set[str] = set()
    protocol_meta = {
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientInfo": {
            "name": "searchright-smoke",
            "version": "0.1.0",
        },
        "io.modelcontextprotocol/clientCapabilities": {},
    }

    def params(**values: object) -> dict[str, object]:
        return {**values, "_meta": protocol_meta}

    try:
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
            if "2026-07-28" not in versions:
                errors.append("server/discover did not advertise MCP 2026-07-28")
            if not isinstance(result, dict) or result.get("resultType") != "complete":
                errors.append("server/discover did not return a complete result")
            elif not isinstance(result.get("ttlMs"), int) or result.get("cacheScope") not in {
                "private",
                "public",
            }:
                errors.append("server/discover omitted MCP cache metadata")

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
            if not isinstance(result, dict) or result.get("resultType") != "complete":
                errors.append("tools/list did not return a complete result")
            elif not isinstance(result.get("ttlMs"), int) or result.get("cacheScope") not in {
                "private",
                "public",
            }:
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
                annotations = tool.get("annotations")
                expected_annotations = {
                    "readOnlyHint": True,
                    "destructiveHint": False,
                    "idempotentHint": True,
                    "openWorldHint": False,
                }
                if not isinstance(annotations, dict) or any(
                    annotations.get(key) != value
                    for key, value in expected_annotations.items()
                ):
                    errors.append(
                        f"tool {tool.get('name')} lacks governed read-only annotations"
                    )
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

        send(
            process,
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": params(name="workflow", arguments={}),
            },
        )
        call_response = receive(process, lines, 3)
        if "error" in call_response:
            errors.append(f"tools/call failed: {call_response['error']}")
        else:
            result = call_response.get("result", {})
            if not isinstance(result, dict) or result.get("resultType") != "complete":
                errors.append("tools/call did not return a complete MCP result")
            elif not isinstance(result.get("structuredContent"), dict):
                errors.append("tools/call did not return structuredContent")

    except (OSError, RuntimeError, TimeoutError, json.JSONDecodeError) as exc:
        errors.append(str(exc))
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)

    receipt = {
        "schema_version": "org.searchright.mcp-smoke-receipt.v1",
        "status": "failed" if errors else "passed",
        "protocol_version": "2026-07-28",
        "rust_sdk": "rmcp 3.1.2",
        "expected_tools": len(expected),
        "observed_tools": len(observed),
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
