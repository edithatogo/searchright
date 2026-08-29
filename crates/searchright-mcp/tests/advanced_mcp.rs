//! Official rmcp client coverage for the bounded local advanced profile.

use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rmcp::{
    ClientHandler, ClientLifecycleMode, ClientServiceExt, ServiceExt,
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, CancelTaskParams, ClientCapabilities,
        ClientInfo, ElicitRequestParams, ElicitResult, ElicitationAction, GetPromptRequestParams,
        GetTaskParams, Implementation, JsonObject, PaginatedRequestParams, ProtocolVersion,
        ReadResourceRequestParams, ResourceContents, ServerNotification, SubscriptionFilter,
        TaskPayload,
    },
    service::{RequestContext, RoleClient},
};
use searchright_mcp::{
    EffectAuthorityAttestation, EffectAuthorityError, EffectAuthorityRequest,
    EffectAuthorityVerifier, SearchrightServer, live_client_success_cases,
};

async fn current_client(
    tasks: bool,
) -> anyhow::Result<(
    rmcp::service::RunningService<RoleClient, ClientInfo>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
)> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default().serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let capabilities = if tasks {
        ClientCapabilities::builder().enable_tasks().build()
    } else {
        ClientCapabilities::default()
    };
    let info = ClientInfo::new(capabilities, Implementation::new("track24-test", "1.0.0"));
    let client = info
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await?;
    Ok((client, server))
}

async fn previous_client() -> anyhow::Result<(
    rmcp::service::RunningService<RoleClient, ClientInfo>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
)> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default().serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let info = ClientInfo::new(
        ClientCapabilities::builder().enable_tasks().build(),
        Implementation::new("track24-previous-test", "1.0.0"),
    )
    .with_protocol_version(ProtocolVersion::V_2025_11_25);
    let client = info.serve(client_transport).await?;
    Ok((client, server))
}

#[tokio::test]
async fn preview_tools_do_not_create_the_local_store() -> anyhow::Result<()> {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let store_root = std::env::temp_dir().join(format!(
        "searchright-track10-preview-{}-{unique}",
        std::process::id()
    ));
    if store_root.exists() {
        fs::remove_dir_all(&store_root)?;
    }
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server_store_root = store_root.clone();
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default()
            .with_local_store_root(server_store_root)
            .serve(server_transport)
            .await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("track10-preview-test", "1.0.0"),
    )
    .serve_with_lifecycle(
        client_transport,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![ProtocolVersion::V_2026_07_28],
        },
    )
    .await?;

    for case in live_client_success_cases()
        .map_err(anyhow::Error::msg)?
        .into_iter()
        .filter(|case| {
            matches!(
                case.tool_name,
                "plan_review" | "press_review_strategy" | "execute_search"
            )
        })
    {
        let CallToolResponse::Complete(result) = client
            .peer()
            .call_tool_once(
                CallToolRequestParams::new(case.tool_name).with_arguments(case.arguments),
            )
            .await
            .map_err(|error| anyhow::anyhow!("{} preview failed: {error}", case.tool_name))?
        else {
            anyhow::bail!("preview unexpectedly returned a task response")
        };
        anyhow::ensure!(
            !result.is_error.unwrap_or(false),
            "{} failed",
            case.tool_name
        );
        anyhow::ensure!(
            !store_root.exists(),
            "{} preview created the local store",
            case.tool_name
        );
    }

    client.cancel().await?;
    server.await??;
    Ok(())
}

