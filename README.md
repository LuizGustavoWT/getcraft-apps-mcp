# GetCraft Apps MCP

[Português (Brasil)](README.pt-BR.md) · [Contributing](CONTRIBUTING.md) · [Contribuindo](CONTRIBUTING.pt-BR.md)

One Rust MCP gateway and one installable plugin bundle for the Storytold Craft applications:
[FilmCraft](https://github.com/storytold/filmcraft), [PhotoCraft](https://github.com/storytold/photocraft), [PdfCraft](https://github.com/storytold/pdfcraft), [DesignCraft](https://github.com/storytold/designcraft), [EffectCraft](https://github.com/storytold/effectcraft), [LightCraft](https://github.com/storytold/lightcraft), and [VectorCraft](https://github.com/storytold/vectorcraft).

The gateway discovers installed native CLIs, checks versions, searches their live command catalogs, runs finite commands without a shell, and generates configuration for each app's own MCP server. It does not reimplement the native editors or their MCP tools. Seven bundled skills give Codex and Claude Code app-specific workflows and safety notes.

## What is included

| Component | Purpose |
| --- | --- |
| `getcraft-mcp serve` | Rust stdio MCP server with the seven tools below. |
| `getcraft-mcp doctor` | Installed CLI versions and the upstream audit snapshot; `apps` provides basic discovery and `--version` reports the gateway version. |
| `plugins/getcraft-apps-mcp/` | One plugin directory compatible with Codex and Claude Code. |
| Seven skills | Targeted FilmCraft, PhotoCraft, PdfCraft, DesignCraft, EffectCraft, LightCraft, and VectorCraft instructions. |
| Two marketplaces | `.agents/plugins/marketplace.json` for Codex and `.claude-plugin/marketplace.json` for Claude Code. |
| [Upstream snapshot](upstream-versions.json) | Release tags, exact audited commits, tested CLI versions, and unreleased-feature notes. |

## MCP tools

| Tool | Purpose |
| --- | --- |
| `craft_list_apps` | Discover native CLI executables and environment overrides. |
| `craft_app_info` | Inspect one app's installed version and recorded upstream compatibility information. |
| `craft_cli_help` | Read the installed CLI's help before composing a command. |
| `craft_command_catalog` | Return the full native catalog, including its original JSON in the result's `stdout`. |
| `craft_search_commands` | Search and page the installed catalog without dumping it all into context. |
| `craft_run_cli` | Run a finite native CLI command directly. |
| `craft_native_mcp_config` | Generate, but do not install or launch, a native MCP configuration. |

`craft_search_commands` accepts `app`, optional `query`, `kind`, `limit`, and `offset`; it returns `{app, kind, total, matched, offset, next_offset, entries}`. The default kind is `commands`; `controls` is available only for LightCraft. Native entries retain their metadata. For example, search PhotoCraft with `{"app":"photocraft","query":"actions","limit":10}` or LightCraft controls with `{"app":"lightcraft","kind":"controls","query":"exposure"}`. Follow `next_offset` for more matches.

## Security model

`craft_run_cli` uses no shell, closes child stdin, bounds arguments/output, and permits at most 120 seconds per call. It rejects persistent servers (`mcp`, `serve`, `denoise-serve`), including when options precede them. A failed or timed-out command may already have changed a document or written partial outputs; inspect before retrying. Use the native MCP for long jobs and persistent editing sessions.

On Unix, each invocation has its own process group, which is terminated when the invocation ends, times out, or is cancelled. This also cleans up ordinary CLI workers. Processes that deliberately detach from the group are outside that guarantee; non-Unix platforms currently terminate only the direct child.

This gateway is **not an OS sandbox**: finite CLI commands run with your account's permissions. Roots in a generated native MCP configuration do not constrain `craft_run_cli`. Grant only the paths needed by the selected workflow:

| App | Native MCP confinement / operational note |
| --- | --- |
| FilmCraft | No native filesystem-root flags; use trusted explicit paths. |
| PhotoCraft | Headless read/write roots are independent; absent roots grant no filesystem authority. A bridge uses the desktop's roots and authentication, not gateway root flags. |
| PdfCraft | `workspace` becomes `--root` for both reads and writes. Separate automation roots exist after v0.8.0; both restrictions apply when combined. Without roots, access is unconfined. |
| DesignCraft | Separate automation roots exist after v0.8.0, headless only. Desktop bridge mode is not confined. Without roots, access is unconfined. |
| EffectCraft | Repeatable roots: reads may use read or write roots; writes require a write root. With no roots it is unconfined. `autosave` intentionally persists headless checkpoints. |
| LightCraft | `workspace` selects a persistent library, **not a sandbox**. One process holds its lock; connect to the desktop to use a library already open there. |
| VectorCraft | Headless configs explicitly use `--headless`. With one root, omitted access is denied; with neither, access is unconfined. A bridge requires roots configured on the desktop itself. |

`craft_native_mcp_config` accepts `mode: "headless"` (default) or `"bridge"`, with a loopback-only `connect` address for bridges. Unsupported options or mode combinations fail rather than being silently ignored. PdfCraft and DesignCraft automation roots are checked against installed CLI help; an older binary is refused, never downgraded to unrestricted access.

Optional fields: `compact` for PdfCraft/LightCraft; `control_token_file` for a PhotoCraft bridge; `autosave` for headless EffectCraft; additional `read_roots`/`write_roots` arrays for EffectCraft's repeated flags. The PhotoCraft token field is a file path, not a secret value; the gateway never reads its contents. PhotoCraft/VectorCraft bridge configs reject root fields—configure those on the desktop. For PdfCraft/DesignCraft, specifying just one automation root denies the other access; omitting both disables that extra policy.

## Prerequisites

- Rust 1.88 or newer.
- At least one supported native Craft CLI on `PATH`, or its absolute executable path in `GETCRAFT_<APP>_CLI`, for example `GETCRAFT_PHOTOCRAFT_CLI` or `GETCRAFT_VECTORCRAFT_CLI`.
- Codex or Claude Code for plugin installation.

The gateway does not download, bundle, or license the upstream applications. Install each one from its own official release or build instructions.

## Install or update the Rust server

Until crates.io publishing is configured, install directly from GitHub:

```sh
cargo install --git https://github.com/LuizGustavoWT/getcraft-apps-mcp --locked --force --bin getcraft-mcp
getcraft-mcp --version
getcraft-mcp doctor
```

For local development:

```sh
cargo install --path . --locked --force --bin getcraft-mcp
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

Both sparse paths are required: `.agents/plugins` contains the catalog, while `plugins` contains the plugin itself. For an already configured Git marketplace, update the binary above, then refresh the marketplace and reinstall its plugin:

```sh
codex plugin marketplace upgrade getcraft-apps
codex plugin add getcraft-apps-mcp@getcraft-apps
```

Keep the two sparse paths when changing the marketplace source. This follows the [official Codex marketplace workflow](https://developers.openai.com/plugins/build/plugins). Start a new Codex chat after installation or update to pick up new tools and skills. The plugin starts `getcraft-mcp serve`. To use the server without the plugin:

```sh
codex mcp add getcraft-apps -- getcraft-mcp serve
```

## Install in Claude Code

Install the binary first. Then add the marketplace and install its plugin:

```sh
claude plugin marketplace add LuizGustavoWT/getcraft-apps-mcp
claude plugin install getcraft-apps-mcp@getcraft-apps
```

Restart Claude Code or start a new session. Skills are namespaced, for example `/getcraft-apps-mcp:vectorcraft`. For package updates, refresh the Rust binary above and use Claude Code's plugin manager to refresh the marketplace/plugin. To use only the server:

```sh
claude mcp add getcraft-apps -- getcraft-mcp serve
```

Validate the Claude package after cloning or changing it:

```sh
claude plugin validate ./plugins/getcraft-apps-mcp
```

## Example workflow

1. Discover installed apps with `craft_list_apps`, then check the selected app with `craft_app_info`.
2. Use its skill and `craft_search_commands` to find relevant operations in the installed version.
3. Read current help/parameter descriptions before `craft_run_cli`; each finite invocation is a new native session.
4. For detailed editing, call `craft_native_mcp_config` and add the resulting `mcpServers.<app>` entry directly to the client.
5. Use explicit output paths and verify a render or inspection result before replacing an original file.

## Audited upstream versions

Snapshot checked on **2026-10-17**; “tested CLI” means local read-only compatibility checks, not exhaustive validation of every editor feature. Exact commits and documentation links are in [upstream-versions.json](upstream-versions.json).

| App | Latest release at audit | Tested CLI |
| --- | --- | --- |
| FilmCraft | 0.8.0 | 0.8.0 |
| PhotoCraft | 0.9.0 | 0.9.0 |
| PdfCraft | 0.8.0 | 0.8.0 |
| DesignCraft | 0.8.0 | 0.8.0 |
| EffectCraft | 0.10.0 | 0.10.0 |
| LightCraft | 0.8.0 | 0.8.0 |
| VectorCraft | 0.12.0 | 0.11.0 |

Updating this plugin does **not** update those applications. VectorCraft 0.12.0 features require upgrading the native app separately. EffectCraft's audited `main` declares 0.11.0, but its latest published release at audit is 0.10.0. Several PdfCraft/DesignCraft confinement options and LightCraft workflows are also newer than their releases. Skills explicitly gate these on installed help/catalogs; the snapshot is not a live release checker. See [CHANGELOG.md](CHANGELOG.md) for gateway changes.

## Development

With Python 3.11+ (standard library only):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_package.py
cargo build --locked
python3 scripts/smoke_mcp.py target/debug/getcraft-mcp
```

The default smoke test needs no Craft apps. With all seven CLIs installed, add `--native` for read-only native checks. Run `claude plugin validate ./plugins/getcraft-apps-mcp` when Claude Code is available; validate actual client installation separately from package tests. Details: [CONTRIBUTING.md](CONTRIBUTING.md).

## Governance

The repository is public and welcomes issues and pull requests. `main` is protected and `CODEOWNERS` names only [@LuizGustavoWT](https://github.com/LuizGustavoWT); only that account holds repository administration/write authority. Therefore external contributors can propose changes through forks and pull requests, while only the owner can merge them or close issues opened by others.

## License

Licensed under [Apache-2.0](LICENSE). This project is independent and is not affiliated with the upstream Storytold Craft projects.
