# syntax=docker/dockerfile:1.8
FROM rust:1.97.1-slim-bookworm AS builder
WORKDIR /src
COPY . .
RUN test -f Cargo.lock && cargo build --locked --release -p searchright-mcp

FROM debian:bookworm-slim
LABEL org.opencontainers.image.source="https://github.com/edithatogo/searchright" \
      org.opencontainers.image.description="Searchright systematic-search MCP server" \
      io.modelcontextprotocol.server.name="io.github.edithatogo/searchright"
RUN useradd --create-home --uid 10001 searchright
COPY --from=builder /src/target/release/searchright-mcp /usr/local/bin/searchright-mcp
USER searchright
ENTRYPOINT ["/usr/local/bin/searchright-mcp"]
