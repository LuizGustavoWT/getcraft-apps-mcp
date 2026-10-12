---
name: filmcraft
description: Drive FilmCraft projects through the GetCraft gateway or FilmCraft's native MCP/CLI when the user needs video editing, project inspection, preview rendering, or export.
---

# FilmCraft

Use `craft_list_apps` first when FilmCraft has not already been detected. Use
`craft_native_mcp_config` to generate FilmCraft's native configuration when the
full command, project, sequence, render, or live-UI tool surface is needed.

For headless work, inspect the project/sequence before an edit, apply one
undoable command or batch, render a preview, then inspect again. Use bridge
mode only after the user has started FilmCraft with the matching control port.

FilmCraft's native MCP does not document filesystem-root confinement flags.
Treat every import, save, and export path as sensitive: use explicit absolute
paths, do not overwrite an existing output unless asked, and ask before a
destructive replacement. Do not start `mcp` via `craft_run_cli`; the gateway
blocks nested servers by design.
