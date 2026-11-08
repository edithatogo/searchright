use noyalib::{
    BudgetBreach, Error, ParserConfig, Value, from_str_borrowing_with_config, from_str_with_config,
};
use std::collections::BTreeMap;

#[test]
fn both_limits_and_precedence() {
    for (merge, aliases) in [("*A", 1), ("[*A, *B]", 2)] {
        for destination in [
            format!("{{<<: {merge}, a: 9, c: 4}}"),
            format!("{{a: 9, c: 4, <<: {merge}}}"),
        ] {
            let yaml = format!("[&A {{a: 1, b: 2}}, &B {{b: 3}}, {destination}]");
            for route in ["value", "owned", "borrowed"] {
                for (a, w) in [
                    (aliases, 3),
                    (aliases - 1, 3),
                    (aliases, 2),
                    (aliases - 1, 2),
                ] {
                    let cfg = ParserConfig::new()
                        .max_alias_expansions(a)
                        .max_mapping_keys(w);
                    let result = if route != "value" {
                        if route == "owned" {
                            from_str_with_config::<Vec<BTreeMap<String, u64>>>(&yaml, &cfg)
                        } else {
                            from_str_borrowing_with_config::<Vec<BTreeMap<String, u64>>>(
                                &yaml, &cfg,
                            )
                        }
                        .map(|v| (v[2].len(), v[2]["a"], v[2]["b"], v[2]["c"]))
                    } else {
                        from_str_with_config::<Value>(&yaml, &cfg).map(|v| {
                            let m = &v.as_sequence().unwrap()[2];
                            (
                                m.as_mapping().unwrap().len(),
                                m["a"].as_u64().unwrap(),
                                m["b"].as_u64().unwrap(),
                                m["c"].as_u64().unwrap(),
                            )
                        })
                    };
                    eprintln!("route={route} a={a} w={w} input={destination} => {result:?}");
                    if route == "borrowed" && destination.starts_with("{a:") {
                        assert!(
                            matches!(result,Err(Error::Custom(ref s)) if s == "$__noyalib_streaming_fallback")
                        );
                    } else if a < aliases {
                        assert!(matches!(result, Err(Error::RepetitionLimitExceeded)));
                    } else if w < 3 {
                        assert!(matches!(
                            result,
                            Err(Error::Budget(BudgetBreach::MaxMappingKeys {
                                limit: 2,
                                observed: 3
                            }))
                        ));
                    } else {
                        assert_eq!(result.unwrap(), (3, 9, 2, 4));
                    }
                }
            }
        }
    }
}

#[test]
fn submitted_yaml_policy_preempts_borrowing_fallback() {
    let cfg = ParserConfig::default()
        .max_merge_keys(0)
        .with_policy(noyalib::policy::DenyAnchors);
    for source in ["{a: 1, <<: {b: 2}}", "{<<: {b: 2}, a: 1}"] {
        let result = from_str_borrowing_with_config::<BTreeMap<String, u64>>(source, &cfg);
        assert!(
            matches!(
                result,
                Err(Error::Budget(BudgetBreach::MaxMergeKeys {
                    limit: 0,
                    observed: 1
                }))
            ),
            "{result:?}"
        );
    }
    let result = from_str_borrowing_with_config::<Vec<BTreeMap<String, u64>>>(
        "[&A {b: 2}, {a: 1, <<: *A}]",
        &cfg,
    );
    assert!(
        matches!(result, Err(Error::Deserialize(ref s)) if s.contains("DenyAnchors")),
        "{result:?}"
    );
}
