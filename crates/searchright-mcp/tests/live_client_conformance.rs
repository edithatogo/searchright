//! Track 10 official `rmcp` client conformance for supported local stdio eras.
//!
//! This launches the real `searchright-mcp` binary through the SDK's child-process
//! stdio transport. It proves that the typed client API can negotiate each
//! declared protocol era, invoke every successful tool shape, independently
//! validate `structuredContent` against the schema observed through `tools/list`,
//! and preserve governed errors without a wire-format shim.

use rmcp::{
    ClientLifecycleMode, ClientServiceExt, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, ClientCapabilities, ClientInfo, Implementation,
        JsonObject, ProtocolVersion,
    },
    service::RoleClient,
};
use searchright_mcp::live_client_success_cases;
use std::{collections::BTreeMap, process::Stdio};
use tokio::process::{ChildStdin, ChildStdout, Command};

type ClientService = rmcp::service::RunningService<RoleClient, ClientInfo>;
type ChildTransport = (ChildStdout, ChildStdin);

fn child_transport() -> anyhow::Result<ChildTransport> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_searchright-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("child stdout was not piped"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("child stdin was not piped"))?;
    Ok((stdout, stdin))
}

async fn serve_current_client() -> anyhow::Result<ClientService> {
    let client = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("searchright-track10-current", "1.0.0"),
    )
    .serve_with_lifecycle(
        child_transport()?,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![ProtocolVersion::V_2026_07_28],
        },
    )
    .await?;
    Ok(client)
}

async fn serve_previous_client() -> anyhow::Result<ClientService> {
    let client = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new("searchright-track10-previous", "1.0.0"),
    )
    .with_protocol_version(ProtocolVersion::V_2025_11_25)
    .serve(child_transport()?)
    .await?;
    Ok(client)
}

fn assert_successful_structured_result(
    tool_name: &str,
    output_schema: &serde_json::Value,
    response: CallToolResponse,
) -> anyhow::Result<()> {
    let CallToolResponse::Complete(result) = response else {
        anyhow::bail!("non-task client unexpectedly received a task response")
    };
    if result.is_error.unwrap_or(false) {
        anyhow::bail!("{tool_name} returned a governed tool error")
    }
    let structured = result
        .structured_content
        .ok_or_else(|| anyhow::anyhow!("{tool_name} did not expose structuredContent"))?;
    validate_observed_schema(output_schema, &structured).map_err(|error| {
        anyhow::anyhow!("{tool_name} structuredContent failed outputSchema: {error}")
    })?;
    Ok(())
}

