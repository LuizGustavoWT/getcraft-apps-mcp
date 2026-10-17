#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::{
    env,
    ffi::OsString,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    time::timeout,
};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const MAX_TIMEOUT_MS: u64 = 120_000;
const MAX_ARGS: usize = 64;
const MAX_ARGUMENT_BYTES: usize = 32 * 1024;
const OUTPUT_LIMIT_BYTES: usize = 1_048_576;
const DISCOVERY_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_SEARCH_LIMIT: usize = 25;
const MAX_SEARCH_LIMIT: usize = 100;

mod cli_args;
mod compatibility;
mod process;

#[derive(Clone, Copy)]
struct AppSpec {
    id: &'static str,
    display_name: &'static str,
    program: &'static str,
    env_var: &'static str,
    repository: &'static str,
    summary: &'static str,
    catalog_args: &'static [&'static str],
}

const APPS: [AppSpec; 7] = [
    AppSpec {
        id: "filmcraft",
        display_name: "FilmCraft",
        program: "filmcraft-cli",
        env_var: "GETCRAFT_FILMCRAFT_CLI",
        repository: "https://github.com/storytold/filmcraft",
        summary: "Non-linear video editing and export.",
        catalog_args: &["commands", "--json"],
    },
    AppSpec {
        id: "photocraft",
        display_name: "PhotoCraft",
        program: "photocraft-cli",
        env_var: "GETCRAFT_PHOTOCRAFT_CLI",
        repository: "https://github.com/storytold/photocraft",
        summary: "Layered raster-image editing.",
        catalog_args: &["commands", "--json"],
    },
    AppSpec {
        id: "pdfcraft",
        display_name: "PdfCraft",
        program: "pdfcraft-cli",
        env_var: "GETCRAFT_PDFCRAFT_CLI",
        repository: "https://github.com/storytold/pdfcraft",
        summary: "PDF inspection, editing, conversion, and audit.",
        catalog_args: &["tools"],
    },
    AppSpec {
        id: "designcraft",
        display_name: "DesignCraft",
        program: "designcraft-cli",
        env_var: "GETCRAFT_DESIGNCRAFT_CLI",
        repository: "https://github.com/storytold/designcraft",
        summary: "Page layout, typography, and publication design.",
        catalog_args: &["commands"],
    },
    AppSpec {
        id: "effectcraft",
        display_name: "EffectCraft",
        program: "effectcraft-cli",
        env_var: "GETCRAFT_EFFECTCRAFT_CLI",
        repository: "https://github.com/storytold/effectcraft",
        summary: "Motion graphics, compositing, and visual effects.",
        catalog_args: &["commands", "--schemas", "--json"],
    },
    AppSpec {
        id: "lightcraft",
        display_name: "LightCraft",
        program: "lightcraft-cli",
        env_var: "GETCRAFT_LIGHTCRAFT_CLI",
        repository: "https://github.com/storytold/lightcraft",
        summary: "Raw-photo development and library workflows.",
        catalog_args: &["commands", "--json"],
    },
    AppSpec {
        id: "vectorcraft",
        display_name: "VectorCraft",
        program: "vectorcraft-cli",
        env_var: "GETCRAFT_VECTORCRAFT_CLI",
        repository: "https://github.com/storytold/vectorcraft",
        summary: "Vector illustration, layouts, and exports.",
        catalog_args: &["commands"],
    },
];

