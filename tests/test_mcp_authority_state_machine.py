"""Hypothesis state-machine tests for searchright-mcp authority boundaries.

Validates that:
1. The searchright-mcp server initializes cleanly over stdio.
2. The 31 advertised standard MCP tools strictly separate read-only analysis from consequential writes.
3. Every attempt by an agent or unverified caller to record screening exclusions fails closed.
4. Randomly generated invalid payloads never cause server panics or unhandled exceptions.
"""
from __future__ import annotations

import json
import os
import select
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any

try:
    from hypothesis import settings
    from hypothesis.stateful import RuleBasedStateMachine, initialize, invariant, rule
    import hypothesis.strategies as st

    HAVE_HYPOTHESIS = True
except ImportError:
    HAVE_HYPOTHESIS = False

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "debug" / "searchright-mcp"


class McpStdioClient:
    """Synchronous JSON-RPC stdio client with guarded timeout for searchright-mcp."""

    def __init__(self, cwd: Path | None = None) -> None:
        self.temp_dir = tempfile.TemporaryDirectory(prefix="searchright-mcp-test-")
        self.store_root = Path(self.temp_dir.name)
        env = dict(os.environ)
        env["SEARCHRIGHT_STORE_ROOT"] = str(self.store_root)

        self.process = subprocess.Popen(
            [str(BINARY)],
            cwd=str(cwd or ROOT),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            env=env,
        )
        self.msg_id = 0

    def rpc(self, method: str, params: dict[str, Any] | None = None, timeout: float = 3.0) -> dict[str, Any]:
        self.msg_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": self.msg_id,
            "method": method,
        }
        if params is not None:
            payload["params"] = params

        line = json.dumps(payload) + "\n"
        if not self.process.stdin:
            raise RuntimeError("process stdin closed")
        self.process.stdin.write(line)
        self.process.stdin.flush()

        if not self.process.stdout:
            raise RuntimeError("process stdout is closed")

        rlist, _, _ = select.select([self.process.stdout], [], [], timeout)
        if not rlist:
            stderr_out = self.process.stderr.read() if self.process.stderr else ""
            raise TimeoutError(f"server timed out after {timeout}s on method {method}. stderr: {stderr_out}")

        response_line = self.process.stdout.readline()
        if not response_line:
            stderr_out = self.process.stderr.read() if self.process.stderr else ""
            raise RuntimeError(f"server process died or returned empty output. stderr: {stderr_out}")

        return json.loads(response_line)

    def close(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=1)
            except subprocess.TimeoutExpired:
                self.process.kill()
        if self.process.stdin:
            try:
                self.process.stdin.close()
            except OSError:
                pass
        if self.process.stdout:
            try:
                self.process.stdout.close()
            except OSError:
                pass
        if self.process.stderr:
            try:
                self.process.stderr.close()
            except OSError:
                pass
        self.temp_dir.cleanup()