#[tokio::test]
async fn resources_prompts_cache_pagination_and_mrtr_are_bounded() -> anyhow::Result<()> {
    let (client, server) = current_client(false).await?;
    let tools = client.list_tools(None).await?;
    assert_eq!(tools.tools.len(), 35);
    assert_eq!(tools.ttl_ms, Some(60_000));
    assert_eq!(tools.cache_scope, Some(CacheScope::Public));

    let first = client.list_resources(None).await?;
    assert_eq!(first.resources.len(), 1);
    assert_eq!(first.ttl_ms, Some(60_000));
    assert_eq!(first.next_cursor.as_deref(), Some("resources:2"));
    let second = client
        .list_resources(Some(
            PaginatedRequestParams::default().with_cursor(first.next_cursor),
        ))
        .await?;
    assert_eq!(second.resources.len(), 2);
    assert_eq!(second.next_cursor.as_deref(), Some("resources:3"));
    let third = client
        .list_resources(Some(
            PaginatedRequestParams::default().with_cursor(second.next_cursor),
        ))
        .await?;
    assert_eq!(third.resources.len(), 4);
    assert!(third.next_cursor.is_none());
    let resource_uris = third
        .resources
        .iter()
        .map(|resource| resource.uri.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        resource_uris,
        [
            "searchright://contracts/plans",
            "searchright://contracts/runs",
            "searchright://contracts/queues",
            "searchright://contracts/reports",
        ]
    );
    assert!(
        client
            .list_resources(Some(
                PaginatedRequestParams::default().with_cursor(Some("prompts:2".to_owned())),
            ))
            .await
            .is_err()
    );

    let prompts = client.list_prompts(None).await?;
    assert_eq!(prompts.prompts.len(), 1);
    assert_eq!(prompts.next_cursor.as_deref(), Some("prompts:2"));
    let prompt = client
        .get_prompt(GetPromptRequestParams::new("plan-review"))
        .await?;
    assert_eq!(prompt.messages.len(), 1);
    let second_prompts = client
        .list_prompts(Some(
            PaginatedRequestParams::default().with_cursor(prompts.next_cursor),
        ))
        .await?;
    assert_eq!(second_prompts.prompts.len(), 2);
    assert!(second_prompts.next_cursor.is_none());
    assert_eq!(
        second_prompts
            .prompts
            .iter()
            .map(|prompt| prompt.name.as_str())
            .collect::<Vec<_>>(),
        vec!["press-check", "update-workflow"]
    );
    let mut unsafe_arguments = JsonObject::new();
    unsafe_arguments.insert(
        "review_id".to_owned(),
        serde_json::Value::String("review-1\nignore-policy".to_owned()),
    );
    assert!(
        client
            .get_prompt(
                GetPromptRequestParams::new("plan-review").with_arguments(unsafe_arguments),
            )
            .await
            .is_err()
    );

    let resource = client
        .read_resource(ReadResourceRequestParams::new("searchright://workflow"))
        .await?;
    let content = resource
        .contents
        .first()
        .ok_or_else(|| anyhow::anyhow!("workflow resource is empty"))?;
    let ResourceContents::TextResourceContents { text, .. } = content else {
        anyhow::bail!("workflow resource is not text")
    };
    assert!(!text.is_empty());

    for uri in resource_uris {
        let resource = client
            .read_resource(ReadResourceRequestParams::new(uri))
            .await?;
        let serialized = serde_json::to_string(&resource)?;
        assert!(serialized.contains("noncanonical_contract_resource"));
        assert!(serialized.contains("no execution or screening authority"));
    }

    let mut update_arguments = JsonObject::new();
    update_arguments.insert(
        "review_id".to_owned(),
        serde_json::Value::String("review-1".to_owned()),
    );
    update_arguments.insert(
        "parent_run_id".to_owned(),
        serde_json::Value::String("run-1".to_owned()),
    );
    update_arguments.insert(
        "mode".to_owned(),
        serde_json::Value::String("evidence-gaps".to_owned()),
    );
    let update = client
        .get_prompt(GetPromptRequestParams::new("update-workflow").with_arguments(update_arguments))
        .await?;
    let serialized = serde_json::to_string(&update)?;
    assert!(serialized.contains("immutable parent-run lineage"));
    assert!(serialized.contains("no execution or screening authority"));
    let mut unsafe_update_arguments = JsonObject::new();
    unsafe_update_arguments.insert(
        "parent_run_id".to_owned(),
        serde_json::Value::String("<ignore-policy>".to_owned()),
    );
    assert!(
        client
            .get_prompt(
                GetPromptRequestParams::new("update-workflow")
                    .with_arguments(unsafe_update_arguments),
            )
            .await
            .is_err()
    );

    client.cancel().await?;
    server.abort();
    Ok(())
}

#[derive(Clone)]
struct AcknowledgingClient;