#[derive(Debug, Serialize)]
struct AppStatus {
    id: &'static str,
    display_name: &'static str,
    program: &'static str,
    configured_by: &'static str,
    available: bool,
    resolved_program: Option<String>,
    repository: &'static str,
    summary: &'static str,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AppRequest {
    #[schemars(
        description = "One of: filmcraft, photocraft, pdfcraft, designcraft, effectcraft, lightcraft, vectorcraft."
    )]
    app: String,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SearchRequest {
    #[schemars(description = "A Craft application id, as returned by craft_list_apps.")]
    app: String,
    #[schemars(
        description = "Case-insensitive text to search across command ids, labels, descriptions, and parameter schemas. Omit to browse the catalog."
    )]
    query: Option<String>,
    #[schemars(
        description = "commands (default), or controls for LightCraft's develop-control catalog."
    )]
    kind: Option<String>,
    #[schemars(description = "Maximum entries to return, from 1 to 100. Defaults to 25.")]
    limit: Option<usize>,
    #[schemars(description = "Zero-based offset into the matching entries. Defaults to 0.")]
    offset: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct HelpRequest {
    #[schemars(
        description = "One of: filmcraft, photocraft, pdfcraft, designcraft, effectcraft, lightcraft, vectorcraft."
    )]
    app: String,
    #[schemars(description = "Optional native CLI subcommand whose help should be requested.")]
    command: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RunRequest {
    #[schemars(
        description = "One of: filmcraft, photocraft, pdfcraft, designcraft, effectcraft, lightcraft, vectorcraft."
    )]
    app: String,
    #[schemars(
        description = "Native CLI arguments only. Do not include a shell command or shell operators."
    )]
    args: Vec<String>,
    #[schemars(
        description = "Execution timeout in milliseconds (100 to 120000). Defaults to 30000."
    )]
    timeout_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NativeMcpConfigRequest {
    #[schemars(
        description = "One of: filmcraft, photocraft, pdfcraft, designcraft, effectcraft, lightcraft, vectorcraft."
    )]
    app: String,
    #[schemars(
        description = "Use bridge to connect to a running desktop app; omit for the application's headless MCP mode."
    )]
    mode: Option<String>,
    #[schemars(
        description = "Loopback host:port (or port where documented) for bridge/connect mode."
    )]
    connect: Option<String>,
    #[schemars(
        description = "Absolute directory allowed for reads by apps that implement a read root."
    )]
    read_root: Option<String>,
    #[schemars(
        description = "Absolute directory allowed for writes by apps that implement a write root."
    )]
    write_root: Option<String>,
    #[schemars(
        description = "Absolute PdfCraft root or LightCraft library directory, depending on the selected app."
    )]
    workspace: Option<String>,
    #[schemars(
        description = "Additional absolute read roots. Supported only by EffectCraft, whose root flags are repeatable."
    )]
    read_roots: Option<Vec<String>>,
    #[schemars(
        description = "Additional absolute write roots. Supported only by EffectCraft, whose root flags are repeatable."
    )]
    write_roots: Option<Vec<String>>,
    #[schemars(
        description = "Enable the compact MCP tool catalog in PdfCraft or LightCraft. All native tools remain reachable through their discovery helpers."
    )]
    compact: Option<bool>,
    #[schemars(
        description = "Absolute path to PhotoCraft's control token file in bridge mode. The token is never read by this gateway."
    )]
    control_token_file: Option<String>,
    #[schemars(
        description = "Enable EffectCraft headless autosave. This intentionally persists checkpoints; not valid in bridge mode."
    )]
    autosave: Option<bool>,
}

#[derive(Debug, Serialize)]
struct InvocationResult {
    app: String,
    program: String,
    args: Vec<String>,
    exit_code: Option<i32>,
    success: bool,
    stdout: String,
    stdout_truncated: bool,
    stderr: String,
    stderr_truncated: bool,
}

#[derive(Debug)]
struct CapturedOutput {
    text: String,
    truncated: bool,
}

#[derive(Clone)]
struct GetCraftServer {
    tool_router: ToolRouter<Self>,
}

