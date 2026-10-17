---
name: pdfcraft
description: Use PdfCraft for PDF inspection, editing, rendering, text extraction, batch conversion/OCR/compression, accessibility workflows, and faux-redaction audits.
---

# PdfCraft

Call `craft_cli_help` and `craft_command_catalog` before composing an operation;
the installed CLI may lag upstream `main`. Use the returned schemas, not a tool
name inferred from a GUI label. Pages are **1-based**; rectangles use points
from the top-left of the displayed page.

## Native sessions and file access

For multiple edits, use `craft_native_mcp_config` with an absolute `workspace`.
It becomes `--root`, which confines reads and writes; relative tool paths resolve
inside that root. PdfCraft's MCP is headless only, not a connection to an open
desktop document. The separate `pdfcraft-cli ui --control FILE ...` interface
requires an application explicitly started with `--control FILE`.

Newer upstream builds also accept `--automation-read-root` and
`--automation-write-root`; the audited v0.8.0 release does not. Use the gateway's
`read_root` / `write_root` only when the installed CLI supports those flags. The
folders must exist. Supplying one root denies the other kind of access; omitting
both leaves this extra policy disabled. If `workspace` is also supplied, both
the `--root` boundary and the matching automation root apply. Do not silently
drop an unsupported requested restriction.

For a small tool listing, native `mcp --compact` exposes `tool_search` and
`tool_call` alongside core tools. Search by name for the complete input schema.
The gateway runs finite commands only; it cannot host a nested `mcp` server.

## Editing and newer workflows

Use `run --script steps.json --root DIR` for several steps in one CLI session;
separate CLI invocations do not retain document IDs. Scripts contain
`[{"tool":"doc_open","args":{"path":"input.pdf"}}, ...]`. Newer builds offer
`--report` for one JSON report; check help before adding it.

For folder processing, `batch INPUT --steps convert,ocr,optimize --out-dir OUTPUT`
supports `--recursive`, `--root DIR` and `--stop-on-error`; the native tool is
`doc_batch`. Choose only the requested steps, with `convert` first when present.
OCR needs already-installed models. `optimize` may lose image detail. Read the
per-file report even on a nonzero exit: earlier successful outputs remain,
existing output names get numbered alternatives, and sources are not replaced.

For text searches, follow `text_find.next` when present and inspect `warnings`:
`complete: false` without `next` can mean unreadable pages, not more results.
Discover `text_word_count`, paginated `accessibility_tags`, and other newer
accessibility tools from the installed catalog before using them; they are not
all present in v0.8.0.

Start with a read-only `info`, `text`, `render`, or audit operation where it
can answer the request. Before `edit`, `combine`, `extract`, or `split`, use a
new explicit output path and confirm any overwrite. An audit finding indicates
that a visual redaction may still expose the underlying text; report it rather
than calling the PDF safely redacted. For `link_open`, review an absolute GoToR
target before opting into `allow_absolute: true`; this never bypasses roots.

Reference: [native MCP and confinement](https://github.com/storytold/pdfcraft/blob/main/docs/mcp.md)
and [folder batches](https://github.com/storytold/pdfcraft/blob/main/docs/cli-batch.md).
