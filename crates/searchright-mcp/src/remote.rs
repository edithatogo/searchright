//! Authenticated Streamable HTTP deployment adapter.
//!
//! The adapter binds only to a loopback address and requires a trusted local
//! TLS terminator to mark requests with `X-Forwarded-Proto: https`. Identity
//! claims are verified from a locally provisioned rotating JWKS file before a
//! request reaches the MCP service. Client-provided identity metadata is never
//! trusted.

use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use searchright_access::{ReplayLedger, authorise_with_replay};
use searchright_contracts::{
    ACCESS_REQUEST_SCHEMA_VERSION, AccessRequest, AccessScope, PrincipalKind, TenantPolicy,
    Validate,
};
use serde::{Deserialize, Serialize};

use crate::SearchrightServer;

const REQUEST_ID_HEADER: &str = "x-request-id";
const FORWARDED_PROTO_HEADER: &str = "x-forwarded-proto";
const RATE_WINDOW: Duration = Duration::from_mins(1);
const REMOTE_POLICY_SCHEMA_VERSION: &str = "org.searchright.remote-mcp-policy.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteMcpPolicy {
    schema_version: String,
    issuer: String,
    maximum_token_age_seconds: u64,
    maximum_requests_per_minute: u32,
    deployment_region: String,
    tenant_policy: TenantPolicy,
}

