---
name: vectorcraft
description: Use VectorCraft for vector illustration, document structure, geometry, typography, live effects, rendering, and multi-format export through its native MCP or CLI.
---

# VectorCraft

Use `craft_native_mcp_config` for the full first-party VectorCraft MCP. In
headless mode set distinct, absolute `read_root` and `write_root` values; omit
either one to deny that kind of access. Do not use a bridge with those gateway
flags: the running app must be started with the matching roots instead.

For a large document, inspect a shallow structure first, find the relevant
object, then request that object rather than dumping the entire model. Discover
commands before executing them, apply a small command batch, and render an
artboard or preview to verify visual output. Export to an explicit write-root
path and preserve the editable document before lossy conversion.

`craft_run_cli` is useful for finite `commands`, `run`, `convert`, `info`,
`bench`, or `perf` calls. It cannot launch a nested MCP server.