impl GetCraftServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl GetCraftServer {
    #[tool(
        description = "List all supported Craft apps and report whether each native CLI can be found. Environment-variable overrides are shown too."
    )]
    async fn craft_list_apps(&self) -> Result<String, String> {
        to_pretty_json(&APPS.iter().map(app_status).collect::<Vec<_>>())
    }

    #[tool(
        description = "Inspect the installed CLI version and the upstream release/commit audited by this plugin. This is a read-only local check, not an online update or an installation."
    )]
    async fn craft_app_info(
        &self,
        Parameters(request): Parameters<AppRequest>,
    ) -> Result<String, String> {
        to_pretty_json(&app_info(lookup_app(&request.app)?).await)
    }

    #[tool(
        description = "Return the exact native CLI help for one Craft app. Use this before assembling a command when its syntax may have changed."
    )]
    async fn craft_cli_help(
        &self,
        Parameters(request): Parameters<HelpRequest>,
    ) -> Result<String, String> {
        let spec = lookup_app(&request.app)?;
        let mut args = Vec::new();
        if let Some(command) = &request.command {
            validate_token(command, "command")?;
            if command.starts_with('-') || command.chars().any(char::is_whitespace) {
                return Err(
                    "command must be one subcommand name, not flags or a command line".to_string(),
                );
            }
            args.push(command.clone());
        }
        args.push("--help".to_string());
        let result = invoke(spec, args, DEFAULT_TIMEOUT_MS).await?;
        // Several Craft CLIs only implement top-level help and reject `mcp --help`.
        if !result.success && request.command.is_some() {
            let fallback = invoke(spec, vec!["--help".to_string()], DEFAULT_TIMEOUT_MS).await?;
            let mut value = serde_json::to_value(fallback).map_err(|error| error.to_string())?;
            value["requested_command"] = json!(request.command);
            value["help_scope"] = json!("top-level fallback");
            value["subcommand_help_error"] = json!(result.stderr);
            to_pretty_json(&value)
        } else {
            to_pretty_json(&result)
        }
    }

    #[tool(
        description = "Read the command or automation catalog of one Craft CLI. The exact native catalog is returned without hard-coding its command IDs."
    )]
    async fn craft_command_catalog(
        &self,
        Parameters(request): Parameters<AppRequest>,
    ) -> Result<String, String> {
        let spec = lookup_app(&request.app)?;
        let args = spec
            .catalog_args
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        let result = invoke(spec, args, DEFAULT_TIMEOUT_MS).await?;
        to_pretty_json(&result)
    }

    #[tool(
        description = "Search the installed CLI's current JSON command catalog, with pagination and native parameter metadata. New upstream commands appear automatically. LightCraft also supports kind=controls."
    )]
    async fn craft_search_commands(
        &self,
        Parameters(request): Parameters<SearchRequest>,
    ) -> Result<String, String> {
        let spec = lookup_app(&request.app)?;
        let args = search_catalog_args(spec, &request)?;
        let result = invoke(spec, args, DEFAULT_TIMEOUT_MS).await?;
        if !result.success {
            return Err(format!(
                "{} catalog failed: {}",
                spec.program, result.stderr
            ));
        }
        if result.stdout_truncated {
            return Err("native catalog exceeded the capture limit; use the app's native MCP discovery tools".to_string());
        }
        to_pretty_json(&search_catalog(&request, &result.stdout)?)
    }

    #[tool(
        description = "Run a finite native Craft CLI command without a shell. Known persistent server subcommands are rejected; use craft_native_mcp_config for the app's richer first-party MCP server."
    )]
    async fn craft_run_cli(
        &self,
        Parameters(request): Parameters<RunRequest>,
    ) -> Result<String, String> {
        let spec = lookup_app(&request.app)?;
        validate_invocation(spec.id, &request.args)?;
        let timeout_ms = checked_timeout(request.timeout_ms)?;
        let result = invoke(spec, request.args, timeout_ms).await?;
        to_pretty_json(&result)
    }

    #[tool(
        description = "Generate a copyable MCP configuration for an app's own first-party MCP server. Prefer a confined filesystem mode whenever that app supports it."
    )]
    async fn craft_native_mcp_config(
        &self,
        Parameters(request): Parameters<NativeMcpConfigRequest>,
    ) -> Result<String, String> {
        let config = native_mcp_config(&request)?;
        verify_root_support(&request).await?;
        to_pretty_json(&config)
    }
}

#[tool_handler(
    router = self.tool_router,
    name = "getcraft-apps-mcp",
    version = "0.2.0",
    instructions = "Use craft_list_apps first and craft_app_info to check installed versions. Use craft_search_commands to discover current commands and parameter schemas. Prefer craft_native_mcp_config for persistent editing and long renders; finite CLI calls do not share an in-memory session. Configuration roots are native-MCP-only, not a sandbox for craft_run_cli."
)]
impl ServerHandler for GetCraftServer {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    match args.first().and_then(|value| value.to_str()) {
        Some("serve") => {
            let server = GetCraftServer::new()
                .serve(rmcp::transport::stdio())
                .await?;
            server.waiting().await?;
        }
        Some("apps") => {
            println!(
                "{}",
                to_pretty_json(&APPS.iter().map(app_status).collect::<Vec<_>>())?
            );
        }
        Some("doctor") => {
            let mut reports = Vec::new();
            for spec in APPS {
                reports.push(app_info(spec).await);
            }
            println!("{}", to_pretty_json(&reports)?);
        }
        Some("--version" | "-V") => println!("getcraft-mcp {}", env!("CARGO_PKG_VERSION")),
        Some("--help" | "-h" | "help") | None => print_usage(),
        Some(other) => {
            return Err(format!("unknown subcommand `{other}`; see `getcraft-mcp --help`").into());
        }
    }
    Ok(())
}

