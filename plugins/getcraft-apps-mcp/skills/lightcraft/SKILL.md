---
name: lightcraft
description: Use LightCraft for photo-library workflows, raw development, masking, rendering, contact sheets, and export through the native MCP or CLI.
---

# LightCraft

Discover the CLI with `craft_list_apps`. Use `craft_native_mcp_config` for
native LightCraft tools; pass an absolute `workspace` only when a persistent
library is intended. A headless MCP process cannot open a library that the
desktop app has locked. Use bridge mode to edit that live library instead.

For a photo workflow, inspect/query first, make a small develop or mask edit,
render a preview, then export to an explicit new destination. Confirm before
copy/move imports, deleting photos, or overwriting an export. Prefer the native
MCP's image responses to guessing how an adjustment changed the photograph.

Use the finite gateway command runner for one-shot work and native config for
the persistent MCP server.
