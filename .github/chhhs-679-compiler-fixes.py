from __future__ import annotations

import json
from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"expected source fragment was not found in {path}: {old!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    Path("crates/searchright-chhhs-demo/src/attribution.rs"),
    "use std::collections::BTreeSet;\n\n",
    "",
)

model = Path("crates/searchright-chhhs-demo/src/model.rs")
text = model.read_text(encoding="utf-8")
start_marker = "pub fn normalise_text(value: &str) -> String {"
end_marker = "\n}\n\nfn require_text"
start = text.find(start_marker)
if start < 0:
    raise SystemExit("normalise_text start marker was not found")
end = text.find(end_marker, start)
if end < 0:
    raise SystemExit("normalise_text end marker was not found")
replacement = "\n".join(
    [
        "pub fn normalise_text(value: &str) -> String {",
        "    let mut normalised = String::with_capacity(value.len());",
        "    for character in value.chars() {",
        "        if character == '&' {",
        '            normalised.push_str(" and ");',
        "        } else if character.is_alphanumeric() {",
        "            normalised.extend(character.to_lowercase());",
        "        } else {",
        "            normalised.push(' ');",
        "        }",
        "    }",
        "    normalised",
        "        .split_whitespace()",
        "        .collect::<Vec<_>>()",
        '        .join(" ")',
        "}",
    ]
)
text = text[:start] + replacement + text[end + 2 :]
old_expected = '            "cairns hinterland hhs"\n'
new_expected = '            "cairns and hinterland hhs"\n'
if old_expected not in text:
    raise SystemExit("normalisation test expectation was not found")
text = text.replace(old_expected, new_expected, 1)
model.write_text(text, encoding="utf-8")

fixture_path = Path("crates/searchright-chhhs-demo/fixtures/europe-pmc.json")
fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
rows = fixture.get("resultList", {}).get("result", [])
if len(rows) != 2:
    raise SystemExit(f"expected two Europe PMC fixture rows, found {len(rows)}")
for row in rows:
    row["source"] = "MED"
fixture_path.write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")

taxonomy_path = Path("crates/searchright-chhhs-demo/resources/research-taxonomy.json")
taxonomy = json.loads(taxonomy_path.read_text(encoding="utf-8"))
redundant = {
    "systematic-review": {"meta-analysis"},
    "case-control-study": {"case-control"},
    "cross-sectional-study": {"cross-sectional"},
    "modelling-economic-evaluation": {"cost-effectiveness"},
}
removed: set[tuple[str, str]] = set()
for rule in taxonomy["study_types"]:
    excluded = redundant.get(rule["category_id"], set())
    retained: list[str] = []
    for term in rule["terms"]:
        if term in excluded:
            removed.add((rule["category_id"], term))
        else:
            retained.append(term)
    rule["terms"] = retained
expected = {(category, term) for category, terms in redundant.items() for term in terms}
if removed != expected:
    raise SystemExit(f"unexpected taxonomy correction set: {sorted(removed)}")
taxonomy_path.write_text(
    json.dumps(taxonomy, indent=2, ensure_ascii=False) + "\n",
    encoding="utf-8",
)
