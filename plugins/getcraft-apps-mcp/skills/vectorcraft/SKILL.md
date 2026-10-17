---
name: vectorcraft
description: Use VectorCraft for vector illustration, document structure, geometry, typography, live effects, rendering, and multi-format export through its native MCP or CLI.
---

# VectorCraft

Use `craft_native_mcp_config` for the full first-party VectorCraft MCP. Check
installed help and native tool schemas first: v0.12.0 adds capabilities missing
from v0.11.0, and upstream `main` contains further changes.

## Mode and file access

Headless configuration must explicitly include `--headless`: bare `mcp` first
tries the desktop app on port 7979, then falls back to headless. Set absolute
`read_root` and `write_root` to existing folders. Supplying one root denies the
other kind of access, but omitting **both** disables the root policy; it is not
deny-all. Roots imply headless mode and cannot accompany `--connect`.

For a bridge, start the desktop app with its own `--automation-read-root` and
`--automation-write-root` and then connect without those gateway flags. File
roots are path boundaries, not protection against every process side effect.

## Discover, edit, verify

For a large document, inspect a shallow structure first, find the relevant
object with `document.find`, and request `document.node {id, summary:true,
depth:0}` rather than dumping the entire model. When advertised by the schema,
`doc_inspect` also accepts `ids`, `kind`, `limit` and `offset` (post-v0.12.0).
Native `command_list` in v0.12.0 accepts `query`, `menu` and `brief` to narrow a
catalog; request full parameter descriptions before executing a command.

Use a small native `command_batch` with `steps: [{id, params}]`; it accepts at
most 256 steps. In v0.12.0, `draw_path` and `draw_shape` can be step IDs and return
new IDs in `created`; `$0.id` can reference a prior drawing result. Their style
options include `dash`, `cap`, `join`, `opacity` (0–100), `name` and `effects`.
Discover the exact shape/effect schema instead of guessing parameters. Do not
confuse the MCP `command_batch` with engine `command.batch`, which takes
`commands: [{command, params}]`.

`atomic: true` groups edits into an undo step; it does not roll back files
already written by a failed batch. Read per-step results before retrying. Render
the edited artboard to verify output, export to an explicit write-root path,
and preserve the editable document before lossy conversion. Artboard indexes
are 0-based; the CLI export `--range` is 1-based.

For `.ai` conversion, v0.12.0 adds `convert IN OUT --ai-version 7` (legacy) or
`--ai-version pdf`, with `--hidden-objects layer|omit|visible` and
`--transparent-images flatten|omit` for legacy output. Keep import/export
warnings and do not add `--replace-lossy` to overwrite a degraded source without
an explicit request.

Newer native MCP builds support progress tokens, cancellation and busy error
`-32003`; these are not present in v0.12.0. A busy refusal did not run. Cancelling
a connected request stops waiting but cannot stop work already started in the
desktop app; inspect before retrying. Gateway CLI calls do not expose this native
progress/cancellation protocol.

`craft_run_cli` is useful for finite `commands`, `run`, `convert`, `info`,
`bench`, or `perf` calls. It cannot launch a nested MCP server.

Reference: [native MCP, CLI and command guide](https://github.com/storytold/vectorcraft/blob/main/docs/mcp.md).