impl RemoteMcpPolicy {
    fn validate(&self) -> Result<(), RemoteError> {
        self.tenant_policy
            .validate()
            .map_err(|_| RemoteError::Policy("tenant policy invariants failed".to_owned()))?;
        if self.schema_version != REMOTE_POLICY_SCHEMA_VERSION
            || self.issuer.trim().is_empty()
            || self.maximum_token_age_seconds == 0
            || self.maximum_requests_per_minute == 0
            || !self
                .tenant_policy
                .allowed_regions
                .contains(&self.deployment_region)
        {
            return Err(RemoteError::Policy(
                "remote policy identity, rate or deployment-region invariants failed".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Environment-backed settings for the remote adapter.
#[derive(Debug, Clone)]
pub struct RemoteRuntimeConfig {
    /// Loopback socket used behind a trusted TLS terminator.
    pub bind: SocketAddr,
    /// Public Host authority accepted by the MCP transport.
    pub allowed_host: String,
    /// Optional allowed browser origins.
    pub allowed_origins: Vec<String>,
    /// Expected `OAuth` audience.
    pub audience: String,
    /// Rotating JSON Web Key Set file.
    pub jwks_path: PathBuf,
    /// Tenant policy loaded at process start.
    pub tenant_policy_path: PathBuf,
    /// Append-only redacted authorization audit JSONL path.
    pub audit_path: PathBuf,
    /// Maximum wall-clock duration of one authenticated HTTP request.
    pub request_timeout: Duration,
    /// Maximum concurrent signature-verification operations.
    pub authentication_concurrency: usize,
}

impl RemoteRuntimeConfig {
    /// Load the fail-closed deployment settings from environment variables.
    pub fn from_environment() -> Result<Self, RemoteError> {
        if std::env::var("SEARCHRIGHT_REMOTE_MCP_ENABLED").as_deref() != Ok("1") {
            return Err(RemoteError::Configuration(
                "SEARCHRIGHT_REMOTE_MCP_ENABLED must equal 1".to_owned(),
            ));
        }
        let bind = required_environment("SEARCHRIGHT_REMOTE_MCP_BIND")?
            .parse::<SocketAddr>()
            .map_err(|_| RemoteError::Configuration("invalid loopback bind address".to_owned()))?;
        if !bind.ip().is_loopback() {
            return Err(RemoteError::Configuration(
                "remote MCP must bind to loopback behind the trusted TLS terminator".to_owned(),
            ));
        }
        let allowed_host = required_environment("SEARCHRIGHT_REMOTE_MCP_ALLOWED_HOST")?;
        let audience = required_environment("SEARCHRIGHT_REMOTE_MCP_AUDIENCE")?;
        let jwks_path = PathBuf::from(required_environment("SEARCHRIGHT_REMOTE_MCP_JWKS")?);
        let tenant_policy_path = PathBuf::from(required_environment(
            "SEARCHRIGHT_REMOTE_MCP_TENANT_POLICY",
        )?);
        let audit_path = PathBuf::from(required_environment("SEARCHRIGHT_REMOTE_MCP_AUDIT_LOG")?);
        let request_timeout_seconds =
            required_environment("SEARCHRIGHT_REMOTE_MCP_REQUEST_TIMEOUT_SECONDS")?
                .parse::<u64>()
                .map_err(|_| {
                    RemoteError::Configuration(
                        "SEARCHRIGHT_REMOTE_MCP_REQUEST_TIMEOUT_SECONDS must be an integer"
                            .to_owned(),
                    )
                })?;
        if !(1..=300).contains(&request_timeout_seconds) {
            return Err(RemoteError::Configuration(
                "SEARCHRIGHT_REMOTE_MCP_REQUEST_TIMEOUT_SECONDS must be between 1 and 300"
                    .to_owned(),
            ));
        }
        let authentication_concurrency =
            required_environment("SEARCHRIGHT_REMOTE_MCP_AUTH_CONCURRENCY")?
                .parse::<usize>()
                .map_err(|_| {
                    RemoteError::Configuration(
                        "SEARCHRIGHT_REMOTE_MCP_AUTH_CONCURRENCY must be an integer".to_owned(),
                    )
                })?;
        if !(1..=256).contains(&authentication_concurrency) {
            return Err(RemoteError::Configuration(
                "SEARCHRIGHT_REMOTE_MCP_AUTH_CONCURRENCY must be between 1 and 256".to_owned(),
            ));
        }
        let allowed_origins = std::env::var("SEARCHRIGHT_REMOTE_MCP_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if allowed_origins.is_empty()
            || allowed_origins
                .iter()
                .any(|origin| !origin.starts_with("https://"))
        {
            return Err(RemoteError::Configuration(
                "SEARCHRIGHT_REMOTE_MCP_ALLOWED_ORIGINS requires an explicit HTTPS allowlist"
                    .to_owned(),
            ));
        }
        Ok(Self {
            bind,
            allowed_host,
            allowed_origins,
            audience,
            jwks_path,
            tenant_policy_path,
            audit_path,
            request_timeout: Duration::from_secs(request_timeout_seconds),
            authentication_concurrency,
        })
    }
}

fn required_environment(name: &'static str) -> Result<String, RemoteError> {
    let value = std::env::var(name)
        .map_err(|_| RemoteError::Configuration(format!("{name} is required")))?;
    if value.trim().is_empty() {
        return Err(RemoteError::Configuration(format!(
            "{name} must not be blank"
        )));
    }
    Ok(value)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct IdentityClaims {
    iss: String,
    sub: String,
    aud: Audience,
    exp: u64,
    iat: u64,
    jti: String,
    tenant_id: String,
    region: String,
    scope: Scopes,
    principal_kind: PrincipalKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

impl Audience {
    fn contains(&self, expected: &str) -> bool {
        match self {
            Self::One(value) => value == expected,
            Self::Many(values) => values.iter().any(|value| value == expected),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum Scopes {
    SpaceDelimited(String),
    List(Vec<String>),
}

impl Scopes {
    fn values(&self) -> impl Iterator<Item = &str> {
        let values: Vec<&str> = match self {
            Self::SpaceDelimited(value) => value.split_ascii_whitespace().collect(),
            Self::List(values) => values.iter().map(String::as_str).collect(),
        };
        values.into_iter()
    }
}

#[derive(Clone)]
struct JwksVerifier {
    path: Arc<PathBuf>,
    audience: Arc<String>,
}

impl JwksVerifier {
    async fn verify(&self, token: &str) -> Result<IdentityClaims, RemoteDenial> {
        let bytes = tokio::fs::read(self.path.as_ref())
            .await
            .map_err(|_| RemoteDenial::service("access.identity.keys_unavailable"))?;
        let keys: JwkSet = serde_json::from_slice(&bytes)
            .map_err(|_| RemoteDenial::service("access.identity.keys_invalid"))?;
        let header = decode_header(token)
            .map_err(|_| RemoteDenial::unauthorized("access.authentication.invalid"))?;
        if header.alg != Algorithm::RS256 {
            return Err(RemoteDenial::unauthorized(
                "access.authentication.algorithm_denied",
            ));
        }
        let key_id = header
            .kid
            .as_deref()
            .ok_or_else(|| RemoteDenial::unauthorized("access.authentication.key_id_required"))?;
        let jwk = keys
            .find(key_id)
            .ok_or_else(|| RemoteDenial::unauthorized("access.authentication.key_unknown"))?;
        let key = DecodingKey::from_jwk(jwk)
            .map_err(|_| RemoteDenial::service("access.identity.key_unusable"))?;
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[self.audience.as_str()]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.validate_nbf = true;
        validation.leeway = 30;
        let claims = decode::<IdentityClaims>(token, &key, &validation)
            .map_err(|_| RemoteDenial::unauthorized("access.authentication.invalid"))?
            .claims;
        if !claims.aud.contains(self.audience.as_str()) {
            return Err(RemoteDenial::unauthorized(
                "access.authentication.audience_denied",
            ));
        }
        Ok(claims)
    }
}

#[derive(Default)]
struct RuntimeCounters {
    requests: HashMap<(String, String), VecDeque<Instant>>,
    active: HashMap<String, u32>,
    replay: ReplayLedger,
}

#[derive(Clone)]
struct RemoteState {
    verifier: JwksVerifier,
    policy: Arc<RemoteMcpPolicy>,
    counters: Arc<Mutex<RuntimeCounters>>,
    request_timeout: Duration,
    audit: AuditSink,
    authentication_slots: Arc<tokio::sync::Semaphore>,
}

#[derive(Clone)]
struct AuditSink {
    file: Arc<Mutex<std::fs::File>>,
}

#[derive(Serialize)]
struct RemoteAuditEvent<'a> {
    schema_version: &'static str,
    observed_at_unix_seconds: u64,
    request_digest: &'a str,
    tenant_digest: String,
    principal_digest: String,
    policy_version: &'a str,
    outcome: &'a str,
}

impl AuditSink {
    fn open(path: &Path) -> Result<Self, RemoteError> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| RemoteError::Audit("audit log is unavailable".to_owned()))?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    fn record(
        &self,
        claims: &IdentityClaims,
        policy_version: &str,
        request_digest: &str,
        outcome: &str,
    ) -> Result<(), RemoteDenial> {
        let event = RemoteAuditEvent {
            schema_version: "org.searchright.remote-mcp-audit-event.v1",
            observed_at_unix_seconds: unix_seconds()?,
            request_digest,
            tenant_digest: digest_identifier(&claims.tenant_id),
            principal_digest: digest_identifier(&claims.sub),
            policy_version,
            outcome,
        };
        let mut bytes = serde_json::to_vec(&event)
            .map_err(|_| RemoteDenial::service("access.audit.serialization_failed"))?;
        bytes.push(b'\n');
        let mut file = self
            .file
            .lock()
            .map_err(|_| RemoteDenial::service("access.audit.lock_failed"))?;
        file.write_all(&bytes)
            .and_then(|()| file.flush())
            .map_err(|_| RemoteDenial::service("access.audit.write_failed"))
    }
}

fn digest_identifier(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

impl RemoteState {
    fn authorise(
        &self,
        claims: &IdentityClaims,
        request_id: &str,
    ) -> Result<RuntimePermit, RemoteDenial> {
        let scopes = parse_scopes(&claims.scope)?;
        if !scopes.contains(&AccessScope::ReviewRead) {
            return Err(RemoteDenial::forbidden("access.scope.review_read_required"));
        }
        let now = unix_seconds()?;
        if claims.iss != self.policy.issuer {
            return Err(RemoteDenial::unauthorized(
                "access.authentication.issuer_denied",
            ));
        }
        if claims.region != self.policy.deployment_region {
            return Err(RemoteDenial::forbidden("access.region.denied"));
        }
        if claims.iat > now.saturating_add(30) || claims.exp <= now {
            return Err(RemoteDenial::unauthorized(
                "access.authentication.invalid_time",
            ));
        }
        if now.saturating_sub(claims.iat) > self.policy.maximum_token_age_seconds {
            return Err(RemoteDenial::unauthorized(
                "access.authentication.token_too_old",
            ));
        }
        let key = (claims.tenant_id.clone(), claims.sub.clone());
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| RemoteDenial::service("access.runtime.lock_failed"))?;
        let observed_requests_last_minute = {
            let request_times = counters.requests.entry(key.clone()).or_default();
            let cutoff = Instant::now()
                .checked_sub(RATE_WINDOW)
                .ok_or_else(|| RemoteDenial::service("access.runtime.clock_failed"))?;
            while request_times.front().is_some_and(|seen| *seen < cutoff) {
                request_times.pop_front();
            }
            u32::try_from(request_times.len())
                .map_err(|_| RemoteDenial::forbidden("access.rate.exceeded"))?
        };
        let active_concurrent_tasks = match counters.active.get(&claims.tenant_id) {
            Some(active) => *active,
            None => 0,
        };
        if observed_requests_last_minute >= self.policy.maximum_requests_per_minute {
            return Err(RemoteDenial::forbidden("access.rate.exceeded"));
        }
        if active_concurrent_tasks >= self.policy.tenant_policy.maximum_concurrent_tasks {
            return Err(RemoteDenial::forbidden("access.concurrency.exceeded"));
        }
        let access_request = AccessRequest {
            schema_version: ACCESS_REQUEST_SCHEMA_VERSION.to_owned(),
            request_id: replay_key(claims, request_id),
            principal_id: claims.sub.clone(),
            principal_kind: claims.principal_kind,
            tenant_id: claims.tenant_id.clone(),
            scopes,
            region: self.policy.deployment_region.clone(),
            authenticated: true,
            external_write: false,
            final_eligibility_decision: false,
            human_approval: false,
        };
        let decision = authorise_with_replay(
            &self.policy.tenant_policy,
            &access_request,
            &mut counters.replay,
        )
        .map_err(|_| RemoteDenial::service("access.policy.invalid"))?;
        if !decision.permitted {
            let blocker = decision
                .blockers
                .first()
                .map_or("access.denied", String::as_str);
            return Err(RemoteDenial::forbidden(blocker));
        }
        counters
            .requests
            .entry(key)
            .or_default()
            .push_back(Instant::now());
        let active = counters.active.entry(claims.tenant_id.clone()).or_default();
        *active = active.saturating_add(1);
        drop(counters);
        Ok(RuntimePermit {
            state: self.clone(),
            tenant_id: claims.tenant_id.clone(),
        })
    }

    fn release(&self, tenant_id: &str) {
        if let Ok(mut counters) = self.counters.lock()
            && let Some(active) = counters.active.get_mut(tenant_id)
        {
            *active = active.saturating_sub(1);
            if *active == 0 {
                counters.active.remove(tenant_id);
            }
        }
    }
}

struct RuntimePermit {
    state: RemoteState,
    tenant_id: String,
}

impl Drop for RuntimePermit {
    fn drop(&mut self) {
        self.state.release(&self.tenant_id);
    }
}

fn replay_key(claims: &IdentityClaims, request_id: &str) -> String {
    blake3::hash(
        format!(
            "{}\0{}\0{}\0{}\0{request_id}",
            claims.iss, claims.tenant_id, claims.sub, claims.jti
        )
        .as_bytes(),
    )
    .to_hex()
    .to_string()
}

fn parse_scopes(scopes: &Scopes) -> Result<Vec<AccessScope>, RemoteDenial> {
    let mut parsed = Vec::new();
    for scope in scopes.values() {
        let value = match scope {
            "review_read" => AccessScope::ReviewRead,
            "review_write" => AccessScope::ReviewWrite,
            "search_execute" => AccessScope::SearchExecute,
            "screening_recommend" => AccessScope::ScreeningRecommend,
            "screening_decide" => AccessScope::ScreeningDecide,
            "tenant_admin" => AccessScope::TenantAdmin,
            "external_write" => AccessScope::ExternalWrite,
            _ => return Err(RemoteDenial::forbidden("access.scope.unknown")),
        };
        if !parsed.contains(&value) {
            parsed.push(value);
        }
    }
    Ok(parsed)
}

fn unix_seconds() -> Result<u64, RemoteDenial> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| RemoteDenial::service("access.runtime.clock_failed"))
}

async fn authorise_http(
    State(state): State<RemoteState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, RemoteDenial> {
    if headers
        .get(FORWARDED_PROTO_HEADER)
        .and_then(|value| value.to_str().ok())
        != Some("https")
    {
        return Err(RemoteDenial::forbidden("access.transport.tls_required"));
    }
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .ok_or_else(|| RemoteDenial::forbidden("access.request_id.required"))?;
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| RemoteDenial::unauthorized("access.authentication.required"))?;
    let token = authorization
        .strip_prefix("Bearer ")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| RemoteDenial::unauthorized("access.authentication.required"))?;
    let _authentication_slot = state
        .authentication_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| RemoteDenial::too_many("access.authentication.capacity_exceeded"))?;
    let claims = state.verifier.verify(token).await?;
    let request_digest = replay_key(&claims, request_id);
    let _permit = state.authorise(&claims, request_id)?;
    state.audit.record(
        &claims,
        &state.policy.tenant_policy.policy_version,
        &request_digest,
        "admitted",
    )?;
    let response = bounded_request(state.request_timeout, next.run(request)).await;
    let outcome = match &response {
        Ok(value) if value.status().is_success() => "completed",
        Ok(_) => "completed_error",
        Err(_) => "timed_out",
    };
    state.audit.record(
        &claims,
        &state.policy.tenant_policy.policy_version,
        &request_digest,
        outcome,
    )?;
    response
}

async fn bounded_request<F>(request_timeout: Duration, future: F) -> Result<Response, RemoteDenial>
where
    F: Future<Output = Response>,
{
    tokio::time::timeout(request_timeout, future)
        .await
        .map_err(|_| RemoteDenial::timeout("access.request.timeout"))
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
}

#[derive(Debug)]
struct RemoteDenial {
    status: StatusCode,
    code: String,
}

impl RemoteDenial {
    fn unauthorized(code: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: code.to_owned(),
        }
    }

    fn forbidden(code: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: code.to_owned(),
        }
    }

    fn service(code: &str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: code.to_owned(),
        }
    }

    fn timeout(code: &str) -> Self {
        Self {
            status: StatusCode::REQUEST_TIMEOUT,
            code: code.to_owned(),
        }
    }

    fn too_many(code: &str) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: code.to_owned(),
        }
    }
}