fn validate_observed_schema(
    schema: &serde_json::Value,
    value: &serde_json::Value,
) -> anyhow::Result<()> {
    let object = schema
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("schema is not an object"))?;
    if let Some(expected) = object.get("const") {
        anyhow::ensure!(value == expected, "const mismatch");
    }
    if let Some(allowed) = object.get("enum").and_then(serde_json::Value::as_array) {
        anyhow::ensure!(allowed.contains(value), "enum mismatch");
    }
    if let Some(branches) = object.get("oneOf").and_then(serde_json::Value::as_array) {
        let matches = branches
            .iter()
            .filter(|branch| validate_observed_schema(branch, value).is_ok())
            .count();
        anyhow::ensure!(matches == 1, "oneOf matched {matches} branches");
    }
    if let Some(expected_type) = object.get("type") {
        let matches = match expected_type {
            serde_json::Value::String(name) => observed_type_matches(value, name),
            serde_json::Value::Array(names) => names
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|name| observed_type_matches(value, name)),
            _ => false,
        };
        anyhow::ensure!(matches, "type mismatch");
    }
    match value {
        serde_json::Value::Object(values) => {
            if let Some(required) = object.get("required").and_then(serde_json::Value::as_array) {
                for field in required.iter().filter_map(serde_json::Value::as_str) {
                    anyhow::ensure!(values.contains_key(field), "missing required field {field}");
                }
            }
            let properties = object
                .get("properties")
                .and_then(serde_json::Value::as_object);
            for (field, field_value) in values {
                if let Some(field_schema) = properties.and_then(|items| items.get(field)) {
                    validate_observed_schema(field_schema, field_value)?;
                } else {
                    match object.get("additionalProperties") {
                        Some(serde_json::Value::Bool(false)) => {
                            anyhow::bail!("unexpected field {field}")
                        }
                        Some(serde_json::Value::Object(additional)) => validate_observed_schema(
                            &serde_json::Value::Object(additional.clone()),
                            field_value,
                        )?,
                        _ => {}
                    }
                }
            }
        }
        serde_json::Value::Array(values) => {
            if let Some(minimum) = object.get("minItems").and_then(serde_json::Value::as_u64) {
                let length = u64::try_from(values.len())?;
                anyhow::ensure!(length >= minimum, "array is too short");
            }
            if object
                .get("uniqueItems")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                for (index, item) in values.iter().enumerate() {
                    let is_duplicate = values.iter().take(index).any(|previous| previous == item);
                    anyhow::ensure!(!is_duplicate, "duplicate array item");
                }
            }
            if let Some(item_schema) = object.get("items") {
                for item in values {
                    validate_observed_schema(item_schema, item)?;
                }
            }
        }
        serde_json::Value::String(text) => {
            if let Some(minimum) = object.get("minLength").and_then(serde_json::Value::as_u64) {
                let length = u64::try_from(text.chars().count())?;
                anyhow::ensure!(length >= minimum, "string is too short");
            }
            if let Some(pattern) = object.get("pattern").and_then(serde_json::Value::as_str) {
                let valid = match pattern {
                    "^[a-f0-9]{64}$" => {
                        text.len() == 64
                            && text
                                .bytes()
                                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    }
                    "^[a-z0-9][a-z0-9_.-]*$" => text.bytes().enumerate().all(|(index, byte)| {
                        (byte.is_ascii_lowercase() || byte.is_ascii_digit())
                            || (index > 0 && matches!(byte, b'_' | b'.' | b'-'))
                    }),
                    "^(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)(?:\\.(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?))*$" => {
                        text.split('.').all(observed_hostname_label_is_valid)
                    }
                    _ => anyhow::bail!("unsupported observed pattern {pattern}"),
                };
                anyhow::ensure!(valid, "pattern mismatch");
            }
        }
        serde_json::Value::Number(number) => {
            if let Some(minimum) = object.get("minimum").and_then(serde_json::Value::as_f64) {
                anyhow::ensure!(
                    number.as_f64().is_some_and(|value| value >= minimum),
                    "number below minimum"
                );
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) => {}
    }
    Ok(())
}

fn observed_hostname_label_is_valid(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && !label.starts_with('-')
        && !label.ends_with('-')
}

fn observed_type_matches(value: &serde_json::Value, expected: &str) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "integer" => value
            .as_number()
            .is_some_and(|number| number.is_i64() || number.is_u64()),
        _ => false,
    }
}

async fn assert_every_success_path(client: &ClientService) -> anyhow::Result<()> {
    let advertised = client
        .list_tools(None)
        .await?
        .tools
        .into_iter()
        .map(|tool| {
            let schema = tool
                .output_schema
                .ok_or_else(|| anyhow::anyhow!("{} lacks outputSchema", tool.name))?;
            Ok((
                tool.name.to_string(),
                serde_json::Value::Object((*schema).clone()),
            ))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let cases = live_client_success_cases().map_err(anyhow::Error::msg)?;
    assert_eq!(
        cases.len(),
        32,
        "31 tools plus the second PRISMA union branch"
    );
    let mut failures = Vec::new();
    for case in cases {
        let response = client
            .peer()
            .call_tool_once(
                CallToolRequestParams::new(case.tool_name).with_arguments(case.arguments),
            )
            .await;
        let result = response.map_err(anyhow::Error::from).and_then(|response| {
            let schema = advertised
                .get(case.tool_name)
                .ok_or_else(|| anyhow::anyhow!("{} is not advertised", case.tool_name))?;
            assert_successful_structured_result(case.tool_name, schema, response)
        });
        if let Err(error) = result {
            failures.push(format!("{}: {error}", case.tool_name));
        }
    }
    anyhow::ensure!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn assert_complete_advertised_catalogue(tools: &[rmcp::model::Tool]) {
    assert_eq!(tools.len(), 31);
    let names = tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();
    assert!(
        names
            .windows(2)
            .all(|pair| matches!(pair, [left, right] if left < right))
    );
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
    let client = serve_current_client().await?;
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
    Ok(())
}

#[tokio::test]
async fn official_rmcp_previous_era_client_consumes_structured_results_and_governed_errors()
-> anyhow::Result<()> {
    let client = serve_previous_client().await?;
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
    Ok(())
}
