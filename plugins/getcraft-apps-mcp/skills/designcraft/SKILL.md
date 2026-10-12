---
name: designcraft
description: Use DesignCraft for page layout, typography, frames, styles, placed images, page rendering, and export through its native MCP or CLI.
---

# DesignCraft

Use `craft_list_apps` for discovery and `craft_native_mcp_config` when you
need DesignCraft's document, story, page-render, and live-UI tools. Use
headless mode for a new or isolated layout; use bridge mode only after the
desktop application is running on the selected control port.

Inspect the document, pages, and stories before layout changes. Coordinates are
points in spread space and page indexes are zero-based. Text offsets in the
native API are UTF-8 byte offsets, not character counts; get the story first
instead of calculating an offset from a display character position.

Render the target page after a structural or style edit and inspect the image
before saving or exporting. Use `craft_cli_help` for current CLI syntax.
