#!/usr/bin/env python3
"""Generate dependency-free Python and TypeScript contract-only bindings."""
from __future__ import annotations

import argparse
import hashlib
import json
import keyword
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "contracts" / "schema-catalog.json"
PYTHON_PACKAGE = ROOT / "sdk" / "python" / "searchright_contracts"
TYPESCRIPT_SOURCE = ROOT / "sdk" / "typescript" / "src" / "index.ts"
MANIFEST = ROOT / "sdk" / "generated-contract-bindings.json"


def type_name(value: str) -> str:
    parts = re.findall(r"[A-Za-z0-9]+", value)
    name = "".join(part[:1].upper() + part[1:] for part in parts) or "Contract"
    return f"Contract{name}" if name[:1].isdigit() else name


def python_field(value: str) -> str:
    return f"{value}_" if keyword.iskeyword(value) else value


def literal(value: Any) -> str:
    if value is None:
        return "None"
    if value is True:
        return "True"
    if value is False:
        return "False"
    return repr(value)


def ts_literal(value: Any) -> str:
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    return json.dumps(value, ensure_ascii=False)


def collect_python_objects(
    schema: Any,
    path: str,
    name: str,
    objects: dict[str, tuple[str, dict[str, Any]]],
) -> None:
    if not isinstance(schema, dict):
        return
    if "properties" in schema or (
        schema.get("type") == "object" and schema.get("additionalProperties") is False
    ):
        objects[path] = (name, schema)
        for field_name, field_schema in sorted(schema.get("properties", {}).items()):
            collect_python_objects(
                field_schema,
                f"{path}/properties/{field_name}",
                f"{name}{type_name(field_name)}",
                objects,
            )
    if schema.get("type") == "array":
        collect_python_objects(schema.get("items", {}), f"{path}/items", f"{name}Item", objects)
    for keyword_name in ("oneOf", "anyOf"):
        for index, variant in enumerate(schema.get(keyword_name, [])):
            collect_python_objects(
                variant,
                f"{path}/{keyword_name}/{index}",
                f"{name}Variant{index + 1}",
                objects,
            )


def python_type(
    schema: Any,
    references: dict[str, str],
    inline_names: dict[str, str],
    path: str,
) -> str:
    if not isinstance(schema, dict):
        return "JsonValue"
    if "$ref" in schema:
        return references.get(schema["$ref"], "JsonValue")
    if "const" in schema:
        return f"Literal[{literal(schema['const'])}]"
    if isinstance(schema.get("enum"), list):
        values = ", ".join(literal(value) for value in schema["enum"])
        return f"Literal[{values}]"
    variants = schema.get("oneOf") or schema.get("anyOf")
    if isinstance(variants, list):
        keyword_name = "oneOf" if "oneOf" in schema else "anyOf"
        return " | ".join(
            dict.fromkeys(
                python_type(
                    item,
                    references,
                    inline_names,
                    f"{path}/{keyword_name}/{index}",
                )
                for index, item in enumerate(variants)
            )
        )
    kind = schema.get("type")
    if isinstance(kind, list):
        return " | ".join(
            dict.fromkeys(
                python_type({**schema, "type": item}, references, inline_names, path)
                for item in kind
            )
        )
    if kind == "string":
        return "str"
    if kind == "integer":
        return "int"
    if kind == "number":
        return "int | float"
    if kind == "boolean":
        return "bool"
    if kind == "null":
        return "None"
    if kind == "array":
        return (
            "list["
            + python_type(schema.get("items", {}), references, inline_names, f"{path}/items")
            + "]"
        )
    if kind == "object" or "properties" in schema:
        if path in inline_names:
            return inline_names[path]
        additional = schema.get("additionalProperties")
        value_type = (
            python_type(additional, references, inline_names, f"{path}/additionalProperties")
            if isinstance(additional, dict)
            else "JsonValue"
        )
        return f"dict[str, {value_type}]"
    return "JsonValue"


def typescript_type(schema: Any, references: dict[str, str]) -> str:
    if not isinstance(schema, dict):
        return "JsonValue"
    if "$ref" in schema:
        return references.get(schema["$ref"], "JsonValue")
    if "const" in schema:
        return ts_literal(schema["const"])
    if isinstance(schema.get("enum"), list):
        return " | ".join(ts_literal(value) for value in schema["enum"])
    variants = schema.get("oneOf") or schema.get("anyOf")
    if isinstance(variants, list):
        union = " | ".join(dict.fromkeys(typescript_type(item, references) for item in variants))
        base = {key: value for key, value in schema.items() if key not in {"oneOf", "anyOf"}}
        if any(key in base for key in ("type", "properties", "required", "$ref")):
            return f"({typescript_type(base, references)}) & ({union})"
        return union
    kind = schema.get("type")
    if isinstance(kind, list):
        return " | ".join(
            dict.fromkeys(typescript_type({**schema, "type": item}, references) for item in kind)
        )
    if kind == "string":
        return "string"
    if kind in {"integer", "number"}:
        return "number"
    if kind == "boolean":
        return "boolean"
    if kind == "null":
        return "null"
    if kind == "array":
        return f"ReadonlyArray<{typescript_type(schema.get('items', {}), references)}>"
    if kind == "object" or "properties" in schema or "required" in schema:
        properties = schema.get("properties", {})
        required = set(schema.get("required", []))
        fields = [
            f"readonly {json.dumps(name)}{'?' if name not in required else ''}: "
            f"{typescript_type(value, references)};"
            for name, value in sorted((properties | {name: {} for name in required - properties.keys()}).items())
        ]
        additional = schema.get("additionalProperties")
        if isinstance(additional, dict):
            fields.append(
                f"readonly [key: string]: {typescript_type(additional, references)};"
            )
        if fields:
            return "{ " + " ".join(fields) + " }"
        return "Readonly<Record<string, JsonValue>>"
    return "JsonValue"


