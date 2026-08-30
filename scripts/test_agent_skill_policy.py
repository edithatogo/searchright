"""Static admission policy tests; not downstream runtime conformance."""

import copy
import unittest

import check_agent_skill as checker


class SiblingPolicyTests(unittest.TestCase):
    def setUp(self):
        self.metadata = {
            "status": "prepared_not_applied",
            "producer": "edithatogo/searchright",
            "consumer": "Imbad0202/academic-research-skills",
            "deployment": "searchright_owned_sibling",
            "routing": "explicit_user_handoff",
            "automated_invocation": "disabled_pending_runtime_admission",
        }

    def test_exact_sibling_declaration_passes(self):
        self.assertEqual(checker.validate_caller_policy(self.metadata), [])

    def test_each_policy_field_is_required_and_exact(self):
        for key in self.metadata:
            for value in (None, "wrong", True, [], {}):
                with self.subTest(key=key, value=value):
                    altered = copy.deepcopy(self.metadata)
                    altered[key] = value
                    self.assertTrue(checker.validate_caller_policy(altered))
            altered = copy.deepcopy(self.metadata)
            del altered[key]
            self.assertTrue(checker.validate_caller_policy(altered))

    def test_in_tree_routing_and_fork_identity_are_rejected(self):
        for key, value in (
            ("deployment", "ars_top_level_skill"),
            ("routing", "automatic_systematic_review_trigger"),
            ("consumer", "edithatogo/academic-research-skills"),
            ("automated_invocation", "enabled"),
            ("status", "adopted"),
        ):
            with self.subTest(key=key):
                altered = dict(self.metadata, **{key: value})
                self.assertTrue(checker.validate_caller_policy(altered))

    def test_malformed_metadata_is_rejected(self):
        for value in (None, [], "sibling", True):
            with self.subTest(value=value):
                self.assertTrue(checker.validate_caller_policy(value))


if __name__ == "__main__":
    unittest.main()
