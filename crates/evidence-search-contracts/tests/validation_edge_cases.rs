//! Edge-case tests for contract validation invariants.
//!
//! Covers query, date-limit, search-strategy, execution-policy and
//! provider-page validation paths not exercised by the named-filter suite.

use evidence_search_contracts::{
    AUDIT_EVENT_SCHEMA_VERSION, Actor, AuditEvent, AuditEventDraft, ContractError, DateLimit,
    ExecutionPolicy, PROVIDER_PAGE_SCHEMA_VERSION, ProviderPage, QueryExpr,
    SEARCH_STRATEGY_SCHEMA_VERSION, SOURCE_RECEIPT_SCHEMA_VERSION, SearchDialect, SearchField,
    SearchLimit, SearchStrategy, SearchTerm, SourceReceipt, Validate,
};

fn term(text: &str, fields: Vec<SearchField>) -> SearchTerm {
    SearchTerm {
        text: text.to_string(),
        fields,
        vocabulary: None,
        explode: false,
        phrase: false,
        truncation: false,
    }
}

fn strategy(query: QueryExpr, limits: SearchLimit) -> SearchStrategy {
    SearchStrategy {
        schema_version: SEARCH_STRATEGY_SCHEMA_VERSION.to_string(),
        strategy_id: "s1".to_string(),
        review_id: "r1".to_string(),
        source_id: "src1".to_string(),
        dialect: SearchDialect::PubMed,
        query,
        limits,
        translated_from: None,
        notes: vec![],
    }
}

// --- SearchTerm ---

#[test]
fn term_empty_text_rejected() {
    let t = term("", vec![]);
    assert_eq!(
        t.validate(),
        Err(ContractError::EmptyField("query.term.text"))
    );
}

#[test]
fn term_whitespace_text_rejected() {
    let t = term("   ", vec![]);
    assert_eq!(
        t.validate(),
        Err(ContractError::EmptyField("query.term.text"))
    );
}

#[test]
fn term_control_characters_rejected() {
    let t = term("cancer\n", vec![]);
    assert_eq!(
        t.validate(),
        Err(ContractError::Invariant(
            "search terms must not contain control characters".to_string()
        ))
    );
}

#[test]
fn term_duplicate_fields_rejected() {
    let t = term("cancer", vec![SearchField::Title, SearchField::Title]);
    assert_eq!(
        t.validate(),
        Err(ContractError::Invariant(
            "search term fields must not contain duplicates".to_string()
        ))
    );
}

#[test]
fn term_custom_field_invalid_characters_rejected() {
    let t = term(
        "cancer",
        vec![SearchField::Custom("bad field!".to_string())],
    );
    assert_eq!(
        t.validate(),
        Err(ContractError::Invariant(
            "custom field names may contain only ASCII letters, digits, `_`, `-`, `.`, or `/`"
                .to_string()
        ))
    );
}

#[test]
fn term_explode_without_vocabulary_rejected() {
    let t = SearchTerm {
        text: "neoplasm".to_string(),
        fields: vec![SearchField::Title],
        vocabulary: None,
        explode: true,
        phrase: false,
        truncation: false,
    };
    assert_eq!(
        t.validate(),
        Err(ContractError::Invariant(
            "explode may be requested only for a controlled-vocabulary term".to_string()
        ))
    );
}

#[test]
fn term_vocabulary_with_non_subject_field_rejected() {
    let t = SearchTerm {
        text: "neoplasm".to_string(),
        fields: vec![SearchField::Title],
        vocabulary: Some("MeSH".to_string()),
        explode: true,
        phrase: false,
        truncation: false,
    };
    assert_eq!(
        t.validate(),
        Err(ContractError::Invariant(
            "controlled-vocabulary terms may target only the subject-heading field".to_string()
        ))
    );
}

#[test]
fn term_valid_subject_heading_explode_ok() {
    let t = SearchTerm {
        text: "neoplasm".to_string(),
        fields: vec![SearchField::SubjectHeading],
        vocabulary: Some("MeSH".to_string()),
        explode: true,
        phrase: false,
        truncation: false,
    };
    assert_eq!(t.validate(), Ok(()));
}

// --- QueryExpr ---

#[test]
fn and_with_single_child_rejected() {
    let q = QueryExpr::And {
        children: vec![QueryExpr::Term {
            term: term("cancer", vec![]),
        }],
    };
    assert_eq!(
        q.validate(),
        Err(ContractError::Invariant(
            "and/or expressions require at least two children".to_string()
        ))
    );
}

#[test]
fn or_with_zero_children_rejected() {
    let q = QueryExpr::Or { children: vec![] };
    assert_eq!(
        q.validate(),
        Err(ContractError::Invariant(
            "and/or expressions require at least two children".to_string()
        ))
    );
}

#[test]
fn proximity_zero_distance_rejected() {
    let q = QueryExpr::Proximity {
        left: Box::new(QueryExpr::Term {
            term: term("lung", vec![]),
        }),
        right: Box::new(QueryExpr::Term {
            term: term("cancer", vec![]),
        }),
        distance: 0,
        ordered: false,
    };
    assert_eq!(
        q.validate(),
        Err(ContractError::Invariant(
            "proximity distance must be greater than zero".to_string()
        ))
    );
}

