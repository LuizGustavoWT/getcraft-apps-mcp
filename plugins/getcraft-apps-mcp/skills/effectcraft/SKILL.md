---
name: effectcraft
description: Use EffectCraft for compositing, motion graphics, effects, scripts, render previews, and exports through its native MCP or finite CLI commands.
---

# EffectCraft

Use `craft_command_catalog` to discover exact command ids and parameter
descriptions. For JSON Schemas use `effectcraft-cli exec --list --schemas
--json`, or native `command_list {"schemas":true}` and `describe_command`.
Generate native MCP configuration with `craft_native_mcp_config` for
composition inspection, command execution, rendering, or a bridge to the live
desktop app.

## Sessions and file access

`effectcraft-cli mcp` starts empty; add `--project /path/p.ecproj` or `--demo`
when intended. Finite CLI commands instead default to the demo: use `--empty`
for a new project or `--project` for an existing one. Each invocation has its
own headless session; use `--save-as` for a new saved project or `--save` when
overwriting the input is intended.

Inspect the project and composition before editing. Make edits through one
command or a small batch, render a frame or preview, then inspect again.
Native `batch` groups edits into one undo step with rollback; `command_batch`
has separate undo steps and may leave completed edits after a failure.

A bridge requires a running `effectcraft --control 9877`; native MCP uses
`--bridge 9877` (a loopback host:port also works). If a connection fails after
an edit, its outcome may be unknown. Inspect the app before explicitly
retrying; a lost response does not mean that no change was applied.

Native MCP accepts repeatable `--automation-read-root DIR` and
`--automation-write-root DIR`. Prefer narrow roots for file-based work: reads
are allowed under either kind of root, writes only under a write root. Paths
must be absolute and inside the permitted roots. In confined mode scripts,
UI-driving tools, and batch result references to file paths are refused; do
not silently remove confinement to make them work. Without any root flags,
the native MCP is not confined.

Headless work normally lives only for the server session; use its documented
`--autosave` capability only when persistence is intended, and save to an
explicit project path before disconnecting.

## Capabilities to discover in updated builds

Check the installed command catalog before using these commands:

- Object Matte: `objectMatte.apply {layer, point:[x,y], box?:[x0,y0,x1,y1]}`
  prompts in layer pixels. Inspect with `objectMatte.status`, propagate using
  `objectMatte.propagate {layer, wait:true}`, then freeze if wanted. The
  `frame` parameter uses layer frames, not comp seconds. A trained model,
  when installed, supports the box prompt; do not download one implicitly.
- Quick Offset: `layer.quickOffset {layers:[...], delta:seconds}` or
  `keys.quickOffset {delta:seconds}` staggers at least two unlocked layers
  (or selected keys across layers) in selection order. The first stays and
  the last moves by the full delta in comp seconds; intermediate offsets
  can land on subframes.
- `project.usedIn {item}` reports direct layer uses and the compositions
  containing them; it is not a recursive dependency count.
- `prop.reset` preserves existing keyframes on animated properties and keys
  the default at the current time. Do not use it as a shortcut for deleting
  animation.

For other keyframe edits discover paths with `get_layer` / CLI `props` first.
Native property/keyframe writes default to layer time; pass `timeBase:"comp"`
when using comp seconds on offset or stretched layers.

## Rendering

For long jobs use the persistent native MCP, not the gateway's finite CLI
timeout. Headless direct `command_run` / `execute_command` of
`renderQueue.render` with `wait:true` supports MCP progress and cancellation.
For `wait:false` or bridge work, poll `renderQueue.list` and stop with
`renderQueue.stop` in that same session. Edits during a render affect the
project, not the job's already-captured snapshot.

Inspect failed queue items even when other outputs succeeded. Recent builds
return exit 1 for a failed `renderQueue.render` inside CLI `run`/`exec`; older
builds may expose failures only in the result. A headless session uses CPU
rendering unless explicitly started with `--gpu`, which fails if no adapter
is usable.

Use `craft_run_cli` for finite `info`, `commands`, `exec`, `run`, or render
operations, never to launch the native MCP itself.

References: [agent and CLI guide](https://github.com/storytold/effectcraft/blob/main/docs/agents.md)
and [MCP confinement and rendering](https://github.com/storytold/effectcraft/blob/main/docs/mcp.md).