fn print_usage() {
    println!(
        "getcraft-mcp — MCP gateway for Storytold Craft CLIs\n\nUSAGE:\n  getcraft-mcp serve       Run the stdio MCP server\n  getcraft-mcp apps        Print native CLI discovery as JSON\n  getcraft-mcp doctor      Compare installed CLI versions with the audited upstream snapshot\n  getcraft-mcp --version   Print the gateway version\n\nSet GETCRAFT_<APP>_CLI to an absolute executable path to override PATH discovery."
    );
}

fn lookup_app(id: &str) -> Result<AppSpec, String> {
    APPS.iter()
        .copied()
        .find(|spec| spec.id == id)
        .ok_or_else(|| format!("unknown app `{id}`; call craft_list_apps for the supported ids"))
}

fn app_status(spec: &AppSpec) -> AppStatus {
    let resolved = locate_program(spec);
    AppStatus {
        id: spec.id,
        display_name: spec.display_name,
        program: spec.program,
        configured_by: spec.env_var,
        available: resolved.is_some(),
        resolved_program: resolved.map(|path| path.display().to_string()),
        repository: spec.repository,
        summary: spec.summary,
    }
}

fn locate_program(spec: &AppSpec) -> Option<PathBuf> {
    if let Some(value) = env::var_os(spec.env_var).filter(|value| !value.is_empty()) {
        let path = PathBuf::from(value);
        return (path.is_absolute() && is_executable(&path)).then_some(path);
    }

    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| {
            let name = if cfg!(windows) {
                format!("{}.exe", spec.program)
            } else {
                spec.program.to_string()
            };
            directory.join(name)
        })
        .find(|candidate| is_executable(candidate))
        .and_then(|candidate| std::path::absolute(candidate).ok())
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

async fn app_info(spec: AppSpec) -> serde_json::Value {
    let status = app_status(&spec);
    let result = invoke(spec, vec!["--version".to_string()], DISCOVERY_TIMEOUT_MS).await;
    let (version, error) = match result {
        Ok(result) if result.success => (Some(result.stdout.trim().to_string()), None),
        Ok(result) => (
            None,
            Some(format!("version probe failed: {}", result.stderr)),
        ),
        Err(error) => (None, Some(error)),
    };
    let audit = compatibility::for_app(spec.id);
    let installed_version = version
        .as_deref()
        .and_then(|text| text.lines().next())
        .and_then(|line| line.split_whitespace().nth(1));
    let audited_version = audit["latest_release"]["tag"]
        .as_str()
        .map(|tag| tag.trim_start_matches('v'));
    let matches_audited_release = installed_version
        .zip(audited_version)
        .map(|(installed, audited)| installed == audited);
    json!({
        "app": status,
        "installed_version": installed_version,
        "version_output": version,
        "version_error": error,
        "matches_audited_release": matches_audited_release,
        "audit": audit,
        "note": "The audit is a bundled snapshot, not a live update check. A matching version does not imply that post-release main-branch features are installed."
    })
}

fn search_catalog_args(spec: AppSpec, request: &SearchRequest) -> Result<Vec<String>, String> {
    let limit = request.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
    if !(1..=MAX_SEARCH_LIMIT).contains(&limit) {
        return Err(format!("limit must be between 1 and {MAX_SEARCH_LIMIT}"));
    }
    if let Some(query) = &request.query
        && !query.is_empty()
    {
        validate_token(query, "query")?;
    }
    let args = match request.kind.as_deref().unwrap_or("commands") {
        "commands" => spec.catalog_args,
        "controls" if spec.id == "lightcraft" => &["controls", "--json"],
        "controls" => return Err("kind=controls is only supported by LightCraft".to_string()),
        _ => return Err("kind must be commands or controls".to_string()),
    };
    Ok(args.iter().map(|value| (*value).to_string()).collect())
}

