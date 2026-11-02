use noyalib::{
    BudgetBreach, Error, ParserConfig, Value, from_str_borrowing_with_config, from_str_with_config,
    include::{IncludeResolver, InputSource},
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

// Single test process. Count requested live bytes; default realloc accounts for
// simultaneous old and new layouts. Ignore allocator bookkeeping and stack.
struct Meter;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static HIGH: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            HIGH.fetch_max(LIVE.fetch_add(l.size(), SeqCst) + l.size(), SeqCst);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), SeqCst);
        unsafe { System.dealloc(p, l) }
    }
}
#[global_allocator]
static M: Meter = Meter;
fn measure<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let base = LIVE.load(SeqCst);
    HIGH.store(base, SeqCst);
    let r = f();
    (r, HIGH.load(SeqCst) - base)
}
#[test]
fn bounded_allocations() {
    paired_replay();
    for merge_sequence in [false, true] {
        let mut peaks = Vec::new();
        for n in [1, 2048] {
            let aliases = if merge_sequence {
                format!("[{}]", vec!["*A"; n].join(","))
            } else {
                "*A".into()
            };
            let tail = if merge_sequence {
                String::new()
            } else {
                format!(", tail: [{}]", vec!["0"; n].join(","))
            };
            let yaml = format!("[&A {{a: 1}}, {{<<: {aliases}{tail}}}]");
            let cfg = ParserConfig::new().max_alias_expansions(0);
            let (r, peak) = measure(|| {
                from_str_borrowing_with_config::<Vec<BTreeMap<String, Value>>>(&yaml, &cfg)
            });
            assert!(matches!(r, Err(Error::RepetitionLimitExceeded)));
            println!(
                "reject seq={merge_sequence} n={n} input={} peak={peak}",
                yaml.len()
            );
            peaks.push(peak);
        }
        println!("growth in peak requested bytes: {}", peaks[1] - peaks[0]);
    }
    for n in [1, 4, 16] {
        let yaml = format!("[{}]", vec!["!include a"; n].join(","));
        let calls = std::sync::Arc::new(AtomicUsize::new(0));
        let ctr = calls.clone();
        let body = format!("[{}]", vec!["0"; 128].join(","));
        let cfg = ParserConfig::new()
            .max_nodes(130)
            .include_resolver(IncludeResolver::new(move |req| {
                ctr.fetch_add(1, SeqCst);
                Ok(InputSource::new(req.spec, body.clone()))
            }));
        let (r, peak) = measure(|| from_str_with_config::<Value>(&yaml, &cfg));
        println!(
            "include n={n} nodes={} calls={} peak={peak}",
            1 + n * 129,
            calls.load(SeqCst)
        );
        assert_eq!(calls.load(SeqCst), 1);
        if n == 1 {
            assert_eq!(r.unwrap()[0].as_sequence().unwrap().len(), 128)
        } else {
            assert!(matches!(
                r,
                Err(Error::Budget(BudgetBreach::MaxNodes {
                    limit: 130,
                    observed: 131
                }))
            ))
        }
    }
}

fn paired_replay() {
    let payload = "x".repeat(8192);
    let yaml = format!(
        "[&A {{a: '{payload}'}}, {{<<: [{}]}}]",
        vec!["*A"; 8].join(",")
    );
    for limit in [1, 8] {
        let cfg = ParserConfig::new()
            .max_alias_expansions(limit)
            .max_mapping_keys(1);
        let (result, peak) = measure(|| {
            from_str_borrowing_with_config::<Vec<BTreeMap<String, String>>>(&yaml, &cfg)
        });
        if limit == 1 {
            assert!(matches!(result, Err(Error::RepetitionLimitExceeded)))
        } else {
            assert_eq!(result.unwrap()[1]["a"], payload)
        }
        println!(
            "same input replay aliases=8 admitted={limit} input={} peak={peak}",
            yaml.len()
        );
    }
    for n in [2, 128] {
        let sources = (0..n)
            .map(|i| format!("{{k{i}: 0}}"))
            .collect::<Vec<_>>()
            .join(",");
        let input = format!("{{<<: [{sources}]}}");
        let (r, peak) = measure(|| {
            from_str_with_config::<Value>(&input, &ParserConfig::new().max_mapping_keys(1))
        });
        assert!(matches!(
            r,
            Err(Error::Budget(BudgetBreach::MaxMappingKeys {
                limit: 1,
                observed: 2
            }))
        ));
        println!(
            "width sources={n} limit=1 input={} peak={peak}",
            input.len()
        );
    }
}

#[test]
fn cumulative_siblings_and_fragment_selection() {
    for (limit, expected_calls, succeeds) in [(10, 3, true), (9, 3, false), (7, 2, false)] {
        let calls = std::sync::Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let cfg = ParserConfig::new()
            .max_nodes(limit)
            .include_resolver(IncludeResolver::new(move |req| {
                counter.fetch_add(1, SeqCst);
                Ok(InputSource::new(req.spec, "{part: [0, 1], extra: 2}"))
            }));
        let result = from_str_with_config::<Value>(
            "[!include a#part, !include b#part, !include c#part]",
            &cfg,
        );
        assert_eq!(calls.load(SeqCst), expected_calls);
        if succeeds {
            assert_eq!(result.unwrap()[2][1].as_u64(), Some(1));
        } else {
            assert!(
                matches!(result, Err(Error::Budget(BudgetBreach::MaxNodes {limit: l, observed})) if l == limit && observed == limit + 1),
                "{result:?}"
            );
        }
    }
}
