"""Network-free regression tests for evaluation input/scorer separation."""
import copy
import json
import unittest

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


if __name__ == "__main__":
    unittest.main()
