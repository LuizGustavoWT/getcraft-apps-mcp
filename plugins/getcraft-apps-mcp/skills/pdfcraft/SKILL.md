---
name: pdfcraft
description: Use PdfCraft for PDF inspection, text extraction, rendering, editing, splitting, combining, conversion, and faux-redaction audits.
---

# PdfCraft

Call `craft_cli_help` or `craft_command_catalog` before composing a PdfCraft
operation. For multiple document edits or the full tool set, call
`craft_native_mcp_config` with an absolute `workspace`; PdfCraft's `--root`
confines paths named by its MCP tools and scripts.

Start with a read-only `info`, `text`, `render`, or audit operation where it
can answer the request. Before `edit`, `combine`, `extract`, or `split`, use a
new explicit output path and confirm any overwrite. An audit finding indicates
that a visual redaction may still expose the underlying text; report it rather
than calling the PDF safely redacted.

The gateway only runs finite native commands. It intentionally rejects `mcp`
as a nested server command.
