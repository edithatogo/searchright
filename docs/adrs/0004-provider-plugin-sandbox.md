# ADR 0004: WASI component provider plugins

Status: proposed

Mature third-party connectors will use WIT contracts and capability-scoped WASI
components. Native in-process plugins are not accepted by default because they
expand the memory-safety, egress and supply-chain boundary.