fn search_catalog(request: &SearchRequest, stdout: &str) -> Result<serde_json::Value, String> {
    let catalog: serde_json::Value = serde_json::from_str(stdout)
        .map_err(|error| format!("native catalog is not valid JSON: {error}"))?;
    let entries = catalog
        .as_array()
        .ok_or_else(|| "native catalog must be a JSON array".to_string())?;
    let query = request.query.as_deref().unwrap_or("").trim().to_lowercase();
    let matching: Vec<_> = entries
        .iter()
        .filter(|entry| query.is_empty() || entry.to_string().to_lowercase().contains(&query))
        .collect();
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
    let page: Vec<_> = matching.iter().skip(offset).take(limit).copied().collect();
    let next_offset = offset
        .checked_add(page.len())
        .filter(|next| !page.is_empty() && *next < matching.len());
    Ok(json!({
        "app": request.app,
        "kind": request.kind.as_deref().unwrap_or("commands"),
        "total": entries.len(),
        "matched": matching.len(),
        "offset": offset,
        "next_offset": next_offset,
        "entries": page
    }))
}

fn program_for(spec: &AppSpec) -> Result<PathBuf, String> {
    locate_program(spec).ok_or_else(|| {
        format!(
            "{} was not found. Install `{}` or set {} to its absolute executable path",
            spec.display_name, spec.program, spec.env_var
        )
    })
}

fn checked_timeout(value: Option<u64>) -> Result<u64, String> {
    let value = value.unwrap_or(DEFAULT_TIMEOUT_MS);
    if !(100..=MAX_TIMEOUT_MS).contains(&value) {
        return Err(format!(
            "timeout_ms must be between 100 and {MAX_TIMEOUT_MS}"
        ));
    }
    Ok(value)
}

fn validate_token(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() || value.contains('\0') {
        return Err(format!(
            "{field} must be non-empty and cannot contain a NUL byte"
        ));
    }
    if value.len() > 8_192 {
        return Err(format!("{field} is too long"));
    }
    Ok(())
}

fn validate_invocation(app: &str, args: &[String]) -> Result<(), String> {
    if args.len() > MAX_ARGS {
        return Err(format!("at most {MAX_ARGS} CLI arguments are allowed"));
    }
    let total_bytes = args.iter().try_fold(0usize, |total, argument| {
        validate_token(argument, "argument")?;
        total
            .checked_add(argument.len())
            .ok_or_else(|| "argument size overflow".to_string())
    })?;
    if total_bytes > MAX_ARGUMENT_BYTES {
        return Err(format!(
            "arguments exceed the {MAX_ARGUMENT_BYTES}-byte limit"
        ));
    }
    if matches!(
        cli_args::subcommand(app, args),
        Some("mcp" | "serve" | "denoise-serve")
    ) {
        return Err("persistent subcommands (mcp, serve, denoise-serve) are not allowed through craft_run_cli, including after global flags; use craft_native_mcp_config for MCP sessions".to_string());
    }
    Ok(())
}

async fn invoke(
    spec: AppSpec,
    args: Vec<String>,
    timeout_ms: u64,
) -> Result<InvocationResult, String> {
    let program = program_for(&spec)?;
    invoke_program(spec, program, args, timeout_ms).await
}