def python_typed_dict(
    name: str,
    schema: dict[str, Any],
    references: dict[str, str],
    inline_names: dict[str, str],
    path: str,
) -> str:
    properties = schema.get("properties", {})
    required = set(schema.get("required", []))
    fields = []
    for field_name, field_schema in sorted(properties.items()):
        annotation = python_type(
            field_schema,
            references,
            inline_names,
            f"{path}/properties/{field_name}",
        )
        wrapper = "Required" if field_name in required else "NotRequired"
        fields.append(f"    {field_name!r}: {wrapper}[{annotation!r}],")
    body = "\n".join(fields)
    return f"{name} = TypedDict(\n    {name!r},\n    {{\n{body}\n    }},\n)"


def root_type_names(entries: list[dict[str, Any]]) -> dict[str, str]:
    """Return stable, catalogue-ID-owned root names without Rust-name collisions."""
    return {entry["id"]: type_name(entry["id"]) for entry in entries}


def external_references(
    entries: list[dict[str, Any]], root_names: dict[str, str]
) -> dict[str, str]:
    references: dict[str, str] = {}
    for entry in entries:
        schema_path = Path(entry["schema"])
        schema = json.loads((ROOT / schema_path).read_text(encoding="utf-8"))
        root_name = root_names[entry["id"]]
        references[schema_path.name] = root_name
        references[schema_path.as_posix()] = root_name
        if isinstance(schema.get("$id"), str):
            references[schema["$id"]] = root_name
    return references


def unresolved_references(
    entries: list[dict[str, Any]], external_refs: dict[str, str]
) -> list[str]:
    unresolved: list[str] = []

    def visit(contract_id: str, schema: Any, definitions: set[str]) -> None:
        if isinstance(schema, dict):
            reference = schema.get("$ref")
            if isinstance(reference, str):
                if reference.startswith("#/$defs/"):
                    if reference.removeprefix("#/$defs/") not in definitions:
                        unresolved.append(f"{contract_id}: {reference}")
                elif reference not in external_refs:
                    unresolved.append(f"{contract_id}: {reference}")
            for value in schema.values():
                visit(contract_id, value, definitions)
        elif isinstance(schema, list):
            for value in schema:
                visit(contract_id, value, definitions)

    for entry in entries:
        schema = json.loads((ROOT / entry["schema"]).read_text(encoding="utf-8"))
        visit(entry["id"], schema, set(schema.get("$defs", {})))
    return sorted(set(unresolved))


def render_python(
    entries: list[dict[str, Any]],
    root_names: dict[str, str],
    external_refs: dict[str, str],
) -> str:
    declarations: list[str] = []
    exported: list[str] = []
    for entry in entries:
        schema = json.loads((ROOT / entry["schema"]).read_text(encoding="utf-8"))
        root_name = root_names[entry["id"]]
        definitions = schema.get("$defs", {})
        references = external_refs | {
            f"#/$defs/{definition}": f"{root_name}{type_name(definition)}"
            for definition in definitions
        }
        objects: dict[str, tuple[str, dict[str, Any]]] = {}
        collect_python_objects(schema, "#", root_name, objects)
        for definition, definition_schema in sorted(definitions.items()):
            name = references[f"#/$defs/{definition}"]
            collect_python_objects(definition_schema, f"#/$defs/{definition}", name, objects)
        inline_names = {path: name for path, (name, _) in objects.items()}
        object_names = set(inline_names.values())
        for path, (name, object_schema) in sorted(objects.items(), key=lambda item: item[1][0]):
            declarations.append(
                python_typed_dict(name, object_schema, references, inline_names, path)
            )
            exported.append(name)
        for definition, definition_schema in sorted(definitions.items()):
            name = references[f"#/$defs/{definition}"]
            if name not in object_names:
                declarations.append(
                    f"{name}: TypeAlias = "
                    f"{python_type(definition_schema, references, inline_names, f'#/$defs/{definition}')}"
                )
                exported.append(name)
        if root_name not in object_names:
            declarations.append(
                f"{root_name}: TypeAlias = {python_type(schema, references, inline_names, '#')}"
            )
            exported.append(root_name)
    contract_ids = tuple(entry["id"] for entry in entries)
    exports = ",\n    ".join(repr(name) for name in sorted(set(exported)))
    return (
        '"""Generated contract-only types. Do not edit by hand."""\n'
        "from __future__ import annotations\n\n"
        "from typing import Literal, NotRequired, Required, TypeAlias, TypedDict\n\n"
        "JsonValue: TypeAlias = None | bool | int | float | str | list['JsonValue'] | dict[str, 'JsonValue']\n"
        f"CONTRACT_IDS = {contract_ids!r}\n\n"
        + "\n\n".join(declarations)
        + "\n\n__all__ = [\n    'CONTRACT_IDS',\n    'JsonValue',\n    "
        + exports
        + "\n]\n"
    )