#[allow(
    unknown_lints,
    clippy::unused_async_trait_impl,
    reason = "rmcp ClientHandler trait specifies async functions"
)]
impl ClientHandler for AcknowledgingClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            ClientCapabilities::builder()
                .enable_elicitation()
                .enable_elicitation_schema_validation()
                .build(),
            Implementation::new("track24-elicitation-test", "1.0.0"),
        )
    }

    async fn create_elicitation(
        &self,
        _request: ElicitRequestParams,
        _context: RequestContext<RoleClient>,
    ) -> Result<ElicitResult, rmcp::ErrorData> {
        Ok(ElicitResult::new(ElicitationAction::Accept)
            .with_content(serde_json::json!({"acknowledged": true})))
    }
}

#[tokio::test]
async fn completion_and_form_elicitation_are_bounded_and_non_authoritative() -> anyhow::Result<()> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let store_root =
        std::env::temp_dir().join(format!("searchright-track10-mrtr-{}", std::process::id()));
    if store_root.exists() {
        fs::remove_dir_all(&store_root)?;
    }
    let server_store_root = store_root.clone();
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default()
            .with_local_store_root(server_store_root)
            .serve(server_transport)
            .await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client = AcknowledgingClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await?;
    let completions = client
        .complete_prompt_simple("plan-review", "mode", "evidence")
        .await?;
    assert_eq!(completions, vec!["evidence-gaps"]);
    let update_completions = client
        .complete_prompt_simple("update-workflow", "mode", "find")
        .await?;
    assert_eq!(update_completions, vec!["findings-only"]);
    assert!(
        client
            .complete_prompt_simple("update-workflow", "parent_run_id", "run")
            .await
            .is_err()
    );
    assert!(
        client
            .complete_prompt_simple("plan-review", "review_id", "secret")
            .await
            .is_err()
    );
    let resource = client
        .read_resource(ReadResourceRequestParams::new(
            "searchright://claim-boundary",
        ))
        .await?;
    let serialized = serde_json::to_string(&resource)?;
    assert!(serialized.contains("No live-provider"));
    assert!(!serialized.contains("authority_granted"));

    let plan: serde_json::Value =
        serde_yaml::from_str(include_str!("../../../contracts/examples/review-plan.yaml"))?;
    let mut arguments = JsonObject::new();
    arguments.insert(
        "document".to_owned(),
        serde_json::Value::String(serde_json::to_string(&plan)?),
    );
    arguments.insert(
        "format".to_owned(),
        serde_json::Value::String("json".to_owned()),
    );
    arguments.insert("apply".to_owned(), serde_json::Value::Bool(true));
    arguments.insert(
        "confirmation".to_owned(),
        serde_json::json!({
            "confirmation_id": "mcp-confirmation-1",
            "confirmed_by": "review-lead",
            "confirmed_at": "2026-08-29T00:00:00Z"
        }),
    );
    assert!(
        client
            .call_tool(CallToolRequestParams::new("plan_review").with_arguments(arguments))
            .await
            .is_err()
    );
    assert!(!store_root.exists());

    let policy: serde_json::Value = serde_yaml::from_str(include_str!(
        "../../../contracts/examples/screening-policy.yaml"
    ))?;
    let mut decision: serde_json::Value = serde_yaml::from_str(include_str!(
        "../../../contracts/examples/screening-decision.yaml"
    ))?;
    let decision_object = decision
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("screening decision fixture must be an object"))?;
    decision_object.insert(
        "reviewer_kind".to_owned(),
        serde_json::Value::String("human".to_owned()),
    );
    decision_object.insert(
        "reviewer_id".to_owned(),
        serde_json::Value::String("forged-human".to_owned()),
    );
    let mut screening_arguments = JsonObject::new();
    screening_arguments.insert(
        "policy_json".to_owned(),
        serde_json::Value::String(serde_json::to_string(&policy)?),
    );
    screening_arguments.insert(
        "decision_json".to_owned(),
        serde_json::Value::String(serde_json::to_string(&decision)?),
    );
    assert!(
        client
            .call_tool(
                CallToolRequestParams::new("record_screening_decision")
                    .with_arguments(screening_arguments),
            )
            .await
            .is_err()
    );
    assert!(!store_root.exists());
    client.cancel().await?;
    server.abort();
    if store_root.exists() {
        fs::remove_dir_all(store_root)?;
    }
    Ok(())
}