async fn invoke_program(
    spec: AppSpec,
    program: PathBuf,
    args: Vec<String>,
    timeout_ms: u64,
) -> Result<InvocationResult, String> {
    let mut command = Command::new(&program);
    command
        .args(&args)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    process::prepare(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("could not start {}: {error}", spec.program))?;
    let process_group = process::ProcessGroup::new(child.id());
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "could not capture child stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "could not capture child stderr".to_string())?;
    // Include output draining in the deadline. A descendant retaining an output
    // pipe must not keep the MCP request blocked after the CLI itself exits.
    let execution = async {
        tokio::try_join!(
            async {
                child
                    .wait()
                    .await
                    .map_err(|error| format!("failed while waiting for {}: {error}", spec.program))
            },
            capture_output(stdout),
            capture_output(stderr)
        )
    };
    let (status, stdout, stderr) = match timeout(Duration::from_millis(timeout_ms), execution).await
    {
        Ok(result) => result?,
        Err(_) => {
            drop(process_group);
            let _ = child.kill().await;
            return Err(format!(
                "{} exceeded the {timeout_ms}ms timeout and was terminated; changes or output files may be partial, so inspect them before retrying. Use native MCP for long-running renders",
                spec.program
            ));
        }
    };
    Ok(InvocationResult {
        app: spec.id.to_string(),
        program: program.display().to_string(),
        args,
        exit_code: status.code(),
        success: status.success(),
        stdout: stdout.text,
        stdout_truncated: stdout.truncated,
        stderr: stderr.text,
        stderr_truncated: stderr.truncated,
    })
}

async fn capture_output<R>(mut reader: R) -> Result<CapturedOutput, String>
where
    R: AsyncRead + Unpin,
{
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8_192];
    let mut truncated = false;
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|error| format!("could not read process output: {error}"))?;
        if count == 0 {
            break;
        }
        let room = OUTPUT_LIMIT_BYTES.saturating_sub(bytes.len());
        let keep = room.min(count);
        bytes.extend_from_slice(&buffer[..keep]);
        truncated |= keep < count;
    }
    Ok(CapturedOutput {
        text: String::from_utf8_lossy(&bytes).into_owned(),
        truncated,
    })
}