impl IntoResponse for RemoteDenial {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorBody { code: &self.code })).into_response()
    }
}

/// Remote adapter setup failure.
#[derive(Debug, thiserror::Error)]
pub enum RemoteError {
    /// Invalid or missing fail-closed configuration.
    #[error("remote MCP configuration rejected: {0}")]
    Configuration(String),
    /// Tenant policy could not be loaded or validated.
    #[error("remote MCP tenant policy rejected: {0}")]
    Policy(String),
    /// Listener or server failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Required append-only audit sink was unavailable.
    #[error("remote MCP audit rejected: {0}")]
    Audit(String),
}

fn load_policy(path: &Path) -> Result<RemoteMcpPolicy, RemoteError> {
    let bytes = std::fs::read(path)
        .map_err(|_| RemoteError::Policy("policy file is unavailable".to_owned()))?;
    let policy: RemoteMcpPolicy = serde_json::from_slice(&bytes)
        .map_err(|_| RemoteError::Policy("policy JSON is invalid".to_owned()))?;
    policy.validate()?;
    Ok(policy)
}

/// Run the authenticated Streamable HTTP adapter from environment settings.
pub async fn run_from_environment() -> anyhow::Result<()> {
    let config = RemoteRuntimeConfig::from_environment()?;
    let policy = load_policy(&config.tenant_policy_path)?;
    let audit = AuditSink::open(&config.audit_path)?;
    let state = RemoteState {
        verifier: JwksVerifier {
            path: Arc::new(config.jwks_path),
            audience: Arc::new(config.audience),
        },
        policy: Arc::new(policy),
        counters: Arc::new(Mutex::new(RuntimeCounters::default())),
        request_timeout: config.request_timeout,
        audit,
        authentication_slots: Arc::new(tokio::sync::Semaphore::new(
            config.authentication_concurrency,
        )),
    };
    let (app, cancellation) = application(state, config.allowed_host, config.allowed_origins);
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            cancellation.cancel();
        })
        .await?;
    Ok(())
}

