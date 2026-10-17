---
name: designcraft
description: Use DesignCraft for page layout, typography, frames, styles, placed images, page rendering, and export through its native MCP or CLI.
---

# DesignCraft

Use `craft_list_apps` for discovery and `craft_native_mcp_config` when you
need DesignCraft's document, story, page-render, and live-UI tools. Use
headless mode for a new or isolated layout; use bridge mode only after the
desktop application is running on the selected control port. Inspect the
installed CLI help and native tool schemas: upstream `main` can contain features
not yet in a release.

## File access and connection mode

Newer builds support headless `mcp --automation-read-root DIR
--automation-write-root DIR`, represented by the gateway's `read_root` and
`write_root`. The audited v0.8.0 release does not support these flags. Only use
them after checking installed help, and never silently drop a requested root.
Folders must exist; supplying just one root denies the other kind of access,
while omitting both leaves access unconfined. Referenced images, book chapters,
data sources and adjacent Document Fonts are checked against the read root too.

`--connect` cannot be combined with these root flags. The desktop control mode
has no equivalent confinement, so do not present a bridge as sandboxed. If a
connected request fails after sending, inspect the document before retrying:
it may already have run; the next call reconnects without replaying it.

## Layout and text

Inspect the document, pages, and stories before layout changes. Coordinates are
points in spread space and page indexes are zero-based. Text offsets in the
native API are UTF-8 byte offsets, not character counts; get the story first
instead of calculating an offset from a display character position.

Use native `list_commands` / CLI `describe COMMAND` for parameters before
`execute` or `batch`. Newer MCP builds reject unknown command keys and validate
all batch steps before starting. Use `commands: [{command, params}]` for a native
batch; result references such as `$0.story` refer to earlier steps. In newer
script parsers `$$` escapes a literal dollar sign, including GREP replacements.

Newer native schemas expose `new_document` options such as `intent`, `units`,
`startPage`, `slug` and `primaryTextFrame`, and an `ime` tool with `preedit` /
`commit`. Discover them before use; IME's `active` clause uses character offsets,
unlike the story APIs' byte offsets.

Hyphenation defaults on. Disable it for selected paragraphs or selected-frame
stories with `execute {"command":"type.para","params":{"attrs":{"hyphenate":false}}}`.
For images, `object.fit` mode `fillProportionally` crops to cover the frame,
`fitProportionally` contains the image, and `fitContentToFrame` stretches it.

Render the target page after a structural or style edit and inspect the image
before saving or exporting. `render_page` without `path` returns an inline PNG;
with `path` it also writes a file, so account for that output. Inspect story
overset before delivery.

Reference: [native MCP guide](https://github.com/storytold/designcraft/blob/main/docs/mcp.md).
