from __future__ import annotations

import importlib.util
import os
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "bootstrap_github", ROOT / "scripts" / "bootstrap_github.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ProjectSecretTests(unittest.TestCase):
    @patch.dict(os.environ, {}, clear=True)
    def test_verifies_existing_protected_environment_secret(self) -> None:
        with patch.object(
            MODULE,
            "run_json",
            return_value=[{"name": "SEARCHRIGHT_PROJECT_TOKEN"}],
        ) as run_json:
            status = MODULE.maybe_set_project_secret("edithatogo/searchright")

        self.assertEqual(status, "verified_in_protected_environment")
        self.assertIn("github-project-write", run_json.call_args.args[0])

    @patch.dict(
        os.environ,
        {"SEARCHRIGHT_PROJECT_TOKEN_VALUE": "test-token-not-a-real-secret"},
        clear=True,
    )
    def test_configures_secret_in_protected_environment(self) -> None:
        with patch.object(MODULE, "run") as run:
            status = MODULE.maybe_set_project_secret("edithatogo/searchright")

        self.assertEqual(status, "configured_in_protected_environment")
        self.assertIn("github-project-write", run.call_args.args[0])
        self.assertEqual(run.call_args.kwargs["input_text"], "test-token-not-a-real-secret")

    @patch.dict(os.environ, {}, clear=True)
    def test_missing_secret_remains_fail_closed(self) -> None:
        with patch.object(MODULE, "run_json", return_value=[]):
            status = MODULE.maybe_set_project_secret("edithatogo/searchright")

        self.assertEqual(status, "manual_secret_required")


if __name__ == "__main__":
    unittest.main()
