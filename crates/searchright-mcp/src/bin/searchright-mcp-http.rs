//! Authenticated loopback Streamable HTTP MCP deployment adapter.

#![forbid(unsafe_code)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    searchright_mcp::remote::run_from_environment().await
}