#[test]
fn nested_and_or_valid() {
    let leaf = || QueryExpr::Term {
        term: term("cancer", vec![]),
    };
    let q = QueryExpr::And {
        children: vec![
            QueryExpr::Or {
                children: vec![leaf(), leaf()],
            },
            QueryExpr::Not {
                include: Box::new(leaf()),
                exclude: Box::new(leaf()),
            },
        ],
    };
    assert_eq!(q.validate(), Ok(()));
}

// --- DateLimit ---

#[test]
fn date_limit_both_none_rejected() {
    let d = DateLimit {
        from_year: None,
        to_year: None,
    };
    assert_eq!(
        d.validate(),
        Err(ContractError::Invariant(
            "publication-date limits require at least one boundary".to_string()
        ))
    );
}

#[test]
fn date_limit_inverted_bounds_rejected() {
    let d = DateLimit {
        from_year: Some(2025),
        to_year: Some(2020),
    };
    assert_eq!(
        d.validate(),
        Err(ContractError::Invariant(
            "publication-date lower bound must not exceed the upper bound".to_string()
        ))
    );
}

#[test]
fn date_limit_valid_bounds_ok() {
    let d = DateLimit {
        from_year: Some(2020),
        to_year: Some(2025),
    };
    assert_eq!(d.validate(), Ok(()));
}

#[test]
fn date_limit_single_bound_ok() {
    let d = DateLimit {
        from_year: Some(2020),
        to_year: None,
    };
    assert_eq!(d.validate(), Ok(()));
}

// --- SearchStrategy ---

#[test]
fn strategy_restrictions_require_rationale() {
    let s = strategy(
        QueryExpr::Term {
            term: term("cancer", vec![]),
        },
        SearchLimit {
            languages: vec!["en".to_string()],
            publication_types: vec![],
            filters: vec![],
            rationale: vec![],
            publication_date: None,
        },
    );
    assert_eq!(
        s.validate(),
        Err(ContractError::Invariant(
            "search restrictions and filters require an explicit rationale".to_string()
        ))
    );
}

#[test]
fn strategy_custom_dialect_requires_name() {
    let mut s = strategy(
        QueryExpr::Term {
            term: term("cancer", vec![]),
        },
        SearchLimit::default(),
    );
    s.dialect = SearchDialect::Custom(String::new());
    assert_eq!(
        s.validate(),
        Err(ContractError::EmptyField("dialect.custom"))
    );
}

// --- ExecutionPolicy ---

const fn policy() -> ExecutionPolicy {
    ExecutionPolicy {
        live_enabled: true,
        max_records: 100,
        max_pages: 5,
        timeout_seconds: 30,
        total_timeout_seconds: Some(300),
        max_retries: 2,
        min_interval_ms: 350,
        retry_base_delay_ms: Some(350),
        retry_max_delay_ms: Some(5600),
        max_response_bytes: Some(16_777_216),
        replay_enabled: true,
        cache_write_enabled: false,
    }
}

#[test]
fn policy_zero_budgets_rejected() {
    let mut p = policy();
    p.max_records = 0;
    assert_eq!(
        p.validate(),
        Err(ContractError::Invariant(
            "execution budgets and timeout must be greater than zero".to_string()
        ))
    );
}

#[test]
fn policy_total_less_than_per_request_rejected() {
    let mut p = policy();
    p.total_timeout_seconds = Some(10);
    assert_eq!(
        p.validate(),
        Err(ContractError::Invariant(
            "total_timeout_seconds must be at least the per-request timeout".to_string()
        ))
    );
}

#[test]
fn policy_retry_inverted_rejected() {
    let mut p = policy();
    p.retry_base_delay_ms = Some(1000);
    p.retry_max_delay_ms = Some(100);
    assert_eq!(
        p.validate(),
        Err(ContractError::Invariant(
            "retry delays require a positive base no greater than the maximum".to_string()
        ))
    );
}

#[test]
fn policy_cache_write_without_replay_rejected() {
    let mut p = policy();
    p.replay_enabled = false;
    p.cache_write_enabled = true;
    assert_eq!(
        p.validate(),
        Err(ContractError::Invariant(
            "cache writes require replay/cache support to be enabled".to_string()
        ))
    );
}

#[test]
fn policy_valid_ok() {
    assert_eq!(policy().validate(), Ok(()));
}

// --- ProviderPage ---

#[test]
fn provider_page_wrong_schema_version_rejected() {
    let page = ProviderPage {
        schema_version: "org.searchright.provider-page.v0".to_string(),
        records: vec![],
        next_cursor: None,
        total_available: None,
        diagnostics: std::collections::BTreeMap::new(),
    };
    assert_eq!(
        page.validate(),
        Err(ContractError::Invariant(
            "`provider_page.schema_version` must be `org.searchright.provider-page.v1`, found `org.searchright.provider-page.v0`"
                .to_string()
        ))
    );
}