def render_typescript(
    entries: list[dict[str, Any]],
    root_names: dict[str, str],
    external_refs: dict[str, str],
) -> str:
    declarations = [
        "// Generated contract-only types. Do not edit by hand.",
        "export type JsonValue = null | boolean | number | string | ReadonlyArray<JsonValue> | { readonly [key: string]: JsonValue };",
    ]
    for entry in entries:
        schema = json.loads((ROOT / entry["schema"]).read_text(encoding="utf-8"))
        root_name = root_names[entry["id"]]
        definitions = schema.get("$defs", {})
        references = external_refs | {
            f"#/$defs/{definition}": f"{root_name}{type_name(definition)}"
            for definition in definitions
        }
        for definition, definition_schema in sorted(definitions.items()):
            name = references[f"#/$defs/{definition}"]
            declarations.append(
                f"export type {name} = {typescript_type(definition_schema, references)};"
            )
        declarations.append(f"export type {root_name} = {typescript_type(schema, references)};")
    ids = ", ".join(json.dumps(entry["id"]) for entry in entries)
    declarations.append(f"export const CONTRACT_IDS = [{ids}] as const;")
    return "\n\n".join(declarations) + "\n"


def output_files(entries: list[dict[str, Any]]) -> dict[Path, str]:
    root_names = root_type_names(entries)
    external_refs = external_references(entries, root_names)
    return {
        PYTHON_PACKAGE / "__init__.py": render_python(
            entries, root_names, external_refs
        ),
        PYTHON_PACKAGE / "py.typed": "",
        ROOT / "sdk" / "python" / "pyproject.toml": (
            "[project]\n"
            "name = \"searchright-contracts\"\n"
            "version = \"0.1.0a1\"\n"
            "requires-python = \">=3.11\"\n"
            "description = \"Generated contract-only Searchright type bindings\"\n"
            "dependencies = []\n\n"
            "[tool.searchright]\n"
            "generated = true\n"
            "domain_logic = false\n"
            "publish = false\n"
        ),
        TYPESCRIPT_SOURCE: render_typescript(entries, root_names, external_refs),
        ROOT / "sdk" / "typescript" / "package.json": (
            json.dumps(
                {
                    "name": "@searchright/contracts",
                    "version": "0.1.0-alpha.1",
                    "private": True,
                    "type": "module",
                    "exports": {".": "./src/index.ts"},
                    "files": ["src/index.ts"],
                    "description": "Generated contract-only Searchright type bindings",
                    "searchright": {"generated": True, "domainLogic": False, "publish": False},
                },
                indent=2,
                sort_keys=True,
            )
            + "\n"
        ),
    }


def sha256(content: str) -> str:
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()

    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    entries = sorted(catalog["entries"], key=lambda entry: entry["id"])
    external_refs = external_references(entries, root_type_names(entries))
    unresolved = unresolved_references(entries, external_refs)
    outputs = output_files(entries)
    generated_manifest = {
        "schema_version": "org.searchright.generated-contract-bindings.v1",
        "source_catalog": "contracts/schema-catalog.json",
        "contracts": len(entries),
        "packages": {
            "python": "searchright-contracts",
            "typescript": "@searchright/contracts",
        },
        "files": {
            str(path.relative_to(ROOT)): sha256(content)
            for path, content in sorted(outputs.items(), key=lambda item: str(item[0]))
        },
        "domain_logic": False,
        "automatic_publication": False,
        "claim_boundary": (
            "Generated contract-only types mirror the canonical JSON Schema surface. "
            "They are not clients, validators, published packages, or downstream compatibility evidence."
        ),
    }
    outputs[MANIFEST] = json.dumps(generated_manifest, indent=2, sort_keys=True) + "\n"
    stale = [
        str(path.relative_to(ROOT))
        for path, content in outputs.items()
        if not path.is_file() or path.read_text(encoding="utf-8") != content
    ]
    if args.write:
        for path, content in outputs.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        stale = []
    receipt = {
        "schema_version": "org.searchright.contract-binding-generation-receipt.v1",
        "status": "passed" if not stale and not unresolved else "failed",
        "mode": "write" if args.write else "check",
        "contracts": len(entries),
        "generated_files": len(outputs),
        "stale": sorted(stale),
        "unresolved_references": unresolved,
        "automatic_publication": False,
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0 if not stale and not unresolved else 1


if __name__ == "__main__":
    raise SystemExit(main())
