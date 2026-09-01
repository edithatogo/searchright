from __future__ import annotations

import json
from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"expected source fragment was not found in {path}: {old!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


attribution = Path("crates/searchright-chhhs-demo/src/attribution.rs")
replace_once(attribution, "use std::collections::BTreeSet;\n\n", "")
replace_once(
    attribution,
    "fn structured_rule(alias: &InstitutionAlias) -> (&'static str, u8, &'static str) {",
    "const fn structured_rule(alias: &InstitutionAlias) -> (&'static str, u8, &'static str) {",
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
replace_once(
    model,
    """        let mut value = profile();
        value.aliases[0].kind = AliasKind::Current;
""",
    """        let mut value = profile();
        let alias = value
            .aliases
            .first_mut()
            .unwrap_or_else(|| panic!("profile should contain a canonical alias"));
        alias.kind = AliasKind::Current;
""",
)
replace_once(
    model,
    """        let mut duplicate = value.aliases[0].clone();
""",
    """        let mut duplicate = value
            .aliases
            .first()
            .cloned()
            .unwrap_or_else(|| panic!("profile should contain a canonical alias"));
""",
)

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

pipeline = Path("crates/searchright-chhhs-demo/src/pipeline.rs")
replace_once(
    pipeline,
    '    output.source_receipt_id = "monitor-diff".to_owned();\n',
    '    "monitor-diff".clone_into(&mut output.source_receipt_id);\n',
)
replace_once(
    pipeline,
    """    (current == "[untitled]" && candidate != "[untitled]")
        || (candidate != "[untitled]" && candidate.len() > current.len())
""",
    """    candidate != "[untitled]"
        && (current == "[untitled]" || candidate.len() > current.len())
""",
)
replace_once(
    pipeline,
    """        assert_eq!(state.runs[0].attributed_count, 5);
        assert_eq!(state.runs[0].excluded_count, 2);
""",
    """        let initial_run = state
            .runs
            .first()
            .unwrap_or_else(|| panic!("initial run should exist"));
        assert_eq!(initial_run.attributed_count, 5);
        assert_eq!(initial_run.excluded_count, 2);
""",
)
replace_once(
    pipeline,
    """        assert_eq!(second.papers.len(), 1);
        assert!(second.papers[0].stale_due_to_provider_failure);
        assert!(second.runs[1].changes.is_empty());
        assert_eq!(second.papers[0].last_seen, "2026-08-01");
""",
    """        assert_eq!(second.papers.len(), 1);
        let paper = second
            .papers
            .first()
            .unwrap_or_else(|| panic!("carried-forward paper should exist"));
        assert!(paper.stale_due_to_provider_failure);
        let second_run = second
            .runs
            .get(1)
            .unwrap_or_else(|| panic!("second run should exist"));
        assert!(second_run.changes.is_empty());
        assert_eq!(paper.last_seen, "2026-08-01");
""",
)

report = Path("crates/searchright-chhhs-demo/src/report.rs")
replace_once(
    report,
    '        state.papers[0].record.title = "A <script>alert(1)</script> paper".to_owned();\n',
    """        let paper = state
            .papers
            .first_mut()
            .unwrap_or_else(|| panic!("fixture paper should exist"));
        paper.record.title = "A <script>alert(1)</script> paper".to_owned();
""",
)
