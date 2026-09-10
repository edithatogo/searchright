"""Regression tests for script-backed and native gate-catalog commands."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_gate_catalog", ROOT / "scripts" / "check_gate_catalog.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class GateCatalogNativeCommandTests(unittest.TestCase):
    def test_repository_script_preserves_source_evidence_profile(self) -> None:
        command = "python scripts/check_agent_skill.py"
        path = MODULE.script_path(command)
        self.assertEqual(path, "scripts/check_agent_skill.py")
        kind = MODULE.command_kind(command, path)
        self.assertEqual(kind, "repository_script")
        profile = MODULE.capabilities(command, kind, path)
        self.assertFalse(profile["network"])
        self.assertFalse(profile["compiler_required"])
        self.assertEqual(profile["evidence_ceiling"], "source_verified")

    def test_cargo_command_is_catalogued_as_native_compiler_gate(self) -> None:
        command = (
            "cargo clippy -p searchright-cli --all-targets --locked -- -D warnings"
        )
        path = MODULE.script_path(command)
        self.assertIsNone(path)
        kind = MODULE.command_kind(command, path)
        self.assertEqual(kind, "rust_toolchain")
        profile = MODULE.capabilities(command, kind, path)
        self.assertTrue(profile["compiler_required"])
        self.assertTrue(profile["network"])
        self.assertEqual(profile["evidence_ceiling"], "compiler_verified")
        self.assertRegex(
            MODULE.gate_slug(command),
            r"^SR-GATE-CARGO-CLIPPY-[0-9A-F]{10}$",
        )

    def test_frozen_cargo_command_has_no_dependency_network_capability(self) -> None:
        command = "cargo test --frozen -p searchright-agent"
        kind = MODULE.command_kind(command, None)
        self.assertFalse(MODULE.capabilities(command, kind, None)["network"])

    def test_distinct_native_commands_have_distinct_gate_ids(self) -> None:
        test_id = MODULE.gate_slug("cargo test -p searchright-agent --locked")
        clippy_id = MODULE.gate_slug(
            "cargo clippy -p searchright-agent --locked -- -D warnings"
        )
        self.assertNotEqual(test_id, clippy_id)

    def test_compound_shell_commands_fail_closed(self) -> None:
        with self.assertRaisesRegex(ValueError, "compound shell"):
            MODULE.command_parts("cargo test && curl https://example.invalid")

    def test_current_catalog_renders_and_validates_native_commands(self) -> None:
        rendered = MODULE.render()
        self.assertEqual(rendered["schema_version"], "org.searchright.gate-catalog.v2")
        self.assertEqual(MODULE.validate(rendered), [])
        native = [
            gate
            for gate in rendered["gates"]
            if gate["command_kind"] != "repository_script"
        ]
        self.assertTrue(native)
        self.assertTrue(any(gate["compiler_required"] for gate in native))
        self.assertTrue(
            all(not gate["harness_gate"] for gate in native),
            "native commands must not be promoted into the network-free static harness",
        )


if __name__ == "__main__":
    unittest.main()
