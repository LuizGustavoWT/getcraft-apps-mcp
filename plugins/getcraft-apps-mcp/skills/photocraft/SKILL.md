---
name: photocraft
description: Drive PhotoCraft with its native MCP or CLI for layered image inspection, non-destructive edits, batch actions, conversion, and export.
---

# PhotoCraft

Start with `craft_list_apps` and `craft_app_info`. Search the installed registry
with `craft_search_commands`; use `craft_cli_help` before assembling CLI arguments.
Capabilities depend on the installed app, not the gateway version. For persistent
document work, generate a native configuration with `craft_native_mcp_config`.

## Native MCP and document safety

- Headless `read_root` and `write_root` are independent capabilities. An omitted
  root denies that filesystem authority; omitting both still permits in-memory
  edits. Grant only the project directories needed. Use native `doc_open`,
  `doc_save` and `doc_place`, not ambient-path engine commands such as `file.open`.
- Bridge mode uses the desktop app's automation roots; roots on the CLI bridge do
  not restrict it. Start the app with the intended roots and a private
  `--control-token-file`, then configure the bridge with the same token file.
  Do not put the token itself in chat or command arguments.
- Native `command_list` returns a bare array, not a `commands` wrapper. Its
  `params` field is prose, not a JSON schema. Check `enabled`, `disabledReason`
  and `needsDocument` before dispatching; an empty session often needs `doc_new`
  or `doc_open` first. Post-0.9.0 builds also search optional `keywords`; do not
  rely on that field in older catalogs.
- Before experimental edits, use `image.duplicate` or `history.newDocument` to
  preserve the original. Inspect layers and history before and after a batch:
  `command_batch` is not atomic. Earlier successful steps remain applied. If a
  timeout or reply-budget error leaves completion uncertain, inspect before
  retrying. Undo does not remove files already exported.
- Embedded Smart Objects can use `layer.smartObjects.editContents` and
  `layer.smartObjects.saveContents`; save nested contents from the inside out,
  then save the parent. Saving the contents document alone does not update its
  parent. External linked-source reads remain restricted.

## Workflows available in PhotoCraft 0.9.0

Confirm arguments against the installed help/catalog, then use finite
`craft_run_cli` calls for `info`, `convert`, `run`, `batch` or `droplet`:

- Photoshop action sets can be batch inputs:
  `batch --actions /work/actions.atn --action "Resize" --in /work/input --out /work/output`.
  Select an action when the set has several. Inspect warnings and the action
  result's `failed` field: an unsupported Photoshop step can stop playback even
  when the outer action result is otherwise successful. Do not add `--in-place`
  unless replacing the input files was requested.
- Krita `.kra` / `.krz` files are import inputs, not save formats. Export to a new
  supported destination and inspect fidelity warnings. An OpenRaster save marks
  the document saved only when that format can hold the complete document;
  otherwise keep a `.pcraft` master.
- DPX export supports `--dpx-bits 8|10|12|16` on `convert`, `run` and `batch`.
  Native headless `doc_save` / `doc_export` take `dpxBits`; bridge saves reject
  this option because the desktop controls export settings.
- Discover action organization and conditional playback via `actions.assignSet`,
  `actions.dialog` and `actions.conditional`.

Post-0.9.0 builds add named `layer.select` targets. When the installed catalog
advertises them, use `name` and a zero-based `occurrence` for duplicate names in
reusable actions; document-specific numeric IDs are not portable.

Do not start `mcp` or `serve` through the finite runner. The one-shot CLI is a
trusted local process, not the native MCP's filesystem sandbox.

Sources: [MCP conventions](https://github.com/storytold/photocraft/blob/main/docs/mcp.md),
[control protocol](https://github.com/storytold/photocraft/blob/main/docs/control-protocol.md),
[CLI](https://github.com/storytold/photocraft/blob/main/apps/photocraft-cli/src/lib.rs).
