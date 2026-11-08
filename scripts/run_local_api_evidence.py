#!/usr/bin/env python3
"""Plan or explicitly run local, offline candidate API evidence. Never installs tools."""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

STABLE = "1.97.1"
NIGHTLY = "nightly-2026-08-11"
PACKAGES = ("evidence-search-contracts", "evidence-search-core", "searchright-plugin-sdk")
PINS = {"public-api": "0.52.0", "semver-checks": "0.50.0"}
BUILD_OVERRIDES = (
    "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTDOC", "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS", "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS",
    "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_RUSTDOCFLAGS",
    "CARGO_BUILD_TARGET",
)
ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class CommandResult:
    returncode: int
    stdout: bytes
    stderr: bytes


class PreconditionError(ValueError):
    """Static, safe diagnostic supplied only by this runner."""


def run_command(argv: list[str], cwd: Path, env: dict[str, str]) -> CommandResult:
    """Trusted local subprocess, not a sandbox for build scripts or tool binaries."""
    try:
        result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True,
                                check=False, timeout=2400)
        return CommandResult(result.returncode, result.stdout, result.stderr)
    except FileNotFoundError:
        return CommandResult(127, b"", b"required executable unavailable")
    except subprocess.TimeoutExpired as error:
        return CommandResult(124, error.stdout or b"", error.stderr or b"command timed out")


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def prepare_output(root: Path, output: Path, execute: bool) -> Path:
    """Reject broad/outside/symlink paths; never replace existing output bytes."""
    output = Path(os.path.abspath(output))
    target = root / "target"
    if not output.is_relative_to(target) or output == target:
        raise PreconditionError("output must be a child directory beneath repository target")
    for path in (output, *output.parents):
        if path == root:
            break
        if path.is_symlink():
            raise PreconditionError("output must not traverse symlinks")
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise PreconditionError("output directory must be absent or empty")
    if execute:
        output.mkdir(parents=True, exist_ok=True, mode=0o700)
    return output


