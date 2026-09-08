"""Offline command-double tests: no Cargo, rustup or network execution."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from run_local_api_evidence import BUILD_OVERRIDES, CommandResult, collect_evidence


class FakeRunner:
    def __init__(self):
        self.calls = []
        self.dirty = False
        self.absent = False
        self.missing_tool = False
        self.semver_output = b""
        self.semver_code = 0
        self.empty_capture = False
        self.drift = False
        self.head_reads = 0
        self.wrong_version = False
        self.missing_binary = False

    def __call__(self, argv, cwd, env):
        self.calls.append((list(argv), cwd, dict(env)))
        if argv[0] == "git":
            if "--show-toplevel" in argv:
                return CommandResult(0, str(cwd).encode(), b"")
            if "status" in argv:
                return CommandResult(0, b"?? changed" if self.dirty else b"", b"")
            if "HEAD^{commit}" in argv:
                self.head_reads += 1
                sha = "d" if self.drift and self.head_reads > 1 else "a"
                return CommandResult(0, (sha * 40).encode(), b"")
            if "rev-parse" in argv:
                return CommandResult(0, (("c" if "HEAD^{tree}" in argv else "b") * 40).encode(), b"")
            if "ls-tree" in argv:
                return CommandResult(0, b"" if self.absent else b"100644 blob abc\tCargo.toml\n", b"")
            return CommandResult(0, b"", b"")
        if self.missing_binary:
            return CommandResult(127, b"", b"unavailable")
        if argv[1:] == ["toolchain", "list"]:
            value = "1.97.1-host\n" if self.missing_tool else "1.97.1-host\nnightly-2026-08-11-host\n"
            return CommandResult(0, value.encode(), b"")
        if "--version" in argv:
            name = argv[-2]
            versions = {"rustc": "rustc 1.97.1 (test)", "cargo": "cargo 1.97.1 (test)",
                        "public-api": "cargo-public-api 0.52.0", "semver-checks": "cargo-semver-checks 0.50.0"}
            value = "cargo 1.98.0" if self.wrong_version and name == "cargo" else versions[name]
            return CommandResult(0, value.encode(), b"")
        if "public-api" in argv:
            return CommandResult(0, b"" if self.empty_capture else b"pub fn synthetic();\n", b"")
        return CommandResult(self.semver_code, self.semver_output, b"")


class LocalApiEvidenceTests(unittest.TestCase):
    def setUp(self):
        environment = patch.dict("os.environ", {name: "" for name in BUILD_OVERRIDES})
        environment.start()
        self.addCleanup(environment.stop)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "release").mkdir()
        names = ["evidence-search-contracts", "evidence-search-core", "searchright-plugin-sdk"]
        (self.root / "release/public-packages.json").write_text(json.dumps({"development_toolchain": "1.97.1", "packages": [{"name": n} for n in names]}))
        (self.root / "Cargo.lock").write_text("# synthetic\n")
        self.fake = FakeRunner()

    def run_evidence(self, **kwargs):
        return collect_evidence(self.root, "chosen-base", runner=self.fake, **kwargs)

    def test_dry_run_never_probes_tools_or_creates_output(self):
        report = self.run_evidence()
        self.assertEqual(report["status"], "dry_run")
        self.assertTrue(all(call[0][0] == "git" for call in self.fake.calls))
        self.assertFalse((self.root / "target").exists())
        self.assertEqual(report["base_revision"], "b" * 40)

    def test_success_uses_rustup_exact_toolchains_and_offline_environment(self):
        report = self.run_evidence(execute=True)
        self.assertEqual(report["status"], "passed")
        commands = [x for x in self.fake.calls if "public-api" in x[0] and "--version" not in x[0]]
        self.assertEqual(len(commands), 3)
        for argv, _, env in commands:
            self.assertEqual(argv[:4], ["rustup", "run", "nightly-2026-08-11", "cargo"])
            self.assertEqual(env["CARGO_NET_OFFLINE"], "true")
            self.assertEqual(env["RUSTUP_AUTO_INSTALL"], "0")
        self.assertEqual(len(report["packages"]), 3)
        self.assertTrue(all(len(p["capture"]["stdout_sha256"]) == 64 for p in report["packages"]))

    def test_missing_nightly_is_not_success_and_runs_no_capture(self):
        self.fake.missing_tool = True
        report = self.run_evidence(execute=True)
        self.assertEqual(report["status"], "tool_unavailable")
        self.assertFalse(any("-sss" in c[0] for c in self.fake.calls))

    def test_dirty_tree_fails_before_tool_execution(self):
        self.fake.dirty = True
        self.assertEqual(self.run_evidence(execute=True)["status"], "blocked")
        self.assertTrue(all(c[0][0] == "git" for c in self.fake.calls))

    def test_absent_baseline_is_initial_capture_not_comparison_success(self):
        self.fake.absent = True
        report = self.run_evidence(execute=True)
        self.assertEqual(report["status"], "initial_capture_only")
        self.assertTrue(all(p["comparison"]["status"] == "initial_baseline_absent" for p in report["packages"]))

    def test_semver_incompatible_and_generic_failure_are_distinct(self):
        self.fake.semver_code = 1
        self.fake.semver_output = b"semver requires new major version"
        report = self.run_evidence(execute=True)
        self.assertEqual(report["status"], "incompatible")
        self.fake.semver_output = b"rustdoc failed"
        report = self.run_evidence(execute=True, output=self.root / "target/second")
        self.assertEqual(report["status"], "failed")

    def test_empty_capture_cannot_pass(self):
        self.fake.empty_capture = True
        report = self.run_evidence(execute=True)
        self.assertEqual(report["status"], "failed")
        self.assertTrue(all(p["comparison"]["status"] == "skipped_capture_failed" for p in report["packages"]))

    def test_output_no_clobber_and_symlink_rejection(self):
        out = self.root / "target/kept"
        out.mkdir(parents=True)
        (out / "user").write_text("keep")
        self.assertEqual(self.run_evidence(execute=True, output=out)["status"], "blocked")
        self.assertEqual((out / "user").read_text(), "keep")
        (self.root / "target/link").symlink_to(out, target_is_directory=True)
        self.assertEqual(self.run_evidence(execute=True, output=self.root / "target/link/new")["status"], "blocked")

    def test_output_outside_target_rejected(self):
        self.assertEqual(self.run_evidence(execute=True, output=self.root / "elsewhere")["status"], "blocked")

    def test_source_drift_invalidates_success(self):
        self.fake.drift = True
        self.assertEqual(self.run_evidence(execute=True)["status"], "source_changed")

    def test_wrong_cargo_version_rejects_before_capture(self):
        self.fake.wrong_version = True
        self.assertEqual(self.run_evidence(execute=True)["status"], "tool_unavailable")
        self.assertFalse(any("-sss" in c[0] for c in self.fake.calls))

    def test_missing_rustup_is_recorded_not_success(self):
        self.fake.missing_binary = True
        report = self.run_evidence(execute=True)
        self.assertEqual(report["status"], "tool_unavailable")
        self.assertEqual(report["missing_toolchains"], ["1.97.1", "nightly-2026-08-11"])

    def test_saved_receipt_and_artifact_hashes_match(self):
        import hashlib
        report = self.run_evidence(execute=True)
        saved = json.loads((Path(report["output_directory"]) / "receipt.json").read_text())
        self.assertEqual(saved, report)
        for command in report["commands"]:
            for stream in ("stdout", "stderr"):
                self.assertEqual(command[stream + "_sha256"], hashlib.sha256(Path(command[stream + "_path"]).read_bytes()).hexdigest())

    def test_invalid_explicit_base_never_runs_git_or_tools(self):
        for base in ("", "--help", "HEAD\n"):
            with self.subTest(base=base):
                report = collect_evidence(self.root, base, execute=True, runner=self.fake)
                self.assertEqual(report["status"], "blocked")
        self.assertEqual(self.fake.calls, [])

    def test_compiler_wrapper_and_flag_overrides_fail_without_exposing_values(self):
        for name in BUILD_OVERRIDES:
            with self.subTest(name=name), patch.dict("os.environ", {name: "private-override"}):
                report = self.run_evidence(execute=True)
                self.assertEqual(report["status"], "blocked")
                self.assertEqual(report["rejected_build_overrides"], [name])
                self.assertNotIn("private-override", json.dumps(report))
        self.assertEqual(self.fake.calls, [])

    def test_target_specific_flags_rejected_before_git_or_tools(self):
        for name in ("CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS",
                     "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS"):
            with self.subTest(name=name), patch.dict("os.environ", {name: "--cfg private_api"}):
                report = self.run_evidence(execute=True)
                self.assertEqual(report["status"], "blocked")
                self.assertEqual(report["rejected_build_overrides"], [name])
                self.assertNotIn("private_api", json.dumps(report))
        self.assertEqual(self.fake.calls, [])


if __name__ == "__main__":
    unittest.main()
