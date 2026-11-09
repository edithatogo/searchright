# syntax=docker/dockerfile:1.8
FROM rust:1.97.1-slim-bookworm@sha256:2775a09d208ff0d7c1f50490c45b62db929e87ba1dcbc3f2132ac71a704bcdd3 AS builder
WORKDIR /src
COPY . .
RUN test -f Cargo.lock && cargo build --locked --release -p searchright-mcp

FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
LABEL org.opencontainers.image.source="https://github.com/edithatogo/searchright" \
      org.opencontainers.image.description="Searchright systematic-search MCP server" \
      io.modelcontextprotocol.server.name="io.github.edithatogo/searchright"
RUN useradd --create-home --uid 10001 searchright
COPY --from=builder /src/target/release/searchright-mcp /usr/local/bin/searchright-mcp
USER searchright
ENTRYPOINT ["/usr/local/bin/searchright-mcp"]
