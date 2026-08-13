//! Track 10 official `rmcp` client conformance for supported local stdio eras.
//!
//! This deliberately uses the SDK's in-memory duplex transport rather than the
//! repository's JSON-RPC smoke harness.  It proves that the typed client API can
//! negotiate each declared protocol era and consume representative successful
//! structured responses and governed errors without a wire-format shim.

use rmcp::{
    ClientLifecycleMode, ClientServiceExt, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, ClientCapabilities, ClientInfo, Implementation,
        JsonObject, ProtocolVersion,
    },
    service::RoleClient,
};
use searchright_mcp::{
    SearchrightServer, live_client_output_matches_schema, live_client_success_cases,
};

type ClientService = rmcp::service::RunningService<RoleClient, ClientInfo>;
type ServerTask = tokio::task::JoinHandle<anyhow::Result<()>>;

async fn serve_current_client() -> anyhow::Result<(ClientService, ServerTask)> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default().serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("searchright-track10-current", "1.0.0"),
    )
    .serve_with_lifecycle(
        client_transport,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![ProtocolVersion::V_2026_07_28],
        },
    )
    .await?;
    Ok((client, server))
}

async fn serve_previous_client() -> anyhow::Result<(ClientService, ServerTask)> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = tokio::spawn(async move {
        let service = SearchrightServer::default().serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });
    let client = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("searchright-track10-previous", "1.0.0"),
    )
    .with_protocol_version(ProtocolVersion::V_2025_11_25)
    .serve(client_transport)
    .await?;
    Ok((client, server))
}

fn assert_successful_structured_result(
    tool_name: &str,
    response: CallToolResponse,
) -> anyhow::Result<()> {
    let CallToolResponse::Complete(result) = response else {
        anyhow::bail!("non-task client unexpectedly received a task response")
    };
    if result.is_error.unwrap_or(false) {
        anyhow::bail!("representative read-only tool returned a governed tool error")
    }
    let structured = result
        .structured_content
        .ok_or_else(|| anyhow::anyhow!("{tool_name} did not expose structuredContent"))?;
    if !live_client_output_matches_schema(tool_name, &structured) {
        anyhow::bail!("{tool_name} structuredContent failed its advertised outputSchema")
    }
    Ok(())
}

async fn assert_every_success_path(client: &ClientService) -> anyhow::Result<()> {
    let cases = live_client_success_cases().map_err(anyhow::Error::msg)?;
    assert_eq!(
        cases.len(),
        32,
        "31 tools plus the second PRISMA union branch"
    );
    for case in cases {
        let response = client
            .peer()
            .call_tool_once(
                CallToolRequestParams::new(case.tool_name).with_arguments(case.arguments),
            )
            .await?;
        assert_successful_structured_result(case.tool_name, response)?;
    }
    Ok(())
}

fn assert_complete_advertised_catalogue(tools: &[rmcp::model::Tool]) {
    assert_eq!(tools.len(), 31);
    let names = tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(tools.iter().all(|tool| tool.output_schema.is_some()));
}

fn semantically_invalid_plan_arguments() -> JsonObject {
    let mut arguments = JsonObject::new();
    let document = include_str!("../../../contracts/examples/review-plan.yaml")
        .replace(
            "review_id: demo-paediatric-metabolic-search",
            "review_id: live-client-invalid",
        )
        .replace("strategy-medline-v1", "live-client-invalid")
        .replace("strategy-europe-pmc-v1", "live-client-invalid");
    arguments.insert("document".to_owned(), serde_json::Value::String(document));
    arguments.insert(
        "format".to_owned(),
        serde_json::Value::String("yaml".to_owned()),
    );
    arguments
}

#[tokio::test]
async fn official_rmcp_current_client_consumes_structured_results_and_governed_errors()
-> anyhow::Result<()> {
    let (client, server) = serve_current_client().await?;
    let peer = client
        .peer_info()
        .ok_or_else(|| anyhow::anyhow!("current server peer information is absent"))?;
    assert_eq!(peer.protocol_version, ProtocolVersion::V_2026_07_28);
    assert_complete_advertised_catalogue(&client.list_tools(None).await?.tools);

    assert_every_success_path(&client).await?;

    let governed_error = client
        .peer()
        .call_tool_once(
            CallToolRequestParams::new("validate_plan")
                .with_arguments(semantically_invalid_plan_arguments()),
        )
        .await?;
    let CallToolResponse::Complete(error) = governed_error else {
        anyhow::bail!("malformed request unexpectedly returned a task response")
    };
    assert_eq!(error.is_error, Some(true));
    assert!(error.structured_content.is_none());

    client.cancel().await?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn official_rmcp_previous_era_client_consumes_structured_results_and_governed_errors()
-> anyhow::Result<()> {
    let (client, server) = serve_previous_client().await?;
    let peer = client
        .peer_info()
        .ok_or_else(|| anyhow::anyhow!("previous-era server peer information is absent"))?;
    assert_eq!(peer.protocol_version, ProtocolVersion::V_2025_11_25);
    assert_complete_advertised_catalogue(&client.list_tools(None).await?.tools);

    assert_every_success_path(&client).await?;

    let governed_error = client
        .peer()
        .call_tool_once(
            CallToolRequestParams::new("validate_plan")
                .with_arguments(semantically_invalid_plan_arguments()),
        )
        .await?;
    let CallToolResponse::Complete(error) = governed_error else {
        anyhow::bail!("malformed request unexpectedly returned a task response")
    };
    assert_eq!(error.is_error, Some(true));
    assert!(error.structured_content.is_none());

    client.cancel().await?;
    server.abort();
    Ok(())
}
