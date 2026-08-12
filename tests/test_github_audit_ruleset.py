from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "audit_github_control_plane", ROOT / "scripts" / "audit_github_control_plane.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def settings() -> dict:
    return {
        "repository": "edithatogo/searchright",
        "visibility": "public",
        "description": "description",
        "homepage": "https://example.test",
        "features": {"issues": True, "projects": True, "discussions": True, "wiki": False},
        "merge_policy": {
            "squash": True,
            "rebase": True,
            "merge_commit": False,
            "delete_head_branch": True,
            "allow_auto_merge": True,
            "allow_update_branch": True,
        },
        "topics": ["rust"],
        "environments": ["github-project-write"],
        "ruleset": {
            "name": "main-protection",
            "target": "branch",
            "enforcement": "active",
            "include": ["~DEFAULT_BRANCH"],
            "required_status_checks": ["Static", "PR scope"],
            "required_linear_history": True,
            "required_signed_commits": False,
            "deletion": False,
            "non_fast_forward": False,
        },
    }


class RulesetAuditTests(unittest.TestCase):
    def responses(self, drift: bool = False):
        declared = settings()
        expected = MODULE.ruleset_payload(declared)
        if drift:
            expected["rules"][-1]["parameters"]["required_status_checks"].pop()
        repository = {
            "full_name": declared["repository"],
            "visibility": "public",
            "description": "description",
            "homepage": "https://example.test",
            "has_issues": True,
            "has_projects": True,
            "has_discussions": True,
            "has_wiki": False,
            "allow_squash_merge": True,
            "allow_rebase_merge": True,
            "allow_merge_commit": False,
            "delete_branch_on_merge": True,
            "allow_auto_merge": True,
            "allow_update_branch": True,
            "html_url": "https://github.com/edithatogo/searchright",
            "default_branch": "main",
            "security_and_analysis": {},
        }
        return iter(
            [
                repository,
                {"names": ["rust"]},
                [{"environments": [{"name": "github-project-write"}]}],
                {"deployment_branch_policy": {"protected_branches": True}},
                [{"name": "main-protection", "id": 1}],
                dict(expected, id=1),
            ]
        )

    def test_full_ruleset_payload_matches(self) -> None:
        responses = self.responses()
        with patch.object(MODULE, "run_json", side_effect=lambda *_args, **_kwargs: next(responses)):
            errors: list[str] = []
            MODULE.compare_repository(settings(), errors, [])
        self.assertEqual(errors, [])

    def test_missing_required_check_is_drift(self) -> None:
        responses = self.responses(drift=True)
        with patch.object(MODULE, "run_json", side_effect=lambda *_args, **_kwargs: next(responses)):
            errors: list[str] = []
            MODULE.compare_repository(settings(), errors, [])
        self.assertIn("main-protection ruleset differs from the declared full payload", errors)


if __name__ == "__main__":
    unittest.main()