fn native_mcp_config(request: &NativeMcpConfigRequest) -> Result<serde_json::Value, String> {
    let spec = lookup_app(&request.app)?;
    let read_roots = collect_roots(&request.read_root, &request.read_roots, "read_root")?;
    let write_roots = collect_roots(&request.write_root, &request.write_roots, "write_root")?;
    let has_roots = !read_roots.is_empty() || !write_roots.is_empty();
    validate_optional_path(&request.workspace, "workspace")?;
    validate_optional_path(&request.control_token_file, "control_token_file")?;
    let mode = request.mode.as_deref().unwrap_or("headless");
    if mode != "headless" && mode != "bridge" {
        return Err("mode must be `headless` or `bridge`".to_string());
    }
    if mode == "headless" && request.connect.is_some() {
        return Err(
            "connect requires mode=bridge; it cannot be ignored in headless mode".to_string(),
        );
    }
    if request.workspace.is_some()
        && (mode != "headless" || !matches!(spec.id, "pdfcraft" | "lightcraft"))
    {
        return Err(
            "workspace is supported only for headless PdfCraft (--root) or LightCraft (--library)"
                .to_string(),
        );
    }
    if request.compact == Some(true) && !matches!(spec.id, "pdfcraft" | "lightcraft") {
        return Err(
            "compact MCP catalogs are supported only by PdfCraft and LightCraft".to_string(),
        );
    }
    if request.autosave == Some(true) && (spec.id != "effectcraft" || mode != "headless") {
        return Err("autosave is supported only by EffectCraft in headless mode".to_string());
    }
    if request.control_token_file.is_some() && (spec.id != "photocraft" || mode != "bridge") {
        return Err(
            "control_token_file is supported only by PhotoCraft in bridge mode".to_string(),
        );
    }
    let additional_roots = request
        .read_roots
        .as_ref()
        .is_some_and(|roots| !roots.is_empty())
        || request
            .write_roots
            .as_ref()
            .is_some_and(|roots| !roots.is_empty());
    if additional_roots && spec.id != "effectcraft" {
        return Err("read_roots/write_roots are supported only by EffectCraft; use the singular read_root/write_root fields for other apps".to_string());
    }
    if has_roots && matches!(spec.id, "filmcraft" | "lightcraft") {
        return Err(format!(
            "{} does not support MCP filesystem-root flags; the requested confinement cannot be provided",
            spec.display_name
        ));
    }
    if has_roots
        && mode == "bridge"
        && matches!(spec.id, "photocraft" | "designcraft" | "vectorcraft")
    {
        return Err(format!(
            "{} bridge mode cannot apply MCP-side filesystem roots. Use headless mode, or configure the desktop's own policy where supported",
            spec.display_name
        ));
    }
    let mut args = vec!["mcp".to_string()];
    let notes = match (spec.id, mode) {
        ("filmcraft", "bridge") => {
            args.extend(["--bridge".to_string(), require_connect(request)?]);
            "Bridge mode requires FilmCraft to be running with its matching --control port. FilmCraft's MCP has no filesystem-root confinement flags."
        }
        ("filmcraft", _) => {
            "Headless FilmCraft may read and write anywhere the user can. Use only with trusted project paths."
        }
        ("photocraft", "bridge") => {
            args.extend(["--bridge".to_string(), require_connect(request)?]);
            if let Some(path) = &request.control_token_file {
                args.extend(["--control-token-file".to_string(), path.clone()]);
            }
            "PhotoCraft's desktop owns the filesystem policy in bridge mode. Start it with explicit automation roots and authentication. Supply control_token_file or configure PHOTOCRAFT_CONTROL_TOKEN_FILE in the client environment; the gateway never reads the token."
        }
        ("photocraft", _) => {
            append_roots(&mut args, &read_roots, &write_roots);
            "PhotoCraft headless grants no filesystem access when both roots are omitted. Each omitted root denies its kind of access; in-memory editing remains available."
        }
        ("pdfcraft", "bridge") => {
            return Err("PdfCraft documents only a headless stdio MCP server".to_string());
        }
        ("pdfcraft", _) => {
            if let Some(root) = &request.workspace {
                args.extend(["--root".to_string(), root.clone()]);
            }
            append_roots(&mut args, &read_roots, &write_roots);
            "PdfCraft --root bounds all paths. Separate read/write roots require a build newer than v0.8.0 and intersect --root when both policies are set. With either automation root set, the omitted kind of access is denied; with no roots at all the server is not confined."
        }
        ("designcraft", "bridge") => {
            args.extend(["--connect".to_string(), require_connect(request)?]);
            "Start DesignCraft with its control port before connecting. Its desktop currently has no filesystem-root policy; bridge mode is not confined."
        }
        ("designcraft", _) => {
            append_roots(&mut args, &read_roots, &write_roots);
            "Headless DesignCraft starts an in-process engine. Automation roots require a build newer than v0.8.0; once either root is set, an omitted root denies that access. With neither root set, filesystem access is not confined."
        }
        ("effectcraft", "bridge") => {
            args.extend(["--bridge".to_string(), require_connect(request)?]);
            append_roots(&mut args, &read_roots, &write_roots);
            "EffectCraft bridge roots confine command file access and refuse scripts/UI-driving tools. Reads may use read or write roots; writes require a write root. Without roots it is not confined. Start the desktop on the matching control port."
        }
        ("effectcraft", _) => {
            append_roots(&mut args, &read_roots, &write_roots);
            if request.autosave == Some(true) {
                args.push("--autosave".to_string());
            }
            "EffectCraft roots are repeatable: reads may use read or write roots, writes require a write root, and scripts are refused while confined. No roots means unconfined. Autosave intentionally persists headless checkpoints."
        }
        ("lightcraft", "bridge") => {
            args.extend(["--connect".to_string(), require_connect(request)?]);
            "Connect mode drives the running LightCraft desktop app and avoids a library-lock conflict. It is not filesystem-confined. Inspect state before retrying a command whose connection failed with an unknown outcome."
        }
        ("lightcraft", _) => {
            if let Some(library) = &request.workspace {
                args.extend(["--library".to_string(), library.clone()]);
            }
            "Set workspace to a LightCraft library for persistence, not filesystem confinement. A headless server cannot open a library already locked by the desktop app."
        }
        ("vectorcraft", "bridge") => {
            args.extend(["--connect".to_string(), require_connect(request)?]);
            "For VectorCraft bridge mode, the running desktop must enforce its own automation roots. No MCP-side roots are applied."
        }
        ("vectorcraft", _) => {
            // Bare `mcp` auto-connects to a desktop; headless must be explicit.
            args.push("--headless".to_string());
            append_roots(&mut args, &read_roots, &write_roots);
            "VectorCraft is explicitly headless, with no desktop auto-connect. With either automation root set, an omitted root denies that access. With neither root set, filesystem access is not confined."
        }
        _ => return Err("unsupported app/mode combination".to_string()),
    };
    if request.compact == Some(true) {
        args.push("--compact".to_string());
    }
    let program = locate_program(&spec).unwrap_or_else(|| PathBuf::from(spec.program));
    let audit = compatibility::for_app(spec.id);
    Ok(json!({
        "mcpServers": {
            spec.id: {
                "type": "stdio",
                "command": program,
                "args": args
            }
        },
        "notes": notes,
        "first_party_docs": audit["docs"],
        "scope": "These options configure the native MCP server only; they do not sandbox craft_run_cli or untrusted native scripts. Install a compatible CLI and create the root directories before starting the server."
    }))
}

