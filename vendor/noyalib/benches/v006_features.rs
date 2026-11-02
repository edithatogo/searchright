// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! v0.0.6 feature benchmarks — `recovery`, `sval`, `tokio`.
//!
//! Three bench groups:
//!
//! 1. **recovery_strict_vs_lenient** — `from_str` vs
//!    `recovery::parse_lenient` on valid input. The lenient
//!    path is a thin wrapper around `from_str_with_config` for
//!    valid inputs; the bench asserts the wrapper does not
//!    regress the strict throughput by more than rounding noise.
//! 2. **sval_vs_serde_value** — full deserialise into
//!    `noyalib::Value` (the serde route) vs streaming the same
//!    pre-built `Value` through a no-op `sval::Stream`. Shows
//!    the relative cost of the two reflection surfaces.
//! 3. **tokio_async_drain** — `tokio_async::from_async_reader`
//!    against an in-memory `BufReader` source vs the sync
//!    `from_slice` baseline. Quantifies the async-fixed-overhead
//!    cost when the I/O source has zero latency.
//!
//! Run with all three features:
//!     `cargo bench --bench v006_features --features recovery,sval,tokio`

#![allow(missing_docs, unused_results)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// ───────────────────────── Fixture ──────────────────────────

const SMALL: &str = "\
name: noyalib
version: 0.0.6
features:
  - recovery
  - sval
  - tokio
nested:
  a: 1
  b: 2.5
  c: true
  d: null
items:
  - one
  - two
  - three
";

// ───────────────────────── recovery ─────────────────────────

#[cfg(feature = "recovery")]
const INVALID: &str = "\
name: noyalib
version: 0.0.6
features:
  - recovery
  - sval
  - tokio
nested:
  a: 1
  b: 2.5
  c: true
  d: null
items:
  - one
  - two
  - [unclosed-flow
";

#[cfg(feature = "recovery")]
fn bench_recovery_strict_vs_lenient(c: &mut Criterion) {
    use noyalib::recovery::parse_lenient;
    use noyalib::{Value, from_str};

    let mut g = c.benchmark_group("recovery_strict_vs_lenient");
    g.bench_function("strict_from_str", |b| {
        b.iter(|| {
            let v: Value = from_str(black_box(SMALL)).unwrap();
            black_box(v);
        });
    });
    g.bench_function("lenient_parse_lenient_valid", |b| {
        b.iter(|| {
            let r = parse_lenient(black_box(SMALL));
            black_box(r);
        });
    });
    // The recovery surface's actual value proposition is on
    // malformed input. This arm benches the 3-pass recovery loop
    // on a realistic LSP-style half-typed buffer.
    g.bench_function("lenient_parse_lenient_invalid", |b| {
        b.iter(|| {
            let r = parse_lenient(black_box(INVALID));
            black_box(r);
        });
    });
    g.finish();
}

#[cfg(not(feature = "recovery"))]
fn bench_recovery_strict_vs_lenient(_c: &mut Criterion) {}

// ────────────────────────── sval ────────────────────────────

#[cfg(feature = "sval")]
fn bench_sval_vs_serde_value(c: &mut Criterion) {
    use noyalib::{Value, from_str};

    let value: Value = from_str(SMALL).unwrap();

    let mut g = c.benchmark_group("sval_vs_serde_value");
    g.bench_function("serde_from_str_to_value", |b| {
        b.iter(|| {
            let v: Value = from_str(black_box(SMALL)).unwrap();
            black_box(v);
        });
    });
    g.bench_function("sval_stream_prebuilt_value", |b| {
        let mut sink = NullStream::default();
        b.iter(|| {
            sval::Value::stream(black_box(&value), &mut sink).unwrap();
            sink.count = 0;
        });
    });
    g.finish();
}

#[cfg(not(feature = "sval"))]
fn bench_sval_vs_serde_value(_c: &mut Criterion) {}

#[cfg(feature = "sval")]
#[derive(Default)]
struct NullStream {
    count: u64,
}

#[cfg(feature = "sval")]
impl sval::Stream<'_> for NullStream {
    fn null(&mut self) -> sval::Result {
        self.count += 1;
        Ok(())
    }
    fn bool(&mut self, _: bool) -> sval::Result {
        self.count += 1;
        Ok(())
    }
    fn i64(&mut self, _: i64) -> sval::Result {
        self.count += 1;
        Ok(())
    }
    fn f64(&mut self, _: f64) -> sval::Result {
        self.count += 1;
        Ok(())
    }
    fn text_begin(&mut self, _: Option<usize>) -> sval::Result {
        Ok(())
    }
    fn text_fragment_computed(&mut self, _: &str) -> sval::Result {
        self.count += 1;
        Ok(())
    }
    fn text_end(&mut self) -> sval::Result {
        Ok(())
    }
    fn map_begin(&mut self, _: Option<usize>) -> sval::Result {
        Ok(())
    }
    fn map_end(&mut self) -> sval::Result {
        Ok(())
    }
    fn map_key_begin(&mut self) -> sval::Result {
        Ok(())
    }
    fn map_key_end(&mut self) -> sval::Result {
        Ok(())
    }
    fn map_value_begin(&mut self) -> sval::Result {
        Ok(())
    }
    fn map_value_end(&mut self) -> sval::Result {
        Ok(())
    }
    fn seq_begin(&mut self, _: Option<usize>) -> sval::Result {
        Ok(())
    }
    fn seq_end(&mut self) -> sval::Result {
        Ok(())
    }
    fn seq_value_begin(&mut self) -> sval::Result {
        Ok(())
    }
    fn seq_value_end(&mut self) -> sval::Result {
        Ok(())
    }
}

// ────────────────────────── tokio ───────────────────────────

#[cfg(feature = "tokio")]
fn bench_tokio_async_drain(c: &mut Criterion) {
    use noyalib::Value;
    use tokio::runtime::Builder;

    let rt = Builder::new_current_thread().build().unwrap();

    let mut g = c.benchmark_group("tokio_async_drain");
    g.bench_function("sync_from_slice", |b| {
        b.iter(|| {
            let v: Value = noyalib::from_slice(black_box(SMALL.as_bytes())).unwrap();
            black_box(v);
        });
    });
    g.bench_function("async_from_async_reader", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut reader = tokio::io::BufReader::new(SMALL.as_bytes());
                let v: Value = noyalib::tokio_async::from_async_reader(&mut reader)
                    .await
                    .unwrap();
                black_box(v);
            });
        });
    });
    g.finish();
}

#[cfg(not(feature = "tokio"))]
fn bench_tokio_async_drain(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_recovery_strict_vs_lenient,
    bench_sval_vs_serde_value,
    bench_tokio_async_drain
);
criterion_main!(benches);
