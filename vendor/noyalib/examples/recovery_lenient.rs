// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `recovery::parse_lenient` — best-effort parsing for LSP / IDE
//! partial-document scenarios.
//!
//! Run with: `cargo run --example recovery_lenient --features recovery`

#[cfg(feature = "recovery")]
fn main() {
    use noyalib::recovery::{LenientConfig, parse_lenient, parse_lenient_with};

    // ── Pattern 1: default knobs on a half-typed buffer ──
    let half_typed = "\
config:
  name: noyalib
  version: 0.0.6
  features: [recovery, std
  # ^^ user is mid-typing — the flow sequence is unclosed
";

    let result = parse_lenient(half_typed);

    println!("Pattern 1 — parse_lenient(default):");
    println!("  is_complete: {}", result.is_complete);
    println!("  errors:      {}", result.errors.len());
    for (i, err) in result.errors.iter().enumerate() {
        println!("    [{i}] {err}");
    }

    // ── Pattern 2: tighten the budget for an adversarial input ──
    //
    // `truncation_event_budget` caps the cumulative byte cost of
    // line-truncation retries on malformed input. The default
    // (1 MiB) is suited to LSP-edit buffers; lower it for hostile
    // input where every retry costs CPU.
    let cfg = LenientConfig {
        truncation_event_budget: 1024, // 1 KiB cap
        ..LenientConfig::default()
    };
    let mut adversarial = String::from("a: 1\n");
    for _ in 0..500 {
        adversarial.push_str("[malformed-flow\n");
    }
    let bounded = parse_lenient_with(&adversarial, &cfg);
    println!("\nPattern 2 — parse_lenient_with(truncation_event_budget = 1 KiB):");
    println!(
        "  is_complete: {} (expected false — 500 broken lines)",
        bounded.is_complete
    );
    println!("  errors:      {}", bounded.errors.len());

    // ── Pattern 3: pnpm-lock.yaml shape (issue #46 fix verifies) ──
    //
    // Wide-but-shallow `pnpm-lock`-shaped input with N empty flow
    // mappings used to fail `from_str::<Value>` with
    // RecursionLimitExceeded at exactly N = max_depth. v0.0.6
    // closes that bug; recovery now parses it cleanly too.
    let mut lockfile = String::from("packages:\n");
    for i in 0..300 {
        let _ =
            std::fmt::Write::write_fmt(&mut lockfile, format_args!("  pkg-{i}@1.0.{i}: {{}}\n"));
    }
    let pnpm = parse_lenient(&lockfile);
    println!("\nPattern 3 — 300-package pnpm-lock-shaped input:");
    println!(
        "  is_complete: {} (was: RecursionLimitExceeded pre-v0.0.6)",
        pnpm.is_complete
    );
    println!("  errors:      {}", pnpm.errors.len());
}

#[cfg(not(feature = "recovery"))]
fn main() {
    eprintln!("This example requires the `recovery` feature.");
    eprintln!("Run with: cargo run --example recovery_lenient --features recovery");
}
