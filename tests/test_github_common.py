from __future__ import annotations

import importlib.util
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "github_common", ROOT / "scripts" / "github_common.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class GitHubCommandEncodingTests(unittest.TestCase):
    def test_cli_output_is_decoded_as_utf8_on_windows(self) -> None:
        completed = subprocess.CompletedProcess(
            args=["gh", "api", "example"],
            returncode=0,
            stdout='{"title":"Māori"}',
            stderr="",
        )

        with patch.object(MODULE.subprocess, "run", return_value=completed) as run:
            result = MODULE.run_json(["gh", "api", "example"])

        self.assertEqual(result, {"title": "Māori"})
        self.assertEqual(run.call_args.kwargs["encoding"], "utf-8")
        self.assertEqual(run.call_args.kwargs["errors"], "replace")


if __name__ == "__main__":
    unittest.main()
