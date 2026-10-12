---
name: photocraft
description: Drive PhotoCraft with its native MCP or CLI for layered image inspection, non-destructive edits, batch actions, conversion, and export.
---

# PhotoCraft

Start with `craft_list_apps`; use `craft_cli_help` or `craft_command_catalog`
instead of guessing a command id or parameter. For detailed document work,
generate a native MCP configuration with `craft_native_mcp_config`.

In headless native MCP mode, supply the narrowest possible
`read_root` and `write_root`. Omit a root rather than granting access by
default. Before trying an edit, duplicate the image or open a new history
document when the original must be preserved. Inspect layers and history before
and after a command batch; a failed batch is not a rollback.

Use the direct CLI gateway only for finite commands such as `info`, `convert`,
`run`, or `batch`. Do not attempt to pass `mcp` or `serve` to `craft_run_cli`.
