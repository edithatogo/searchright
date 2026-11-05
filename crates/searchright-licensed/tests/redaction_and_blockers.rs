//! Licensed request planning edge cases.

use searchright_contracts::{CompiledStrategy, LicensedAdapterProfile};
use searchright_licensed::{LicensedError, plan_licensed_request, plan_request};

fn inputs() -> (LicensedAdapterProfile, CompiledStrategy) {
    let profile: LicensedAdapterProfile = noyalib::from_str(include_str!(
        "../../../contracts/examples/licensed-adapter.yaml"
    ))
    .unwrap_or_else(|_| panic!("fixture must parse"));
    let mut strategy: CompiledStrategy = noyalib::from_str(include_str!(
        "../../../contracts/examples/compiled-strategy.yaml"
    ))
    .unwrap_or_else(|_| panic!("fixture must parse"));
    strategy.dialect = profile.dialect.clone();
    (profile, strategy)
}

#[test]
fn malformed_url_propagates_parse_error() {
    let (profile, strategy) = inputs();
    assert!(matches!(
        plan_request(&profile, &strategy, "not a url"),
        Err(LicensedError::Url(_))
    ));
}

#[test]
fn username_only_url_is_credential_bearing() {
    let (profile, strategy) = inputs();
    assert!(matches!(
        plan_request(&profile, &strategy, "https://api-user@embase.com/search"),
        Err(LicensedError::CredentialBearingEndpoint)
    ));
}

#[test]
fn ftp_scheme_is_insecure() {
    let (profile, strategy) = inputs();
    assert!(matches!(
        plan_request(&profile, &strategy, "ftp://embase.com/search"),
        Err(LicensedError::InsecureEndpoint)
    ));
}

#[test]
fn uppercase_host_is_normalised() {
    let (profile, strategy) = inputs();
    let plan = plan_request(&profile, &strategy, "https://EMBASE.COM/search?q=x")
        .unwrap_or_else(|_| panic!("expected plan"));
    assert_eq!(plan.endpoint, "https://embase.com");
}

#[test]
fn path_query_fragment_do_not_leak_into_origin() {
    let (profile, strategy) = inputs();
    let plan = plan_request(&profile, &strategy, "https://embase.com/a/b?secret=1#frag")
        .unwrap_or_else(|_| panic!("expected plan"));
    assert_eq!(plan.endpoint, "https://embase.com");
    let serialized = serde_json::to_string(&plan).unwrap_or_else(|_| panic!("expected json"));
    assert!(!serialized.contains("secret=1"));
    assert!(!serialized.contains("frag"));
}

#[test]
fn port_is_preserved_in_origin() {
    let (profile, strategy) = inputs();
    let plan = plan_request(&profile, &strategy, "https://embase.com:8443/search")
        .unwrap_or_else(|_| panic!("expected plan"));
    assert_eq!(plan.endpoint, "https://embase.com:8443");
}

#[test]
fn invalid_profile_schema_propagates_contract_error() {
    let (mut profile, strategy) = inputs();
    profile.schema_version = "org.searchright.licensed-adapter.v99".to_string();
    assert!(matches!(
        plan_request(&profile, &strategy, "https://embase.com/search"),
        Err(LicensedError::Contract(_))
    ));
}

#[test]
fn empty_host_url_fails_with_missing_or_parse_error() {
    let (profile, strategy) = inputs();
    let result = plan_request(&profile, &strategy, "https:///search");
    assert!(result.is_err());
}

#[test]
fn compiled_strategy_dialect_matches_profile() {
    let (profile, strategy) = inputs();
    assert_eq!(strategy.dialect, profile.dialect);
}

#[test]
fn alias_produces_identical_plan_to_primary() {
    let (profile, strategy) = inputs();
    let first = plan_request(&profile, &strategy, "https://embase.com/search")
        .unwrap_or_else(|_| panic!("expected plan"));
    let second = plan_licensed_request(&profile, &strategy, "https://embase.com/search")
        .unwrap_or_else(|_| panic!("expected plan"));
    assert_eq!(first, second);
}

#[test]
fn plan_carries_provider_database_platform_hash() {
    let (profile, strategy) = inputs();
    let plan = plan_request(&profile, &strategy, "https://embase.com/search")
        .unwrap_or_else(|_| panic!("expected plan"));
    assert_eq!(plan.provider_id, "embase-byo");
    assert_eq!(plan.database, "Embase");
    assert_eq!(plan.platform, "Embase.com");
    assert_eq!(plan.compilation_hash, strategy.compilation_hash);
}

#[test]
fn blockers_wording_names_the_missing_environment() {
    let (profile, strategy) = inputs();
    let plan = plan_request(&profile, &strategy, "https://embase.com/search")
        .unwrap_or_else(|_| panic!("expected plan"));
    // In this test environment credentials are absent, so at least the
    // credential blocker is expected unless the CI image sets it.
    let credential_blocker_present = plan
        .blockers
        .iter()
        .any(|b| b.contains(&profile.credential_environment_variable));
    let live_blocker_present = plan
        .blockers
        .iter()
        .any(|b| b.contains(&profile.live_opt_in_environment_variable));
    assert!(credential_blocker_present || plan.credential_present);
    assert!(live_blocker_present || plan.live_opt_in);
}
