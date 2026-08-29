//! Executable CLI snapshots shared by every hosted operating-system runner.

#![forbid(unsafe_code)]

use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_searchright"))
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("searchright must execute: {error}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

#[test]
fn help_output_matches_the_cross_platform_snapshot() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    assert_eq!(text(&output.stdout), include_str!("snapshots/help.txt"));
    assert!(output.stderr.is_empty());
}

#[test]
fn dry_run_json_matches_the_stable_snapshot_without_writing() {
    let output = run(&["init", "--target", "snapshot.json"]);
    assert!(output.status.success());
    assert_eq!(text(&output.stdout), include_str!("snapshots/init.json"));
    assert!(output.stderr.is_empty());
}

#[test]
fn usage_errors_are_machine_readable_and_stable() {
    let output = run(&["invalid-command"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let actual: serde_json::Value = serde_json::from_slice(&output.stderr)
        .unwrap_or_else(|error| panic!("usage error must be JSON: {error}"));
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("snapshots/usage-error.json"))
            .unwrap_or_else(|error| panic!("usage snapshot must be JSON: {error}"));
    assert_eq!(actual, expected);
}

#[test]
fn distribution_documents_are_available_without_filesystem_writes() {
    let completions = run(&["completions", "bash"]);
    assert!(completions.status.success());
    assert!(text(&completions.stdout).contains("_searchright"));

    let manpage = run(&["manpage"]);
    assert!(manpage.status.success());
    let manpage = text(&manpage.stdout);
    assert!(manpage.contains(".TH searchright 1"));
    assert!(manpage.contains("searchright\\-completions(1)"));
}