async fn verify_root_support(request: &NativeMcpConfigRequest) -> Result<(), String> {
    let has_read = request.read_root.is_some()
        || request
            .read_roots
            .as_ref()
            .is_some_and(|roots| !roots.is_empty());
    let has_write = request.write_root.is_some()
        || request
            .write_roots
            .as_ref()
            .is_some_and(|roots| !roots.is_empty());
    if !has_read && !has_write {
        return Ok(());
    }
    let spec = lookup_app(&request.app)?;
    // Do not return a supposedly confined config for a missing or older binary.
    // In particular, PdfCraft/DesignCraft v0.8.0 predate these flags.
    let help = invoke(spec, vec!["--help".to_string()], DISCOVERY_TIMEOUT_MS).await?;
    let output = format!("{}\n{}", help.stdout, help.stderr);
    verify_root_help(spec, &output, help.success, has_read, has_write)
}

fn verify_root_help(
    spec: AppSpec,
    help: &str,
    success: bool,
    has_read: bool,
    has_write: bool,
) -> Result<(), String> {
    if !success
        || (has_read && !help.contains("--automation-read-root"))
        || (has_write && !help.contains("--automation-write-root"))
    {
        return Err(format!(
            "the installed {} does not advertise the requested automation-root flags. Upgrade to a build that supports them (see craft_app_info); refusing to generate a configuration that would silently lose confinement",
            spec.program
        ));
    }
    Ok(())
}

fn require_connect(request: &NativeMcpConfigRequest) -> Result<String, String> {
    let value = request
        .connect
        .as_deref()
        .ok_or_else(|| "connect is required when mode is `bridge`".to_string())?;
    validate_token(value, "connect")?;
    let address = if let Ok(port) = value.parse::<u16>() {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    } else if let Some(port) = value.strip_prefix("localhost:") {
        let port = port
            .parse::<u16>()
            .map_err(|_| "connect has an invalid port".to_string())?;
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    } else {
        value.parse::<SocketAddr>().map_err(|_| {
            "connect must be a port or a loopback host:port (for example 127.0.0.1:9876)"
                .to_string()
        })?
    };
    if !address.ip().is_loopback() || address.port() == 0 {
        return Err("connect must use a loopback address and a port from 1 to 65535".to_string());
    }
    Ok(address.to_string())
}

fn validate_optional_path(value: &Option<String>, field: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_token(value, field)?;
        if !Path::new(value).is_absolute() {
            return Err(format!("{field} must be an absolute path"));
        }
    }
    Ok(())
}

fn collect_roots(
    single: &Option<String>,
    additional: &Option<Vec<String>>,
    field: &str,
) -> Result<Vec<String>, String> {
    validate_optional_path(single, field)?;
    let mut roots: Vec<String> = single.iter().cloned().collect();
    if let Some(additional) = additional {
        if additional.len() > 16 {
            return Err(format!("{field}: at most 16 additional roots are allowed"));
        }
        for root in additional {
            validate_optional_path(&Some(root.clone()), field)?;
            if !roots.contains(root) {
                roots.push(root.clone());
            }
        }
    }
    Ok(roots)
}

fn append_roots(args: &mut Vec<String>, read_roots: &[String], write_roots: &[String]) {
    for root in read_roots {
        args.extend(["--automation-read-root".to_string(), root.clone()]);
    }
    for root in write_roots {
        args.extend(["--automation-write-root".to_string(), root.clone()]);
    }
}

fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map_err(|error| format!("could not serialize result: {error}"))
}

#[cfg(test)]
mod tests;
