use noyalib::{Error, ParserConfig, from_str_borrowing_with_config};
use std::collections::BTreeMap;
type Document = BTreeMap<String, BTreeMap<String, u32>>;

fn parse(yaml: &str, limit: usize) -> Result<Document, Error> {
    from_str_borrowing_with_config(yaml, &ParserConfig::default().max_alias_expansions(limit))
}

#[test]
fn direct_merge() {
    let yaml = "base: &base {a: 1}\ncopy: {<<: *base}\n";
    assert_eq!(parse(yaml, 1).unwrap()["copy"]["a"], 1);
    assert!(matches!(parse(yaml, 0), Err(Error::RepetitionLimitExceeded)));
}

#[test]
fn sequence_merge() {
    let yaml = "base: &base {a: 1}\ncopy: {<<: [*base, *base]}\n";
    assert_eq!(parse(yaml, 2).unwrap()["copy"]["a"], 1);
    assert!(matches!(parse(yaml, 1), Err(Error::RepetitionLimitExceeded)));
}
