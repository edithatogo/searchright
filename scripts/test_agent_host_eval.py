"""Network-free regression tests for evaluation input/scorer separation."""
import copy
import json
import unittest
from unittest.mock import patch
from subprocess import CompletedProcess
from pathlib import Path

import run_agent_host_eval as runner


class EvaluationIntegrityTests(unittest.TestCase):
    def setUp(self):
        self.suite = runner.load(runner.SCENARIOS)

    def test_prompt_does_not_expose_expected_answers_or_descriptive_ids(self):
        text = runner.prompt(self.suite)
        inputs = json.loads(text.split("SCENARIOS\n", 1)[1])
        self.assertEqual(len(inputs), len(self.suite["cases"]))
        for index, case in enumerate(inputs):
            self.assertEqual(case["id"], f"case-{index + 1:03d}")
            self.assertNotIn("expected", case)
            self.assertLessEqual(set(case), {"id", "request", "authority_record"})

    def test_answer_and_metadata_changes_cannot_change_prompt(self):
        changed = copy.deepcopy(self.suite)
        for case in changed["cases"]:
            case["expected"] = {"allowed": "SECRET_LABEL", "reason": "SECRET_LABEL"}
            case["id"] = "SECRET_LABEL"
            case["future_metadata"] = "SECRET_LABEL"
        self.assertEqual(runner.prompt(self.suite), runner.prompt(changed))

    def test_scorer_accepts_exact_answers_using_opaque_ids(self):
        decisions = [{"id": f"case-{index + 1:03d}", **case["expected"]}
                     for index, case in enumerate(self.suite["cases"])]
        rows, errors = runner.evaluate({"decisions": decisions}, self.suite)
        self.assertEqual(errors, [])
        self.assertTrue(all(row["match"] for row in rows))
        decisions[0]["allowed"] = 1
        self.assertTrue(runner.evaluate({"decisions": decisions}, self.suite)[1])

    def test_reason_contract_is_global_and_generic_authority_stays_denied(self):
        text = runner.prompt(self.suite)
        self.assertIn("regardless of principal or approval receipt", text)
        schema = runner.output_schema(["case-001"])
        reasons = schema["properties"]["decisions"]["items"]["properties"]["reason"]["enum"]
        self.assertEqual(set(reasons), set(runner.REASON_CODES))
        for reason in reasons:
            self.assertIn(reason, text)

    def test_host_version_must_be_observed_not_copied(self):
        with patch.object(runner.subprocess, "run", return_value=CompletedProcess([], 0, "codex-cli 0.144.1\n", "")):
            self.assertEqual(runner.verify_host_version("codex-cli", "0.144.1"), "0.144.1")
            with self.assertRaises(ValueError):
                runner.verify_host_version("codex-cli", "0.999.0")

    def test_host_error_and_tool_events_fail_closed(self):
        for item_type in ("error", "command_execution", "mcp_tool_call", "unknown"):
            with self.assertRaises(ValueError):
                runner.check_codex_events(json.dumps({"type": "item.completed", "item": {"type": item_type}}))
        self.assertEqual(runner.check_codex_events(json.dumps({"type": "item.completed", "item": {"type": "agent_message"}})), ["agent_message"])

    def test_receipts_cannot_escape_or_overwrite_history(self):
        with self.assertRaises(ValueError):
            runner.receipt_path("../outside.json")
        with self.assertRaises(ValueError):
            runner.receipt_path("verification/receipts/track-11-host-model-codex-cli-gpt-5.6-sol.json")

    def test_codex_adapter_isolates_execution_and_inspects_events(self):
        def fake_run(command, **kwargs):
            self.assertNotEqual(kwargs["cwd"], runner.ROOT)
            self.assertIn("shell_tool", command)
            self.assertIn("unified_exec", command)
            self.assertIn("shell_snapshot", command)
            self.assertIn('web_search="disabled"', command)
            self.assertIn("--json", command)
            destination = Path(command[command.index("--output-last-message") + 1])
            destination.write_text('{"decisions":[]}', encoding="utf-8")
            return CompletedProcess(command, 0, json.dumps({"type": "item.completed", "item": {"type": "agent_message"}}), "")
        with patch.object(runner.subprocess, "run", side_effect=fake_run):
            result, evidence = runner.run_codex("test-model", "synthetic", runner.output_schema([]))
        self.assertEqual(result, {"decisions": []})
        self.assertEqual(evidence["event_integrity"], "passed")

    def test_malformed_decision_and_unknown_top_level_event_fail_closed(self):
        self.assertTrue(runner.evaluate({"decisions": [{"id": []}]}, self.suite)[1])
        with self.assertRaises(ValueError):
            runner.check_codex_events('{"type":"unknown"}')


if __name__ == "__main__":
    unittest.main()
