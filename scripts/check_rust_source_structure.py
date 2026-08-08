#!/usr/bin/env python3
"""Network-free structural checks for the Rust workspace.

This is deliberately a compiler surrogate, not a compiler claim. It catches
common generated-source defects before CI obtains a real toolchain.
"""
from __future__ import annotations

import json
import re
import sys
import tomllib
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ERRORS: list[str] = []
CHECKS: Counter[str] = Counter()

IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


def err(msg: str) -> None:
    ERRORS.append(msg)


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def strip_comments_and_literals(source: str) -> str:
    """Replace comments and literals with spaces while preserving newlines."""
    out: list[str] = []
    i = 0
    n = len(source)
    block_depth = 0
    while i < n:
        if block_depth:
            if source.startswith("/*", i):
                block_depth += 1
                out.extend("  ")
                i += 2
            elif source.startswith("*/", i):
                block_depth -= 1
                out.extend("  ")
                i += 2
            else:
                ch = source[i]
                out.append("\n" if ch == "\n" else " ")
                i += 1
            continue
        if source.startswith("//", i):
            end = source.find("\n", i)
            if end < 0:
                out.extend(" " * (n - i))
                break
            out.extend(" " * (end - i))
            out.append("\n")
            i = end + 1
            continue
        if source.startswith("/*", i):
            block_depth = 1
            out.extend("  ")
            i += 2
            continue
        ch = source[i]
        if ch == '"':
            # Normal string. Raw strings are handled below.
            out.append(" ")
            i += 1
            while i < n:
                ch = source[i]
                out.append("\n" if ch == "\n" else " ")
                i += 1
                if ch == "\\" and i < n:
                    ch2 = source[i]
                    out.append("\n" if ch2 == "\n" else " ")
                    i += 1
                elif ch == '"':
                    break
            continue
        if ch == "r":
            match = re.match(r'r(#{0,255})"', source[i:])
            if match:
                hashes = match.group(1)
                opener = match.group(0)
                out.extend(" " * len(opener))
                i += len(opener)
                closer = '"' + hashes
                end = source.find(closer, i)
                if end < 0:
                    out.extend("\n" if c == "\n" else " " for c in source[i:])
                    break
                out.extend("\n" if c == "\n" else " " for c in source[i:end + len(closer)])
                i = end + len(closer)
                continue
        if ch == "'":
            # Distinguish a character literal from a lifetime.
            if i + 2 < n and source[i + 1] != " " and (
                source[i + 1] == "\\" or source.find("'", i + 1, min(n, i + 8)) != -1
            ):
                out.append(" ")
                i += 1
                while i < n:
                    ch = source[i]
                    out.append("\n" if ch == "\n" else " ")
                    i += 1
                    if ch == "\\" and i < n:
                        ch2 = source[i]
                        out.append("\n" if ch2 == "\n" else " ")
                        i += 1
                    elif ch == "'":
                        break
                continue
        out.append(ch)
        i += 1
    if block_depth:
        err("unterminated block comment")
    return "".join(out)