def collect_evidence(root: Path, base: str, *, execute: bool = False,
                     output: Path | None = None, runner=run_command) -> dict:
    original_root = Path(os.path.abspath(root))
    root = root.resolve()
    if output is not None:
        output = Path(os.path.abspath(output))
        if output.is_relative_to(original_root):
            output = root / output.relative_to(original_root)
    env = dict(os.environ)
    # Override installation/network defaults without changing the user's toolchain.
    env.update(CARGO_NET_OFFLINE="true", RUSTUP_AUTO_INSTALL="0", GIT_TERMINAL_PROMPT="0",
               GIT_NO_REPLACE_OBJECTS="1",
               CARGO_TERM_COLOR="never")
    report = {
        "schema_version": "org.searchright.local-api-evidence.v1",
        "track_id": "03", "mode": "execute" if execute else "dry_run",
        "status": "blocked", "requested_base": base, "commands": [], "packages": [],
        "required_toolchains": [STABLE, NIGHTLY], "required_tools": PINS,
        "workflow_equivalence": "Existing candidate-public-api package set and commands, invoked with explicit rustup run toolchains; no installation or upload steps.",
        "limitations": ["Local API/SemVer evidence is not publication, downstream compatibility or cutover authority.",
                        "Cargo offline and rustup no-auto-install flags are not an OS network sandbox; only trusted local source, Cargo configuration and binaries may execute.",
                        "Tool output is retained locally and hashed; do not publish logs without review.",
                        "A clean-tree recheck detects persistent source drift, not hostile transient changes."],
    }

    def git(*args: str) -> bytes:
        result = runner(["git", *args], root, env)
        if result.returncode:
            raise PreconditionError("local Git inspection failed; require locally available commits and tracked manifests")
        return result.stdout.strip()

    def revision(ref: str) -> str:
        result = git("rev-parse", "--verify", "--end-of-options", ref).decode("ascii")
        if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", result):
            raise PreconditionError("Git did not resolve an exact object identity")
        return result

    try:
        overrides = sorted(name for name, value in env.items() if value and (
            name in BUILD_OVERRIDES
            or (name.startswith("CARGO_TARGET_") and name.endswith("_RUSTFLAGS"))
        ))
        if overrides:
            report["rejected_build_overrides"] = overrides
            raise PreconditionError("compiler/wrapper/flag environment overrides require separate review; values are not recorded")
        if not base or base.startswith("-") or any(ord(c) < 32 for c in base):
            raise PreconditionError("an explicit non-option base revision is required")
        if Path(git("rev-parse", "--show-toplevel").decode()).resolve() != root:
            raise PreconditionError("root must be the exact repository worktree")
        if git("status", "--porcelain", "--untracked-files=all"):
            raise PreconditionError("a clean worktree including untracked source is required")
        head = revision("HEAD^{commit}")
        tree = revision("HEAD^{tree}")
        baseline = revision(base + "^{commit}")
        base_tree = revision(baseline + "^{tree}")
        git("cat-file", "-e", head + ":Cargo.lock")
        policy = json.loads((root / "release/public-packages.json").read_text())
        if policy.get("development_toolchain") != STABLE or tuple(p["name"] for p in policy["packages"]) != PACKAGES:
            raise PreconditionError("candidate policy changed; review runner/workflow pins explicitly")
        report.update(head_revision=head, head_tree=tree, base_revision=baseline, base_tree=base_tree,
                      policy_sha256=sha256((root / "release/public-packages.json").read_bytes()))
        destination = output or root / "target/local-api-evidence" / f"{head[:12]}-{baseline[:12]}"
        destination = prepare_output(root, destination, execute)
        report["output_directory"] = str(destination)
        env["CARGO_TARGET_DIR"] = str(destination / "build")
        for name in PACKAGES:
            manifest = f"crates/{name}/Cargo.toml"
            git("cat-file", "-e", head + ":" + manifest)
            exists = bool(git("ls-tree", baseline, "--", manifest))
            report["packages"].append({
                "package": name, "baseline_package_present": exists,
                "capture": {"status": "planned", "cwd": str(root / "crates" / name), "argv": ["rustup", "run", NIGHTLY, "cargo", "public-api", "-sss"]},
                "comparison": {"status": "planned" if exists else "initial_baseline_absent",
                               "cwd": str(root),
                               "argv": ["rustup", "run", STABLE, "cargo", "semver-checks", "--package", name, "--baseline-rev", baseline] if exists else None},
            })
        if not execute:
            report["status"] = "dry_run"
            return report

        def invoke(argv: list[str], cwd: Path, label: str) -> dict:
            result = runner(argv, cwd, env)
            item = {"argv": list(argv), "cwd": str(cwd), "exit_code": result.returncode,
                    "environment": {key: env[key] for key in ("CARGO_NET_OFFLINE", "RUSTUP_AUTO_INSTALL", "CARGO_TARGET_DIR", "CARGO_TERM_COLOR")}}
            for stream, data in (("stdout", result.stdout), ("stderr", result.stderr)):
                path = destination / f"{label}.{stream}.log"
                with path.open("xb") as handle:
                    handle.write(data)
                item[stream + "_path"] = str(path)
                item[stream + "_sha256"] = sha256(data)
                item[stream + "_bytes"] = len(data)
            report["commands"].append(item)
            return {"record": item, "result": result}

        tools = invoke(["rustup", "toolchain", "list"], root, "toolchains")
        installed = tools["result"].stdout.decode("utf-8", errors="replace").splitlines()
        missing = [pin for pin in (STABLE, NIGHTLY) if
            not any(line.split()[0] == pin or line.split()[0].startswith(pin + "-")
                    for line in installed if line.split())]
        report["missing_toolchains"] = missing
        tool_error = tools["result"].returncode != 0 or bool(missing)
        versions = [(["rustup", "run", STABLE, "rustc", "--version"], "rustc", STABLE),
                    (["rustup", "run", STABLE, "cargo", "--version"], "cargo", STABLE),
                    (["rustup", "run", NIGHTLY, "cargo", "--version"], "nightly-cargo", None)]
        versions += [(["rustup", "run", NIGHTLY if name == "public-api" else STABLE, "cargo", name, "--version"], name, pin)
                     for name, pin in PINS.items()]
        report["observed_tool_versions"] = {}
        if not tool_error:
            for argv, label, expected in versions:
                probe = invoke(argv, root, "version-" + label)
                result = probe["result"]
                match = re.search(rb"\b(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?)\b", result.stdout)
                observed = match.group(1).decode() if match else None
                report["observed_tool_versions"][label] = observed
                if result.returncode or observed is None or (expected and observed != expected):
                    tool_error = True
        if tool_error:
            report["status"] = "tool_unavailable"
            for package in report["packages"]:
                package["capture"]["status"] = "skipped_tool_unavailable"
                if package["baseline_package_present"]:
                    package["comparison"]["status"] = "skipped_tool_unavailable"
        else:
            for package in report["packages"]:
                name = package["package"]
                capture = invoke(package["capture"]["argv"], root / "crates" / name, name + "-api")
                package["capture"].update(capture["record"])
                captured = capture["result"].returncode == 0 and bool(capture["result"].stdout.strip())
                package["capture"]["status"] = "succeeded" if captured else "failed"
                if not captured:
                    if package["baseline_package_present"]:
                        package["comparison"]["status"] = "skipped_capture_failed"
                    continue
                if package["baseline_package_present"]:
                    comparison = invoke(package["comparison"]["argv"], root, name + "-semver")
                    package["comparison"].update(comparison["record"])
                    result = comparison["result"]
                    status = "compatible" if result.returncode == 0 else "failed"
                    # Conservative recognition: unrelated compiler/tool errors remain failed.
                    if result.returncode == 1 and re.search(rb"semver requires new (?:major|minor) version", result.stdout + result.stderr):
                        status = "incompatible"
                    package["comparison"]["status"] = status
            states = [p["comparison"]["status"] for p in report["packages"]]
            if any(p["capture"]["status"] != "succeeded" for p in report["packages"]) or "failed" in states:
                report["status"] = "failed"
            elif "incompatible" in states:
                report["status"] = "incompatible"
            elif "initial_baseline_absent" in states:
                report["status"] = "initial_capture_only"
            else:
                report["status"] = "passed"
        if revision("HEAD^{commit}") != head or revision("HEAD^{tree}") != tree or git("status", "--porcelain", "--untracked-files=all"):
            report["status"] = "source_changed"
        with (destination / "receipt.json").open("x") as handle:
            json.dump(report, handle, indent=2, sort_keys=True)
            handle.write("\n")
    except (ValueError, OSError, KeyError, TypeError, UnicodeError) as error:
        report["status"] = "blocked"
        # Deliberately exclude arbitrary exception text and tool output from stdout.
        report["error_kind"] = type(error).__name__
        report["error"] = str(error) if isinstance(error, PreconditionError) else "Local precondition or evidence write failed; no success is claimed. Existing output is retained."
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True, help="Explicit locally available Git base; never fetched")
    parser.add_argument("--execute", action="store_true", help="Run already-installed tools offline")
    parser.add_argument("--output", type=Path, help="Absent or empty directory beneath repository target")
    args = parser.parse_args()
    report = collect_evidence(ROOT, args.base, execute=args.execute, output=args.output)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["status"] in {"dry_run", "passed", "initial_capture_only"} else 1


if __name__ == "__main__":
    raise SystemExit(main())
