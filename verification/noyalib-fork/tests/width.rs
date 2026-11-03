use noyalib::{BudgetBreach, Error, ParserConfig, Value, from_str_with_config};

fn parse(yaml: &str, limit: usize) -> Result<Value, Error> {
    from_str_with_config(yaml, &ParserConfig::default().max_mapping_keys(limit))
}

fn boundary(yaml: &str) {
    assert_eq!(parse(yaml, 3).unwrap().as_mapping().unwrap().len(), 3);
    let result = parse(yaml, 2);
    assert!(matches!(result, Err(Error::Budget(BudgetBreach::MaxMappingKeys {
        limit: 2, observed: 3
    }))), "{result:?}");
}

#[test]
fn direct_merge_width() {
    boundary("{c: 3, <<: {a: 1, b: 2}}");
}

#[test]
fn sequence_merge_width() {
    boundary("{<<: [{a: 1, b: 2}, {c: 3}]}");
}

#[test]
fn duplicate_precedence_at_capacity() {
    for yaml in ["{a: 9, <<: [{a: 1, b: 2}, {b: 3}]}",
                 "{<<: [{a: 1, b: 2}, {b: 3}], a: 9}"] {
        let value = parse(yaml, 2).unwrap();
        assert_eq!(value.as_mapping().unwrap().len(), 2);
        assert_eq!(value["a"].as_u64(), Some(9));
        assert_eq!(value["b"].as_u64(), Some(2));
    }
}