#[derive(Debug)]
struct EchoAuthorityVerifier {
    nonces: AtomicU64,
    principal_override: Option<&'static str>,
}

impl EffectAuthorityVerifier for EchoAuthorityVerifier {
    fn verify(
        &self,
        request: &EffectAuthorityRequest,
    ) -> Result<EffectAuthorityAttestation, EffectAuthorityError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| EffectAuthorityError)?
            .as_secs();
        Ok(EffectAuthorityAttestation {
            tool_name: request.tool_name.clone(),
            request_digest: request.request_digest.clone(),
            review_id: request.review_id.clone(),
            principal: self
                .principal_override
                .unwrap_or(&request.principal_hint)
                .to_owned(),
            policy_digest: request.policy_digest.clone(),
            store_state_digest: request.store_state_digest.clone(),
            nonce: format!(
                "authority-nonce-{:016}",
                self.nonces.fetch_add(1, Ordering::SeqCst)
            ),
            issued_at_unix_seconds: now,
            expires_at_unix_seconds: now + 60,
        })
    }
}

#[tokio::test]
async fn verifier_principal_mismatch_is_denied_without_store_delta() -> anyhow::Result<()> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let store_root = std::env::temp_dir().join(format!(
        "searchright-track10-verifier-spoof-{}",
        std::process::id()
    ));
    if store_root.exists() {
        fs::remove_dir_all(&store_root)?;
    }
    let server_store_root = store_root.clone();
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default()
            .with_local_store_root(server_store_root)
            .with_effect_authority_verifier(Arc::new(EchoAuthorityVerifier {
                nonces: AtomicU64::new(0),
                principal_override: Some("authenticated-other-human"),
            }))
            .serve(server_transport)
            .await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client = AcknowledgingClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await?;
    let plan: serde_json::Value =
        serde_yaml::from_str(include_str!("../../../contracts/examples/review-plan.yaml"))?;
    let mut arguments = JsonObject::new();
    arguments.insert(
        "document".to_owned(),
        serde_json::Value::String(serde_json::to_string(&plan)?),
    );
    arguments.insert(
        "format".to_owned(),
        serde_json::Value::String("json".to_owned()),
    );
    arguments.insert("apply".to_owned(), serde_json::Value::Bool(true));
    arguments.insert(
        "confirmation".to_owned(),
        serde_json::json!({
            "confirmation_id": "mcp-confirmation-spoof",
            "confirmed_by": "forged-human",
            "confirmed_at": "2026-08-29T00:00:00Z"
        }),
    );
    assert!(
        client
            .call_tool(CallToolRequestParams::new("plan_review").with_arguments(arguments))
            .await
            .is_err()
    );
    assert!(!store_root.exists());
    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn previous_era_profile_is_static_and_cannot_create_tasks_or_use_mrtr() -> anyhow::Result<()>
{
    let (client, server) = previous_client().await?;
    let peer = client
        .peer_info()
        .ok_or_else(|| anyhow::anyhow!("previous server info is absent"))?;
    assert_eq!(peer.protocol_version, ProtocolVersion::V_2025_11_25);
    assert!(!peer.capabilities.supports_tasks());
    assert_eq!(client.list_resources(None).await?.resources.len(), 1);
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new(
                "searchright://claim-boundary",
            ))
            .await
            .is_err()
    );
    let response = client
        .peer()
        .call_tool_once(CallToolRequestParams::new("workflow"))
        .await?;
    assert!(matches!(response, CallToolResponse::Complete(_)));
    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn task_is_current_capability_gated_and_cooperatively_cancelled() -> anyhow::Result<()> {
    let (client, server) = current_client(true).await?;
    let response = client
        .peer()
        .call_tool_once(CallToolRequestParams::new("workflow"))
        .await?;
    let CallToolResponse::Task(created) = response else {
        anyhow::bail!("task-capable current client did not receive a task")
    };
    let task_id = created.task.task_id.clone();
    client
        .peer()
        .cancel_task(CancelTaskParams::new(task_id.clone()))
        .await?;
    let terminal = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let task = client
                .peer()
                .get_task(GetTaskParams::new(task_id.clone()))
                .await?
                .task;
            if task.status().is_terminal() {
                break anyhow::Ok(task);
            }
            tokio::task::yield_now().await;
        }
    })
    .await??;
    assert!(matches!(terminal.payload, TaskPayload::Cancelled));

    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn task_completes_with_the_bounded_workflow_payload() -> anyhow::Result<()> {
    let (client, server) = current_client(true).await?;
    let response = client
        .peer()
        .call_tool_once(CallToolRequestParams::new("workflow"))
        .await?;
    let CallToolResponse::Task(created) = response else {
        anyhow::bail!("task-capable current client did not receive a task")
    };
    let task_id = created.task.task_id;
    let terminal = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let task = client
                .peer()
                .get_task(GetTaskParams::new(task_id.clone()))
                .await?
                .task;
            if task.status().is_terminal() {
                break anyhow::Ok(task);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    let TaskPayload::Completed { result } = terminal.payload else {
        anyhow::bail!("workflow task did not complete")
    };
    assert!(result.contains_key("structuredContent"));
    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn clients_without_task_extension_keep_synchronous_tool_semantics() -> anyhow::Result<()> {
    let (client, server) = current_client(false).await?;
    let response = client
        .peer()
        .call_tool_once(CallToolRequestParams::new("workflow"))
        .await?;
    assert!(matches!(response, CallToolResponse::Complete(_)));
    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn subscriptions_emit_only_real_aggregate_task_activity_changes() -> anyhow::Result<()> {
    let (client, server) = current_client(true).await?;
    let requested = SubscriptionFilter::builder()
        .tools_list_changed()
        .resource_subscriptions(["searchright://runtime/task-activity", "file:///denied"])
        .build();
    let mut subscription = client.listen(requested).await?;
    assert_eq!(
        subscription.acknowledged(),
        &SubscriptionFilter::builder()
            .resource_subscriptions(["searchright://runtime/task-activity"])
            .build()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), subscription.next())
            .await
            .is_err()
    );
    let response = client
        .peer()
        .call_tool_once(CallToolRequestParams::new("workflow"))
        .await?;
    let CallToolResponse::Task(created) = response else {
        anyhow::bail!("task-capable client did not receive a task")
    };
    let notification = tokio::time::timeout(Duration::from_secs(1), subscription.next())
        .await??
        .ok_or_else(|| anyhow::anyhow!("task activity subscription ended early"))?;
    let ServerNotification::ResourceUpdatedNotification(update) = notification else {
        anyhow::bail!("task activity emitted the wrong notification type")
    };
    assert_eq!(update.params.uri, "searchright://runtime/task-activity");
    client
        .peer()
        .cancel_task(CancelTaskParams::new(created.task.task_id))
        .await?;
    subscription.cancel().await?;
    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn local_task_concurrency_is_bounded_and_recovers() -> anyhow::Result<()> {
    let (client, server) = current_client(true).await?;
    let mut task_ids = Vec::new();
    for _ in 0..4 {
        let response = client
            .peer()
            .call_tool_once(CallToolRequestParams::new("workflow"))
            .await?;
        let CallToolResponse::Task(created) = response else {
            anyhow::bail!("bounded task admission did not create a task")
        };
        task_ids.push(created.task.task_id);
    }
    assert!(
        client
            .peer()
            .call_tool_once(CallToolRequestParams::new("workflow"))
            .await
            .is_err()
    );
    client
        .peer()
        .cancel_task(CancelTaskParams::new(task_ids.remove(0)))
        .await?;
    let replacement = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(response) = client
                .peer()
                .call_tool_once(CallToolRequestParams::new("workflow"))
                .await
            {
                break anyhow::Ok(response);
            }
            tokio::task::yield_now().await;
        }
    })
    .await??;
    let CallToolResponse::Task(replacement) = replacement else {
        anyhow::bail!("released task capacity did not admit a replacement")
    };
    task_ids.push(replacement.task.task_id);
    for task_id in task_ids {
        client
            .peer()
            .cancel_task(CancelTaskParams::new(task_id))
            .await?;
    }
    client.cancel().await?;
    server.abort();
    Ok(())
}