#[test]
fn provider_page_empty_diag_key_rejected() {
    let mut diagnostics = std::collections::BTreeMap::new();
    diagnostics.insert(String::new(), serde_json::Value::Null);
    let page = ProviderPage {
        schema_version: PROVIDER_PAGE_SCHEMA_VERSION.to_string(),
        records: vec![],
        next_cursor: None,
        total_available: None,
        diagnostics,
    };
    assert_eq!(
        page.validate(),
        Err(ContractError::Invariant(
            "provider diagnostic keys must not be empty".to_string()
        ))
    );
}

// --- SourceReceipt ---

fn receipt() -> SourceReceipt {
    SourceReceipt {
        schema_version: SOURCE_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: "receipt-1".to_string(),
        review_id: "review-1".to_string(),
        run_id: "run-1".to_string(),
        provider_id: "pubmed".to_string(),
        source_label: "MEDLINE via PubMed".to_string(),
        strategy_id: "strategy-1".to_string(),
        query_hash: "a".repeat(64),
        executed_at: "2026-08-06T00:00:00Z".to_string(),
        records_retrieved: 10,
        pages_retrieved: 1,
        execution_mode: "fixture".to_string(),
        endpoint: Some("example.test".to_string()),
        policy: policy(),
        provider_version: "fixture/1".to_string(),
        compiler_version: "compiler/1".to_string(),
        result_digest: "b".repeat(64),
        cache_hits: 0,
        cache_writes: 0,
        warnings: vec![],
    }
}

#[test]
fn receipt_records_without_page_rejected() {
    let mut r = receipt();
    r.pages_retrieved = 0;
    assert_eq!(
        r.validate(),
        Err(ContractError::Invariant(
            "a receipt cannot report records without a retrieved page".to_string()
        ))
    );
}

#[test]
fn receipt_cache_counters_inconsistent_rejected() {
    let mut r = receipt();
    r.cache_hits = 5;
    r.cache_writes = 5;
    assert_eq!(
        r.validate(),
        Err(ContractError::Invariant(
            "cache counters are inconsistent with retrieved pages".to_string()
        ))
    );
}

#[test]
fn receipt_empty_warning_rejected() {
    let mut r = receipt();
    r.warnings = vec![String::new()];
    assert_eq!(
        r.validate(),
        Err(ContractError::Invariant(
            "source-receipt warnings must not be empty".to_string()
        ))
    );
}

#[test]
fn receipt_valid_ok() {
    assert_eq!(receipt().validate(), Ok(()));
}

// --- AuditEvent ---

fn actor() -> Actor {
    Actor {
        actor_id: "reviewer-1".to_string(),
        actor_type: "human".to_string(),
        provenance: Some("test".to_string()),
    }
}

fn audit_event() -> AuditEvent {
    AuditEvent {
        schema_version: AUDIT_EVENT_SCHEMA_VERSION.to_string(),
        event_id: "event-1".to_string(),
        review_id: "review-1".to_string(),
        event_type: "review.validated".to_string(),
        occurred_at: "2026-08-06T00:00:00Z".to_string(),
        actor: actor(),
        payload: serde_json::Value::Null,
        previous_hash: "GENESIS".to_string(),
        event_hash: "a".repeat(64),
    }
}

#[test]
fn audit_event_valid_ok() {
    assert_eq!(audit_event().validate(), Ok(()));
}

#[test]
fn audit_event_wrong_schema_version_rejected() {
    let mut e = audit_event();
    e.schema_version = "org.searchright.audit-event.v99".to_string();
    assert_eq!(
        e.validate(),
        Err(ContractError::Invariant(
            "`audit.schema_version` must be `org.searchright.audit-event.v1`, found `org.searchright.audit-event.v99`"
                .to_string()
        ))
    );
}

#[test]
fn audit_event_empty_event_id_rejected() {
    let mut e = audit_event();
    e.event_id = String::new();
    assert_eq!(
        e.validate(),
        Err(ContractError::EmptyField("audit.event_id"))
    );
}

#[test]
fn audit_event_empty_previous_hash_rejected() {
    let mut e = audit_event();
    e.previous_hash = String::new();
    assert_eq!(
        e.validate(),
        Err(ContractError::EmptyField("audit.previous_hash"))
    );
}

#[test]
fn actor_empty_id_rejected() {
    let mut a = actor();
    a.actor_id = String::new();
    assert_eq!(
        a.validate(),
        Err(ContractError::EmptyField("audit.actor.actor_id"))
    );
}

#[test]
fn draft_roundtrip_rejects_wrong_version() {
    let draft = AuditEventDraft {
        schema_version: "org.searchright.audit-event.v2".to_string(),
        event_id: "event-1".to_string(),
        review_id: "review-1".to_string(),
        event_type: "event".to_string(),
        occurred_at: "2026-08-06T00:00:00Z".to_string(),
        actor: actor(),
        payload: serde_json::Value::Null,
    };
    assert_eq!(
        draft.validate(),
        Err(ContractError::Invariant(
            "`audit.schema_version` must be `org.searchright.audit-event.v1`, found `org.searchright.audit-event.v2`"
                .to_string()
        ))
    );
}
