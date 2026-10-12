---
name: effectcraft
description: Use EffectCraft for compositing, motion graphics, effects, scripts, render previews, and exports through its native MCP or finite CLI commands.
---

# EffectCraft

Use `craft_command_catalog` to discover exact command ids and parameter
schemas. Generate native MCP configuration with `craft_native_mcp_config` for
composition inspection, command execution, rendering, or a bridge to the live
desktop app.

Inspect the project and composition before editing. Make edits through one
command or a small batch, render a frame or preview, then inspect again. A
bridge requires a running EffectCraft instance started with its control port.
Headless work normally lives only for the server session; use its documented
`--autosave` capability only when persistence is intended, and save to an
explicit project path before disconnecting.

Use `craft_run_cli` for finite `info`, `commands`, `exec`, `run`, or render
operations, never to launch the native MCP itself.
