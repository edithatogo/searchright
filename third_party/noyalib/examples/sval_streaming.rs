// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `sval_adapter` — stream a `noyalib::Value` graph through any
//! `sval::Stream` consumer.
//!
//! The bundled `Recorder` is a minimal `sval::Stream` impl that
//! captures every event as a human-readable line so you can see
//! the shape of the stream sval consumers receive.
//!
//! Run: `cargo run --example sval_streaming --features sval`

#[cfg(feature = "sval")]
fn main() {
    use noyalib::{Value, from_str};

    let yaml = "\
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
";

    let value: Value = from_str(yaml).expect("parse");

    let mut recorder = Recorder::default();
    sval::Value::stream(&value, &mut recorder).expect("stream");

    println!("sval event stream (one per line):");
    for line in &recorder.events {
        println!("  {line}");
    }
    println!("\ntotal events: {}", recorder.events.len());

    // ── Pattern: non-finite float coercion ──────────────────────
    //
    // `.nan` / `.inf` literals are valid YAML floats but `sval_json`
    // and similar JSON-bound consumers reject non-finites. Use
    // `SvalConfig::coerce_non_finite_to_null` to emit `Null`
    // instead — required for round-tripping into JSON.
    use noyalib::Number;
    use noyalib::sval_adapter::{SvalConfig, to_sval_writer_with_config};

    let nan_value = Value::Number(Number::Float(f64::NAN));
    let cfg = SvalConfig {
        coerce_non_finite_to_null: true,
    };

    let mut nan_recorder = Recorder::default();
    to_sval_writer_with_config(&mut nan_recorder, &nan_value, &cfg).expect("stream NaN");
    println!("\nNaN with coerce_non_finite_to_null = true:");
    for line in &nan_recorder.events {
        println!("  {line}");
    }
}

#[cfg(feature = "sval")]
#[derive(Default)]
struct Recorder {
    events: Vec<String>,
}

#[cfg(feature = "sval")]
impl sval::Stream<'_> for Recorder {
    fn null(&mut self) -> sval::Result {
        self.events.push("null".into());
        Ok(())
    }
    fn bool(&mut self, v: bool) -> sval::Result {
        self.events.push(format!("bool({v})"));
        Ok(())
    }
    fn i64(&mut self, v: i64) -> sval::Result {
        self.events.push(format!("i64({v})"));
        Ok(())
    }
    fn f64(&mut self, v: f64) -> sval::Result {
        self.events.push(format!("f64({v})"));
        Ok(())
    }
    fn text_begin(&mut self, _: Option<usize>) -> sval::Result {
        self.events.push("text_begin".into());
        Ok(())
    }
    fn text_fragment_computed(&mut self, fragment: &str) -> sval::Result {
        self.events.push(format!("text({fragment:?})"));
        Ok(())
    }
    fn text_end(&mut self) -> sval::Result {
        self.events.push("text_end".into());
        Ok(())
    }
    fn map_begin(&mut self, n: Option<usize>) -> sval::Result {
        self.events.push(format!("map_begin({n:?})"));
        Ok(())
    }
    fn map_end(&mut self) -> sval::Result {
        self.events.push("map_end".into());
        Ok(())
    }
    fn map_key_begin(&mut self) -> sval::Result {
        self.events.push("map_key_begin".into());
        Ok(())
    }
    fn map_key_end(&mut self) -> sval::Result {
        self.events.push("map_key_end".into());
        Ok(())
    }
    fn map_value_begin(&mut self) -> sval::Result {
        self.events.push("map_value_begin".into());
        Ok(())
    }
    fn map_value_end(&mut self) -> sval::Result {
        self.events.push("map_value_end".into());
        Ok(())
    }
    fn seq_begin(&mut self, n: Option<usize>) -> sval::Result {
        self.events.push(format!("seq_begin({n:?})"));
        Ok(())
    }
    fn seq_end(&mut self) -> sval::Result {
        self.events.push("seq_end".into());
        Ok(())
    }
    fn seq_value_begin(&mut self) -> sval::Result {
        self.events.push("seq_value_begin".into());
        Ok(())
    }
    fn seq_value_end(&mut self) -> sval::Result {
        self.events.push("seq_value_end".into());
        Ok(())
    }
}

#[cfg(not(feature = "sval"))]
fn main() {
    eprintln!("This example requires the `sval` feature.");
    eprintln!("Run with: cargo run --example sval_streaming --features sval");
}
