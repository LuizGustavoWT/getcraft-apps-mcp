---
name: filmcraft
description: Drive FilmCraft projects through the GetCraft gateway or FilmCraft's native MCP/CLI when the user needs video editing, project inspection, preview rendering, or export.
---

# FilmCraft

Use `craft_list_apps` first when FilmCraft has not already been detected. Use
`craft_native_mcp_config` to generate FilmCraft's native configuration when the
full command, project, sequence, render, or live-UI tool surface is needed.

## Session and discovery

`filmcraft-cli mcp` starts an empty in-memory project; `--project /path/p.fcproj`
opens an existing project, `--demo` loads the demo, and `--data-dir DIR` selects
the user-preset library. Bridge mode uses `--bridge 127.0.0.1:9876`, with an
already-running desktop app started with `filmcraft --control 9876`; unlike
EffectCraft, the bridge needs a loopback host and port, not a bare port.

Discover the installed surface rather than assuming the latest release:
`craft_command_catalog` or `filmcraft-cli commands --json`, then
`filmcraft-cli describe <id>` for the selected command's parameters. Native MCP
offers `command_list`, `command_run`, `command_batch`, `doc_inspect`, and
`render_preview`; `filmcraft://document` and `filmcraft://commands` are JSON
resources. Get ids from project/sequence inspection, edit, render a preview,
and inspect again.

Each finite CLI invocation has its own session. Pass `--project` to target the
intended project and `--save-as` to preserve edits in a new file; `--save`
overwrites the project passed to `--project`. A JSONL script passed to `run`
keeps its steps in one session. Do not assume one gateway invocation's edits
will be visible in the next without saving or a live bridge.

## Editing capabilities to discover

- `edit.rippleDeleteHead {"clips":[id]}` removes the selected clip heads up to
  the playhead and closes the gap. Linked clips move as a group; a playhead
  beyond the group's out points deletes the group. Inspect time and targets
  before running it.
- `timeline.select {"clips":[id],"linked":false}` selects an individual
  member of a linked pair in builds newer than v0.8.0; require the installed
  descriptor to advertise `linked` before using it. When moving only that member, also use the
  documented `timeline.move` `linked:false` parameter; ordinary moves keep
  linked picture and sound together.
- `timeline.move` accepts `track:"new"` in its `moves` entries; times are
  integer ticks, not seconds (254016000000 ticks per second). Consult the
  command descriptor for commands that also accept seconds or frames.

## Export and file access

For long exports prefer the persistent native MCP over the gateway's finite
CLI timeout. `command_run` with `file.exportMedia` and `wait:true` supports MCP
progress/cancellation in headless mode. Otherwise poll `jobs.list` and cancel
with `jobs.cancel` in the same session; the live desktop bridge also keeps
jobs alive. A new headless CLI process cannot poll a previous process's jobs.

Check `media.status` or `media.findMissing` before exporting. CLI `export` and
`render` may write the output and then exit 1 because media was offline; inspect
`offlineMedia`, stderr, and the written file before retrying. Use
`--allow-offline` only when placeholder frames are wanted. Native exports can
succeed with a nonempty `offlineMedia` list, so success alone is not a fidelity
check.

FilmCraft's native MCP has no filesystem-root confinement flags.
Treat every import, save, and export path as sensitive: use explicit absolute
paths, do not overwrite an existing output unless asked, and ask before a
destructive replacement. Do not start `mcp` via `craft_run_cli`; the gateway
blocks nested servers by design.

Authoritative reference: [FilmCraft agent and CLI documentation](https://github.com/storytold/filmcraft/blob/main/docs/agents.md).
