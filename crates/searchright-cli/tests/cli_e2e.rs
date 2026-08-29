//! Executable CLI snapshots shared by every hosted operating-system runner.

#![forbid(unsafe_code)]

use std::{
    fs,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_searchright"))
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("searchright must execute: {error}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

fn stable_help_text(bytes: &[u8]) -> String {
    text(bytes).replace("Usage: searchright.exe ", "Usage: searchright ")
}

#[test]
fn help_output_matches_the_cross_platform_snapshot() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    assert_eq!(
        stable_help_text(&output.stdout),
        include_str!("snapshots/help.txt")
    );
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
fn review_previews_do_not_create_the_local_store() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| panic!("system clock must follow the epoch: {error}"))
        .as_nanos();
    let store = std::env::temp_dir().join(format!(
        "searchright-cli-preview-{}-{unique}",
        std::process::id()
    ));
    let store_argument = store.to_string_lossy().into_owned();
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root must contain the CLI crate"));

    for (command, example) in [
        ("plan-review", "contracts/examples/review-plan.yaml"),
        (
            "press-review-strategy",
            "contracts/examples/press-review.json",
        ),
    ] {
        let input = repository.join(example);
        let input_argument = input.to_string_lossy();
        let output = run(&[command, &input_argument, "--store", &store_argument]);
        assert!(
            output.status.success(),
            "{command} failed: {}",
            text(&output.stderr)
        );
        assert!(!store.exists(), "{command} preview created the local store");
    }
}

#[test]
fn apply_creates_once_and_refusal_preserves_exact_bytes() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| panic!("system clock must follow the epoch: {error}"))
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "searchright-cli-e2e-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&directory)
        .unwrap_or_else(|error| panic!("temporary directory must be created: {error}"));
    let target = directory.join("config.json");
    let target_argument = target.to_string_lossy().into_owned();

    let apply = run(&["init", "--target", &target_argument, "--apply"]);
    assert!(apply.status.success());
    let applied: serde_json::Value = serde_json::from_slice(&apply.stdout)
        .unwrap_or_else(|error| panic!("apply output must be JSON: {error}"));
    assert_eq!(applied.get("mode"), Some(&serde_json::json!("apply")));
    assert_eq!(applied.get("changed"), Some(&serde_json::json!(true)));
    let original = fs::read(&target)
        .unwrap_or_else(|error| panic!("configuration must have been written: {error}"));

    let refusal = run(&["init", "--target", &target_argument, "--apply"]);
    assert_eq!(refusal.status.code(), Some(3));
    assert!(refusal.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&refusal.stderr)
        .unwrap_or_else(|decode_error| panic!("refusal must be JSON: {decode_error}"));
    assert_eq!(
        error.get("code"),
        Some(&serde_json::json!("cli.filesystem"))
    );
    assert_eq!(error.get("stage"), Some(&serde_json::json!("init")));
    assert_eq!(
        error.get("category"),
        Some(&serde_json::json!("filesystem"))
    );
    assert_eq!(
        fs::read(&target).unwrap_or_else(|read_error| panic!(
            "configuration must remain readable: {read_error}"
        )),
        original
    );
    fs::remove_dir_all(directory)
        .unwrap_or_else(|error| panic!("temporary directory must be removed: {error}"));
}

#[test]
fn operation_errors_name_a_safe_stage_and_category_without_reflecting_paths() {
    let secret = "TRACK09_PATH_SENTINEL_SECRET";
    let output = run(&["validate-plan", secret]);
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let stderr = text(&output.stderr);
    assert!(!stderr.contains(secret));
    let error: serde_json::Value = serde_json::from_str(&stderr)
        .unwrap_or_else(|decode_error| panic!("operation error must be JSON: {decode_error}"));
    assert_eq!(
        error.get("stage"),
        Some(&serde_json::json!("validate-plan"))
    );
    assert_eq!(
        error.get("category"),
        Some(&serde_json::json!("filesystem"))
    );
}

#[test]
fn operation_errors_distinguish_syntax_from_contract_failures() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| panic!("system clock must follow the epoch: {error}"))
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "searchright-cli-errors-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&directory)
        .unwrap_or_else(|error| panic!("temporary directory must be created: {error}"));
    let syntax_path = directory.join("syntax.json");
    let contract_path = directory.join("contract.json");
    fs::write(&syntax_path, "{")
        .unwrap_or_else(|error| panic!("syntax fixture must be written: {error}"));
    fs::write(&contract_path, "{}")
        .unwrap_or_else(|error| panic!("contract fixture must be written: {error}"));

    for (path, expected_category) in [
        (&syntax_path, "document_syntax"),
        (&contract_path, "document_contract"),
    ] {
        let argument = path.to_string_lossy();
        let output = run(&["validate-plan", &argument]);
        assert_eq!(output.status.code(), Some(3));
        let error: serde_json::Value = serde_json::from_slice(&output.stderr)
            .unwrap_or_else(|decode_error| panic!("operation error must be JSON: {decode_error}"));
        assert_eq!(
            error.get("stage"),
            Some(&serde_json::json!("validate-plan"))
        );
        assert_eq!(
            error.get("category"),
            Some(&serde_json::json!(expected_category))
        );
    }

    fs::remove_dir_all(directory)
        .unwrap_or_else(|error| panic!("temporary directory must be removed: {error}"));
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
    for (shell, marker) in [
        ("bash", "_searchright()"),
        ("elvish", "edit:completion:arg-completer[searchright]"),
        ("fish", "__fish_searchright_global_optspecs"),
        (
            "powershell",
            "Register-ArgumentCompleter -Native -CommandName 'searchright'",
        ),
        ("zsh", "#compdef searchright"),
    ] {
        let completions = run(&["completions", shell]);
        assert!(completions.status.success());
        assert!(text(&completions.stdout).contains(marker));
    }

    let manpage = run(&["manpage"]);
    assert!(manpage.status.success());
    let manpage = text(&manpage.stdout);
    assert!(manpage.contains(".TH searchright 1"));
    assert!(manpage.contains("searchright\\-completions(1)"));
}