fn application(
    state: RemoteState,
    allowed_host: String,
    allowed_origins: Vec<String>,
) -> (Router, tokio_util::sync::CancellationToken) {
    let server_config = StreamableHttpServerConfig::default()
        .with_allowed_hosts([allowed_host])
        .with_allowed_origins(allowed_origins)
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_stateless_protocol_metadata_required(true)
        .with_max_request_body_bytes(1024 * 1024);
    let cancellation = server_config.cancellation_token.clone();
    let service: StreamableHttpService<SearchrightServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(SearchrightServer::remote_http()),
            Arc::default(),
            server_config,
        );
    let app = Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(state, authorise_http));
    (app, cancellation)
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "fixture setup failures should stop the focused security test immediately"
)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use jsonwebtoken::{EncodingKey, Header, encode};
    use searchright_contracts::TENANT_POLICY_SCHEMA_VERSION;

    const TEST_PRIVATE_KEY: &str = include_str!("../tests/fixtures/remote-mcp-private.der.b64");
    const TEST_JWKS: &str = include_str!("../tests/fixtures/remote-mcp-jwks.json");

    fn policy(rate: u32, concurrency: u32) -> RemoteMcpPolicy {
        RemoteMcpPolicy {
            schema_version: REMOTE_POLICY_SCHEMA_VERSION.to_owned(),
            issuer: "https://issuer.example.test".to_owned(),
            maximum_token_age_seconds: 300,
            maximum_requests_per_minute: rate,
            deployment_region: "AU".to_owned(),
            tenant_policy: TenantPolicy {
                schema_version: TENANT_POLICY_SCHEMA_VERSION.to_owned(),
                tenant_id: "tenant-demo".to_owned(),
                allowed_regions: vec!["AU".to_owned()],
                allowed_scopes: vec![AccessScope::ReviewRead],
                maximum_concurrent_tasks: concurrency,
                external_model_processing_allowed: false,
                restricted_full_text_persistence_allowed: false,
                cross_tenant_aggregation_allowed: false,
                approved_by: "fixture-owner".to_owned(),
                policy_version: "fixture-1".to_owned(),
            },
        }
    }

    fn claims() -> IdentityClaims {
        let now = unix_seconds().expect("test clock must be available");
        IdentityClaims {
            iss: "https://issuer.example.test".to_owned(),
            sub: "reviewer-1".to_owned(),
            aud: Audience::One("searchright-remote".to_owned()),
            exp: now + 300,
            iat: now,
            jti: "token-1".to_owned(),
            tenant_id: "tenant-demo".to_owned(),
            region: "AU".to_owned(),
            scope: Scopes::SpaceDelimited("review_read".to_owned()),
            principal_kind: PrincipalKind::Human,
        }
    }

    fn token(claims: &IdentityClaims) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("track34-fixture".to_owned());
        let key = STANDARD
            .decode(TEST_PRIVATE_KEY.trim())
            .expect("fixture key must decode");
        encode(&header, claims, &EncodingKey::from_rsa_der(&key))
            .expect("fixture token must encode")
    }

    fn temporary_jwks() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must be available")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "searchright-track34-jwks-{}-{suffix}.json",
            std::process::id()
        ));
        std::fs::write(&path, TEST_JWKS).expect("fixture JWKS must be written");
        path
    }

    fn temporary_audit() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must be available")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "searchright-track34-audit-{}-{suffix}.jsonl",
            std::process::id()
        ))
    }

    fn state(policy: RemoteMcpPolicy, jwks_path: PathBuf) -> RemoteState {
        let audit_path = temporary_audit();
        RemoteState {
            verifier: JwksVerifier {
                path: Arc::new(jwks_path),
                audience: Arc::new("searchright-remote".to_owned()),
            },
            policy: Arc::new(policy),
            counters: Arc::new(Mutex::new(RuntimeCounters::default())),
            request_timeout: Duration::from_secs(30),
            audit: AuditSink::open(&audit_path).expect("test audit sink must open"),
            authentication_slots: Arc::new(tokio::sync::Semaphore::new(4)),
        }
    }

    fn denied(result: Result<RuntimePermit, RemoteDenial>) -> RemoteDenial {
        match result {
            Ok(_) => panic!("request should have been denied"),
            Err(denial) => denial,
        }
    }

    #[tokio::test]
    async fn rotating_jwks_verifies_current_key_and_rejects_removed_key() {
        let path = temporary_jwks();
        let verifier = JwksVerifier {
            path: Arc::new(path.clone()),
            audience: Arc::new("searchright-remote".to_owned()),
        };
        let encoded = token(&claims());
        assert!(verifier.verify(&encoded).await.is_ok());
        std::fs::write(&path, r#"{"keys":[]}"#).expect("rotated JWKS must be written");
        let denial = verifier
            .verify(&encoded)
            .await
            .expect_err("removed signing key must be denied");
        assert_eq!(denial.code, "access.authentication.key_unknown");
        std::fs::remove_file(path).expect("temporary JWKS must be removed");
    }

    #[test]
    fn request_replay_rate_and_concurrency_fail_closed() {
        let replay_state = state(policy(10, 2), PathBuf::new());
        let identity = claims();
        let permit = replay_state
            .authorise(&identity, "request-1")
            .expect("first request must pass");
        drop(permit);
        let replay = denied(replay_state.authorise(&identity, "request-1"));
        assert_eq!(replay.code, "access.replay.request_reused");

        let rate_state = state(policy(1, 2), PathBuf::new());
        let permit = rate_state
            .authorise(&identity, "rate-1")
            .expect("first rate request must pass");
        drop(permit);
        let rate = denied(rate_state.authorise(&identity, "rate-2"));
        assert_eq!(rate.code, "access.rate.exceeded");

        let concurrency_state = state(policy(10, 1), PathBuf::new());
        let _lease = concurrency_state
            .authorise(&identity, "concurrency-1")
            .expect("first concurrent request must pass");
        let concurrency = denied(concurrency_state.authorise(&identity, "concurrency-2"));
        assert_eq!(concurrency.code, "access.concurrency.exceeded");
    }

    #[test]
    fn issuer_region_and_token_age_are_bound_to_remote_policy() {
        let remote_state = state(policy(10, 2), PathBuf::new());
        let mut identity = claims();
        identity.iss = "https://other-issuer.example.test".to_owned();
        assert_eq!(
            denied(remote_state.authorise(&identity, "issuer-denied")).code,
            "access.authentication.issuer_denied"
        );

        let mut identity = claims();
        identity.region = "US".to_owned();
        assert_eq!(
            denied(remote_state.authorise(&identity, "region-denied")).code,
            "access.region.denied"
        );

        let mut identity = claims();
        identity.iat = identity.iat.saturating_sub(301);
        assert_eq!(
            denied(remote_state.authorise(&identity, "age-denied")).code,
            "access.authentication.token_too_old"
        );
    }

    #[test]
    fn audit_events_are_redacted_and_correlated() {
        let path = temporary_audit();
        let sink = AuditSink::open(&path).expect("test audit sink must open");
        let identity = claims();
        sink.record(
            &identity,
            "fixture-1",
            &replay_key(&identity, "audit-request"),
            "admitted",
        )
        .expect("test audit event must be written");
        let text = std::fs::read_to_string(&path).expect("test audit must be readable");
        assert!(text.contains("remote-mcp-audit-event.v1"));
        assert!(text.contains("fixture-1"));
        assert!(!text.contains("reviewer-1"));
        assert!(!text.contains("tenant-demo"));
        assert!(!text.contains("token-1"));
        std::fs::remove_file(path).expect("temporary audit must be removed");
    }

    #[tokio::test(start_paused = true)]
    async fn authenticated_request_budget_times_out_fail_closed() {
        let denial = bounded_request(Duration::from_secs(2), std::future::pending::<Response>())
            .await
            .expect_err("pending authenticated request must be bounded");
        assert_eq!(denial.status, StatusCode::REQUEST_TIMEOUT);
        assert_eq!(denial.code, "access.request.timeout");
    }

    #[tokio::test]
    async fn authenticated_streamable_http_initializes_and_replay_is_denied() {
        let jwks_path = temporary_jwks();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test listener must bind");
        let address = listener.local_addr().expect("listener address must exist");
        let (app, cancellation) = application(
            state(policy(10, 2), jwks_path.clone()),
            address.to_string(),
            Vec::new(),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await });
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2026-07-28",
                "capabilities": {},
                "clientInfo": {"name": "track34-test", "version": "1"}
            }
        });
        let client = reqwest::Client::new();
        let url = format!("http://{address}/mcp");
        let encoded = token(&claims());
        let response = client
            .post(&url)
            .header("accept", "application/json, text/event-stream")
            .header("content-type", "application/json")
            .header("x-forwarded-proto", "https")
            .header("x-request-id", "http-request-1")
            .bearer_auth(&encoded)
            .json(&body)
            .send()
            .await
            .expect("authenticated initialize request must complete");
        assert_eq!(response.status(), StatusCode::OK);

        let replay = client
            .post(&url)
            .header("accept", "application/json, text/event-stream")
            .header("content-type", "application/json")
            .header("x-forwarded-proto", "https")
            .header("x-request-id", "http-request-1")
            .bearer_auth(&encoded)
            .json(&body)
            .send()
            .await
            .expect("replayed request must receive a denial");
        assert_eq!(replay.status(), StatusCode::FORBIDDEN);

        cancellation.cancel();
        server.abort();
        std::fs::remove_file(jwks_path).expect("temporary JWKS must be removed");
    }
}
