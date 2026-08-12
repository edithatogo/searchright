//! Standard-I/O MCP server for Searchright's governed review operations.

#![forbid(unsafe_code)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    searchright_mcp::run_stdio().await
}