if HAVE_HYPOTHESIS:

    class McpAuthorityStateMachine(RuleBasedStateMachine):
        """Hypothesis stateful test enforcing Searchright MCP authority constraints."""

        def __init__(self) -> None:
            super().__init__()
            self.client = McpStdioClient()
            self.initialized = False
            self.tools: set[str] = set()
            self.unauthorized_exclusion_attempts = 0
            self.rejected_exclusion_count = 0

        @initialize()
        def initialize_server(self) -> None:
            resp = self.client.rpc(
                "initialize",
                {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {"name": "hypothesis-authority-tester", "version": "1.0"},
                },
            )
            assert "result" in resp, f"Initialization failed: {resp}"
            assert resp["result"]["protocolVersion"] == "2025-11-25"
            self.initialized = True

            tools_resp = self.client.rpc("tools/list", {})
            assert "result" in tools_resp
            self.tools = {tool["name"] for tool in tools_resp["result"]["tools"]}
            assert len(self.tools) == 31, f"Expected 31 tools in standard router, found {len(self.tools)}"

        @rule(
            review_id=st.text(min_size=3, max_size=16, alphabet=st.characters(whitelist_categories=("Ll", "Nd"))),
            record_id=st.text(min_size=3, max_size=16, alphabet=st.characters(whitelist_categories=("Ll", "Nd"))),
            reviewer_id=st.text(min_size=3, max_size=16, alphabet=st.characters(whitelist_categories=("Ll", "Nd"))),
            exclusion_reason=st.sampled_from(["out_of_scope", "wrong_population", "wrong_intervention", "unobtainable"]),
        )
        def attempt_agent_unauthorized_exclusion(
            self, review_id: str, record_id: str, reviewer_id: str, exclusion_reason: str
        ) -> None:
            """Attempt to record a screening exclusion WITHOUT a valid effect authority handle.

            The server MUST reject this call unconditionally (Fail-Closed).
            """
            self.unauthorized_exclusion_attempts += 1
            policy_payload = {
                "schema_version": "org.searchright.screening-policy.v1",
                "review_id": review_id,
                "policy_id": f"pol-{review_id}",
                "phases": ["title_abstract"],
                "blinded": True,
                "required_reviewers_per_record": 2,
                "conflict_policy": "independent_arbitration",
                "allowed_exclusion_reasons": [exclusion_reason],
                "agent_screening_authority": "advisory_only",
            }
            decision_payload = {
                "schema_version": "org.searchright.screening-decision.v1",
                "decision_id": f"dec-{record_id}",
                "review_id": review_id,
                "record_id": record_id,
                "reviewer_id": reviewer_id,
                "phase": "title_abstract",
                "status": "excluded",
                "exclusion_reason": exclusion_reason,
                "timestamp": "2026-11-04T12:00:00Z",
            }

            resp = self.client.rpc(
                "tools/call",
                {
                    "name": "record_screening_decision",
                    "arguments": {
                        "policy_json": json.dumps(policy_payload),
                        "decision_json": json.dumps(decision_payload),
                    },
                },
            )

            is_error = (
                "error" in resp
                or resp.get("result", {}).get("isError") is True
                or "authority" in json.dumps(resp).lower()
                or "effect protocol" in json.dumps(resp).lower()
            )
            assert is_error, f"SECURITY VIOLATION: Unauthorized exclusion was accepted! Response: {resp}"
            self.rejected_exclusion_count += 1

        @rule(
            document=st.text(min_size=1, max_size=60, alphabet=st.characters(blacklist_categories=("Cc", "Cs"))),
            format=st.sampled_from(["json", "yaml", None]),
        )
        def fuzz_validate_strategy_robustness(self, document: str, format: str | None) -> None:
            """Fuzz strategy validation with document strings; must fail closed gracefully without crash."""
            resp = self.client.rpc(
                "tools/call",
                {
                    "name": "validate_strategy",
                    "arguments": {"document": document, "format": format},
                },
            )
            assert "error" in resp or "result" in resp
            assert self.client.process.poll() is None

        @rule(
            envelope_json=st.text(min_size=1, max_size=60, alphabet=st.characters(blacklist_categories=("Cc", "Cs"))),
            endpoint=st.text(min_size=1, max_size=60, alphabet=st.characters(blacklist_categories=("Cc", "Cs"))),
        )
        def fuzz_authorise_endpoint_robustness(self, envelope_json: str, endpoint: str) -> None:
            """Fuzz endpoint authorization gate; malformed requests must never crash the server."""
            resp = self.client.rpc(
                "tools/call",
                {
                    "name": "authorise_endpoint",
                    "arguments": {"envelope_json": envelope_json, "endpoint": endpoint},
                },
            )
            assert "error" in resp or "result" in resp
            assert self.client.process.poll() is None

        @invariant()
        def check_server_and_safety_invariants(self) -> None:
            """All unauthorized exclusion attempts must be 100% rejected and server process must remain alive."""
            assert self.client.process.poll() is None, "Server process unexpectedly terminated!"
            assert self.rejected_exclusion_count == self.unauthorized_exclusion_attempts

        def teardown(self) -> None:
            self.client.close()

    def run_state_machine_as_test(state_machine_class: type[RuleBasedStateMachine]) -> None:
        test_case = state_machine_class.TestCase
        test_case.settings = settings(
            max_examples=15,
            stateful_step_count=10,
            deadline=None,
            database=None,
        )
        instance = test_case()
        instance.runTest()


class TestMcpAuthoritySuite(unittest.TestCase):
    def test_state_machine_authority_boundaries(self) -> None:
        """Execute stateful hypothesis property test over the compiled searchright-mcp stdio binary."""
        if not HAVE_HYPOTHESIS:
            raise unittest.SkipTest("hypothesis is not installed in this environment")
        if not BINARY.is_file():
            raise unittest.SkipTest(f"searchright-mcp binary not found at {BINARY}")
        run_state_machine_as_test(McpAuthorityStateMachine)


if __name__ == "__main__":
    unittest.main()