def matching_brace(text: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def top_level_segments(body: str) -> list[str]:
    segments: list[str] = []
    start = 0
    brace = paren = bracket = angle = 0
    for i, ch in enumerate(body):
        if ch == "{": brace += 1
        elif ch == "}": brace -= 1
        elif ch == "(": paren += 1
        elif ch == ")": paren -= 1
        elif ch == "[": bracket += 1
        elif ch == "]": bracket -= 1
        elif ch == "<": angle += 1
        elif ch == ">" and angle: angle -= 1
        elif ch == "," and brace == paren == bracket == angle == 0:
            segments.append(body[start:i])
            start = i + 1
    segments.append(body[start:])
    return segments


def check_structs_enums(path: Path, source: str, clean: str) -> None:
    header = re.compile(rf"\b(struct|enum)\s+({IDENT})(?:\s*<[^{{;]*>)?\s*{{")
    for match in header.finditer(clean):
        kind, name = match.group(1), match.group(2)
        opening = clean.find("{", match.start())
        closing = matching_brace(clean, opening)
        if closing is None:
            err(f"{rel(path)}: unmatched brace for {kind} {name}")
            continue
        body = clean[opening + 1:closing]
        names: list[str] = []
        if kind == "struct":
            for segment in top_level_segments(body):
                segment = re.sub(r"#\s*\[[^\]]*\]", " ", segment, flags=re.S).strip()
                field = re.match(rf"(?:pub(?:\([^)]*\))?\s+)?({IDENT})\s*:", segment)
                if field:
                    names.append(field.group(1))
        else:
            for segment in top_level_segments(body):
                segment = re.sub(r"#\s*\[[^\]]*\]", " ", segment, flags=re.S).strip()
                variant = re.match(rf"({IDENT})\b", segment)
                if variant:
                    names.append(variant.group(1))
        duplicates = sorted(name for name, count in Counter(names).items() if count > 1)
        for duplicate in duplicates:
            err(f"{rel(path)}: duplicate {kind} member `{duplicate}` in {name}")
        CHECKS[f"{kind}s_checked"] += 1

        # Eq cannot be derived for direct floating-point fields.
        before = source[max(0, match.start() - 500):match.start()]
        derive_blocks = re.findall(r"#\s*\[\s*derive\s*\(([^)]*)\)\s*\]", before, flags=re.S)
        derives = derive_blocks[-1] if derive_blocks else ""
        if re.search(r"\bEq\b", derives) and re.search(r"\b(?:f32|f64)\b", body):
            err(f"{rel(path)}: {kind} {name} derives Eq with a direct floating-point member")


def check_impl_methods(path: Path, clean: str) -> None:
    impl_header = re.compile(r"\bimpl(?:\s*<[^{}]*>)?\s+([^{}]+?)\s*{")
    for match in impl_header.finditer(clean):
        opening = clean.find("{", match.start())
        closing = matching_brace(clean, opening)
        if closing is None:
            continue
        body = clean[opening + 1:closing]
        methods = re.findall(rf"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+({IDENT})\s*(?:<[^{{;]*>)?\s*\(", body)
        duplicates = sorted(name for name, count in Counter(methods).items() if count > 1)
        for duplicate in duplicates:
            err(f"{rel(path)}: duplicate method `{duplicate}` in impl {match.group(1).strip()}")
        CHECKS["impl_blocks_checked"] += 1


def check_modules(path: Path, clean: str) -> None:
    for match in re.finditer(rf"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+({IDENT})\s*;", clean):
        module = match.group(1)
        base = path.parent
        candidates = [base / f"{module}.rs", base / module / "mod.rs"]
        if not any(candidate.is_file() for candidate in candidates):
            err(f"{rel(path)}: module `{module}` has no source file")
        CHECKS["module_declarations_checked"] += 1


def check_forbidden_patterns(path: Path, clean: str) -> None:
    patterns = {
        "unwrap": r"\.unwrap\s*\(",
        "expect": r"\.expect\s*\(",
        "todo": r"\btodo!\s*\(",
        "unimplemented": r"\bunimplemented!\s*\(",
        "dbg": r"\bdbg!\s*\(",
        "println": r"\bprintln!\s*\(",
        "eprintln": r"\beprintln!\s*\(",
        "unsafe": r"\bunsafe\b",
    }
    crate_allows_stdout = bool(re.search(r"#!\s*\[\s*allow\s*\([^]]*clippy::print_stdout", clean, flags=re.S))
    crate_allows_stderr = bool(re.search(r"#!\s*\[\s*allow\s*\([^]]*clippy::print_stderr", clean, flags=re.S))
    for label, pattern in patterns.items():
        for match in re.finditer(pattern, clean):
            line = clean.count("\n", 0, match.start()) + 1
            context = clean[max(0, match.start() - 300):match.start()]
            # Explicit, reasoned allow attributes are accepted for print macros only.
            if label == "println" and crate_allows_stdout:
                continue
            if label == "eprintln" and crate_allows_stderr:
                continue
            if label in {"println", "eprintln"} and re.search(
                rf"#\s*\[\s*allow\s*\(\s*clippy::print_{'stdout' if label == 'println' else 'stderr'}\s*,\s*reason\s*=",
                context,
            ):
                continue
            err(f"{rel(path)}:{line}: forbidden `{label}` pattern")


def check_delimiters(path: Path, clean: str) -> None:
    pairs = {')': '(', ']': '[', '}': '{'}
    stack: list[tuple[str, int]] = []
    for index, char in enumerate(clean):
        if char in "([{":
            stack.append((char, index))
        elif char in pairs:
            if not stack or stack[-1][0] != pairs[char]:
                line = clean.count("\n", 0, index) + 1
                err(f"{rel(path)}:{line}: mismatched delimiter `{char}`")
                return
            stack.pop()
    if stack:
        char, index = stack[-1]
        line = clean.count("\n", 0, index) + 1
        err(f"{rel(path)}:{line}: unclosed delimiter `{char}`")
    else:
        CHECKS["delimiter_balanced_files"] += 1


def crate_imports(source: str) -> set[str]:
    found = set(re.findall(r"(?m)^\s*(?:pub\s+)?use\s+([a-z][a-z0-9_]*)::", source))
    found.update(re.findall(r"\bextern\s+crate\s+([a-z][a-z0-9_]*)\b", source))
    return found


def package_to_ident(name: str) -> str:
    return name.replace("-", "_")


def check_workspace() -> None:
    root_manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    members = root_manifest.get("workspace", {}).get("members", [])
    member_paths = {str(item) for item in members}
    actual = {p.parent.relative_to(ROOT).as_posix() for p in (ROOT / "crates").glob("*/Cargo.toml")}
    if member_paths != actual:
        err(f"workspace members mismatch: missing={sorted(actual-member_paths)} extra={sorted(member_paths-actual)}")
    CHECKS["workspace_members_checked"] = len(actual)

    known_external = {
        package_to_ident(name) for name in root_manifest.get("workspace", {}).get("dependencies", {})
    }
    known_std = {
        "alloc", "core", "proc_macro", "std", "test", "crate", "self", "super",
    }
    for manifest_path in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        manifest = tomllib.loads(manifest_path.read_text())
        package = manifest.get("package", {})
        crate_name = package_to_ident(str(package.get("name", manifest_path.parent.name)))
        deps = manifest.get("dependencies", {})
        dev_deps = manifest.get("dev-dependencies", {})
        build_deps = manifest.get("build-dependencies", {})
        declared = {package_to_ident(name) for name in [*deps, *dev_deps, *build_deps]}
        source_files = sorted((manifest_path.parent / "src").rglob("*.rs"))
        imports: set[str] = set()
        for source_path in source_files:
            imports |= crate_imports(strip_comments_and_literals(source_path.read_text()))
        local_modules: set[str] = set()
        for source_path in source_files:
            local_clean = strip_comments_and_literals(source_path.read_text())
            local_modules.update(re.findall(rf"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+({IDENT})\s*(?:;|\{{)", local_clean))
        undeclared = sorted(imports - declared - known_std - local_modules - {crate_name})
        # Macros can import companion crates indirectly; all currently used crates should still be explicit.
        if undeclared:
            err(f"{rel(manifest_path)}: imported crates absent from dependencies: {undeclared}")
        unused = sorted(
            ident for ident in declared
            if ident not in imports and ident not in {"serde", "schemars"}
        )
        # Unused dependency is advisory here; cargo-machete is the authoritative compiler-aware gate.
        if unused:
            CHECKS["possibly_unused_dependencies"] += len(unused)
        entrypoints = []
        if (manifest_path.parent / "src/lib.rs").is_file(): entrypoints.append("lib")
        if (manifest_path.parent / "src/main.rs").is_file(): entrypoints.append("main")
        for binary in manifest.get("bin", []):
            binary_path = manifest_path.parent / str(binary.get("path", ""))
            if not binary_path.is_file():
                err(f"{rel(manifest_path)}: binary path does not exist: {binary_path}")
        if not entrypoints and not manifest.get("bin"):
            err(f"{rel(manifest_path)}: crate has no lib, main or bin entrypoint")
        CHECKS["crate_manifests_checked"] += 1


def main() -> int:
    check_workspace()
    rust_files = sorted((ROOT / "crates").rglob("*.rs"))
    for path in rust_files:
        source = path.read_text(encoding="utf-8")
        clean = strip_comments_and_literals(source)
        check_delimiters(path, clean)
        check_modules(path, clean)
        check_structs_enums(path, source, clean)
        check_impl_methods(path, clean)
        check_forbidden_patterns(path, clean)
        CHECKS["rust_files_checked"] += 1

    receipt = {
        "schema_version": "org.searchright.rust-source-structure-receipt.v1",
        "status": "failed" if ERRORS else "passed",
        "checks": dict(sorted(CHECKS.items())),
        "errors": ERRORS,
        "limitations": [
            "This is lexical and structural validation, not Rust parsing, type checking, macro expansion, linking or execution.",
        ],
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 1 if ERRORS else 0


if __name__ == "__main__":
    raise SystemExit(main())
