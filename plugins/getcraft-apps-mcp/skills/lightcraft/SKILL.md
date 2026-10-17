---
name: lightcraft
description: Use LightCraft for photo-library workflows, raw development, masking, HDR and panorama merges, slideshows, and photo export through the native MCP or CLI.
---

# LightCraft

Discover the CLI with `craft_list_apps` and check its version with `craft_app_info`.
Use `craft_search_commands` for engine IDs and parameter descriptions; its
`kind: "controls"` discovers develop-slider IDs and ranges. Check
`craft_cli_help` before building one-shot commands. Do not infer that a feature
on upstream `main` exists in an installed release.

## Choose the session

Use `craft_native_mcp_config` for native LightCraft tools. An absolute `workspace`
opens or creates a persistent library; it is not a filesystem sandbox. A headless
MCP process cannot open a library locked by the desktop app. Use bridge mode
(`--connect 127.0.0.1:7980`, matching the app's `--control` port) for that library.
The CLI and desktop should be from the same release. LightCraft 0.8.0 also
accepts a bare connect port, but an explicit loopback address is unambiguous.

Use `compact: true` when the native tool list is too large. It removes generated
`cmd_*` entries, not their capabilities: `command_run {id, params}` still reaches
the registry. In full mode, replace each `.` in the command ID with `_` and add
`cmd_` to find its generated tool.

For a photo workflow, query first, apply a small edit, render a preview and export
to an explicit destination. Native `get_develop {id}` and `render_photo {id}` do
not change selection; `set_develop`, `crop` and `export` with `id` replace it with
that photo. Export without IDs acts on the selection; provide explicit IDs for
batch work. With native `export`, `longEdge: 0` requests full cropped resolution
(omitting size defaults to 3000 px); the one-shot `render` CLI defaults to full size.

Copy/move imports, deletions and overwrites require the user's intended scope.
Do not turn a native-MCP filesystem restriction into a CLI workaround. A bridge
error saying the outcome is unknown must be followed by state inspection before
retrying: the command may already have run. The same applies to a library-save
error saying a change was applied in memory but not written to disk.

## Newer-build workflows

These were added on upstream `main` after release 0.8.0. Require the corresponding
installed help/catalog entries before using them; do not claim they are available
merely because the plugin has been updated.

- `render INPUT -o OUTPUT --settings LOOK.lcpreset` can apply a single exported
  preset, or an XMP preset; partial settings JSON remains supported. A multi-preset
  file is rejected rather than choosing one silently.
- `merge hdr-batch FILES...` / engine `merge.hdrBatch` discovers bracket sets among
  the supplied photos. Check the documented grouping and output options before
  writing DNGs; keep originals and inspect the resulting merge report.
- For slideshows, discover `slideshow.layout`, `slideshow.overlays`,
  `slideshow.titles`, `slideshow.templates.list` and `slideshow.preview`. Preview
  first, then `slideshow.export` to PDF, a JPEG sequence, or AV1 video (no audio).
  Prefer a persistent native MCP session for background work: retain `taskId`,
  inspect `activity.list` / `slideshow.exportStatus`, and cancel only the intended
  task with `activity.cancel`.
- After a decoder update, headless libraries can run `library.recheckPreviewOnly`
  to revisit raws previously imported as embedded-preview-only. It preserves
  develop settings/history and is not an undo step; inspect `checked`, `updated`
  and `cleared`. Use `all: true` only when ignoring version stamps is intended.
- `photo.readFocusPoints` reads recorded camera focus metadata for older imports;
  inspect `photo.inspect` / `lightcraft://photo/active` rather than guessing focus.

Use the finite runner for `run`, `render`, `merge`, `snapshot`, `commands` or
`controls`, not `mcp` or the persistent `denoise-serve`. For native headless export
progress/cancellation, call `export` or `command_run` for `app.export` directly;
bridge exports and batch-contained exports do not provide that MCP behavior.
Cancellation preserves photos already exported, so inspect outputs before retrying.

Sources: [native MCP](https://github.com/storytold/lightcraft/blob/main/docs/mcp.md),
[CLI](https://github.com/storytold/lightcraft/blob/main/apps/lightcraft-cli/src/main.rs).
