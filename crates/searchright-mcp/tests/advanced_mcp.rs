//! Official rmcp client coverage for the bounded local advanced profile.

use std::time::Duration;

use rmcp::{
    ClientLifecycleMode, ClientServiceExt, ServiceExt,
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, CancelTaskParams, ClientCapabilities,
        ClientInfo, GetPromptRequestParams, GetTaskParams, Implementation, JsonObject,
        PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ResourceContents,
        SubscriptionFilter, TaskPayload,
    },
};
use searchright_mcp::SearchrightServer;

async fn current_client(
    tasks: bool,
) -> anyhow::Result<(
    rmcp::service::RunningService<rmcp::service::RoleClient, ClientInfo>,
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
    rmcp::service::RunningService<rmcp::service::RoleClient, ClientInfo>,
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
async fn resources_prompts_cache_pagination_and_mrtr_are_bounded() -> anyhow::Result<()> {
    let (client, server) = current_client(false).await?;
    let tools = client.list_tools(None).await?;
    assert_eq!(tools.tools.len(), 31);
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
    assert_eq!(second.resources.len(), 1);
    assert!(second.next_cursor.is_none());
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
        .read_resource(ReadResourceRequestParams::new(
            "searchright://claim-boundary",
        ))
        .await?;
    let content = resource
        .contents
        .first()
        .ok_or_else(|| anyhow::anyhow!("claim boundary resource is empty"))?;
    let ResourceContents::TextResourceContents { text, .. } = content else {
        anyhow::bail!("claim boundary is not text")
    };
    assert!(text.contains("No live-provider"));

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
async fn subscriptions_acknowledge_only_supported_filters_and_emit_no_false_changes()
-> anyhow::Result<()> {
    let (client, server) = current_client(false).await?;
    let requested = SubscriptionFilter::builder()
        .resources_list_changed()
        .tools_list_changed()
        .build();
    let mut subscription = client.listen(requested).await?;
    assert_eq!(
        subscription.acknowledged(),
        &SubscriptionFilter::builder()
            .resources_list_changed()
            .build()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), subscription.next())
            .await
            .is_err()
    );
    subscription.cancel().await?;
    client.cancel().await?;
    server.abort();
    Ok(())
}
