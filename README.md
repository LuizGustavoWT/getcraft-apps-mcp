# GetCraft Apps MCP

[Português (Brasil)](README.pt-BR.md) · [Contributing](CONTRIBUTING.md) · [Contribuindo](CONTRIBUTING.pt-BR.md)

One Rust MCP gateway and one installable plugin bundle for the Storytold Craft applications:
[FilmCraft](https://github.com/storytold/filmcraft), [PhotoCraft](https://github.com/storytold/photocraft), [PdfCraft](https://github.com/storytold/pdfcraft), [DesignCraft](https://github.com/storytold/designcraft), [EffectCraft](https://github.com/storytold/effectcraft), [LightCraft](https://github.com/storytold/lightcraft), and [VectorCraft](https://github.com/storytold/vectorcraft).

The gateway deliberately does not reimplement their feature-rich MCP servers. It discovers installed native CLIs, obtains their current help and command catalogs, runs finite commands without a shell, and generates copyable configuration for an app's own MCP server. The bundled skills give Codex and Claude Code app-specific workflows and safety notes.

## What is included

| Component | Purpose |
| --- | --- |
| `getcraft-mcp serve` | Stdio MCP server written in Rust with five tools: `craft_list_apps`, `craft_cli_help`, `craft_command_catalog`, `craft_run_cli`, and `craft_native_mcp_config`. |
| `plugins/getcraft-apps-mcp/` | One plugin directory compatible with Codex and Claude Code. |
| Seven skills | Targeted FilmCraft, PhotoCraft, PdfCraft, DesignCraft, EffectCraft, LightCraft, and VectorCraft instructions. |
| Two marketplaces | `.agents/plugins/marketplace.json` for Codex and `.claude-plugin/marketplace.json` for Claude Code. |

## Security model

`craft_run_cli` calls an installed CLI directly—never through a shell—and rejects the `mcp` and `serve` subcommands so it cannot start a nested persistent server. It caps argument count, argument size, output capture, and execution time.

The first-party MCP server is the preferred choice for detailed editing because it exposes the application's actual tools. Use `craft_native_mcp_config` to generate its configuration. When the app supports filesystem confinement, give it the smallest possible paths:

| App | Native MCP confinement / operational note |
| --- | --- |
| FilmCraft | Its agent documentation does not offer filesystem-root flags; use only trusted explicit paths. |
| PhotoCraft | Separate `--automation-read-root` and `--automation-write-root` flags; omitted authority stays absent. |
| PdfCraft | `--root` confines all MCP reads and writes. |
| DesignCraft | Use the documented headless or live control-port mode. |
| EffectCraft | Use native MCP for full command schemas; `--autosave` intentionally persists headless checkpoints. |
| LightCraft | `--library` is persistent and locked by one process at a time; use connect mode for an open desktop library. |
| VectorCraft | Separate `--automation-read-root` and `--automation-write-root` flags; omitted authority stays absent. |

## Prerequisites

- Rust 1.88 or newer.
- At least one supported native Craft CLI on `PATH`, or an absolute path in the corresponding environment variable: `GETCRAFT_FILMCRAFT_CLI`, `GETCRAFT_PHOTOCRAFT_CLI`, `GETCRAFT_PDFCRAFT_CLI`, `GETCRAFT_DESIGNCRAFT_CLI`, `GETCRAFT_EFFECTCRAFT_CLI`, `GETCRAFT_LIGHTCRAFT_CLI`, or `GETCRAFT_VECTORCRAFT_CLI`.
- Codex or Claude Code for plugin installation.

The gateway does not download, bundle, or license the upstream applications. Install each one from its own official release or build instructions.

## Install the Rust server

Until crates.io publishing is configured, install directly from GitHub:

```sh
cargo install --git https://github.com/LuizGustavoWT/getcraft-apps-mcp --bin getcraft-mcp
getcraft-mcp apps
```

For local development:

```sh
cargo install --path .
getcraft-mcp apps
```

## Install in Codex

Install the binary first. Then add the repository marketplace and plugin:

```sh
codex plugin marketplace add LuizGustavoWT/getcraft-apps-mcp --sparse .agents/plugins --sparse plugins
codex plugin add getcraft-apps-mcp@getcraft-apps
```

If the marketplace was previously added with only `--sparse .agents/plugins`, replace that incomplete checkout before installing:

```sh
codex plugin marketplace remove getcraft-apps
codex plugin marketplace add LuizGustavoWT/getcraft-apps-mcp --sparse .agents/plugins --sparse plugins
```

Start a new Codex chat after installation. The plugin starts `getcraft-mcp serve` and exposes the seven skills. To use the server without the plugin:

```sh
codex mcp add getcraft-apps -- getcraft-mcp serve
```

## Install in Claude Code

Install the binary first. Then add the marketplace and install its plugin:

```sh
claude plugin marketplace add LuizGustavoWT/getcraft-apps-mcp
claude plugin install getcraft-apps-mcp@getcraft-apps
```

Restart Claude Code or start a new session. Skills are namespaced, for example `/getcraft-apps-mcp:vectorcraft`. To use only the server:

```sh
claude mcp add getcraft-apps -- getcraft-mcp serve
```

Validate the Claude package after cloning or changing it:

```sh
claude plugin validate ./plugins/getcraft-apps-mcp
```

## Example workflow

1. Ask the gateway to list native CLIs: `craft_list_apps`.
2. Ask for the relevant skill, for example VectorCraft.
3. For routine finite work, call `craft_cli_help` or `craft_command_catalog`, then `craft_run_cli`.
4. For detailed editing, call `craft_native_mcp_config` and add the resulting `mcpServers.<app>` entry directly to the client.
5. Use explicit output paths and verify a render or inspection result before replacing an original file.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
claude plugin validate ./plugins/getcraft-apps-mcp
```

Codex validates the compatibility manifest and marketplace entry when you add the marketplace from the local checkout or GitHub source.

## Governance

The repository is public and welcomes issues and pull requests. `main` is protected and `CODEOWNERS` names only [@LuizGustavoWT](https://github.com/LuizGustavoWT); only that account holds repository administration/write authority. Therefore external contributors can propose changes through forks and pull requests, while only the owner can merge them or close issues opened by others.

## License

Licensed under [Apache-2.0](LICENSE). This project is independent and is not affiliated with the upstream Storytold Craft projects.
