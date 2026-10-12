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
        catalog_args: &["commands"],
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
        catalog_args: &["commands", "--json"],
    },
    AppSpec {
        id: "lightcraft",
        display_name: "LightCraft",
        program: "lightcraft-cli",
        env_var: "GETCRAFT_LIGHTCRAFT_CLI",
        repository: "https://github.com/storytold/lightcraft",
        summary: "Raw-photo development and library workflows.",
        catalog_args: &["commands"],
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

#[derive(Debug, Deserialize, JsonSchema)]
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
        description = "Return the exact native CLI help for one Craft app. Use this before assembling a command when its syntax may have changed."
    )]
    async fn craft_cli_help(
        &self,
        Parameters(request): Parameters<HelpRequest>,
    ) -> Result<String, String> {
        let spec = lookup_app(&request.app)?;
        let mut args = Vec::new();
        if let Some(command) = request.command {
            validate_token(&command, "command")?;
            args.push(command);
        }
        args.push("--help".to_string());
        let result = invoke(spec, args, DEFAULT_TIMEOUT_MS).await?;
        to_pretty_json(&result)
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
        description = "Run a finite native Craft CLI command without a shell. This never starts a nested MCP or persistent server; use craft_native_mcp_config for the app's richer first-party MCP server."
    )]
    async fn craft_run_cli(
        &self,
        Parameters(request): Parameters<RunRequest>,
    ) -> Result<String, String> {
        let spec = lookup_app(&request.app)?;
        validate_invocation(&request.args)?;
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
        to_pretty_json(&native_mcp_config(&request)?)
    }
}

#[tool_handler(
    router = self.tool_router,
    name = "getcraft-apps-mcp",
    version = "0.1.0",
    instructions = "Use craft_list_apps first. Prefer craft_native_mcp_config when an app's complete first-party tool surface is needed. Use craft_cli_help before invoking an unfamiliar native command."
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
        Some("--help" | "-h" | "help") | None => print_usage(),
        Some(other) => {
            return Err(format!("unknown subcommand `{other}`; see `getcraft-mcp --help`").into());
        }
    }
    Ok(())
}

fn print_usage() {
    println!(
        "getcraft-mcp — MCP gateway for Storytold Craft CLIs\n\nUSAGE:\n  getcraft-mcp serve       Run the stdio MCP server\n  getcraft-mcp apps        Print native CLI discovery as JSON\n\nSet GETCRAFT_<APP>_CLI to an absolute executable path to override PATH discovery."
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
        return path.is_file().then_some(path);
    }

    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(spec.program))
        .find(|candidate| candidate.is_file())
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

fn validate_invocation(args: &[String]) -> Result<(), String> {
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
    if matches!(args.first().map(String::as_str), Some("mcp" | "serve")) {
        return Err("nested MCP and persistent CLI servers are not allowed through craft_run_cli; use craft_native_mcp_config instead".to_string());
    }
    Ok(())
}

async fn invoke(
    spec: AppSpec,
    args: Vec<String>,
    timeout_ms: u64,
) -> Result<InvocationResult, String> {
    let program = program_for(&spec)?;
    let mut command = Command::new(&program);
    command
        .args(&args)
        .kill_on_drop(true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("could not start {}: {error}", spec.program))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "could not capture child stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "could not capture child stderr".to_string())?;
    let stdout_task = tokio::spawn(capture_output(stdout));
    let stderr_task = tokio::spawn(capture_output(stderr));
    let status = match timeout(Duration::from_millis(timeout_ms), child.wait()).await {
        Ok(result) => {
            result.map_err(|error| format!("failed while waiting for {}: {error}", spec.program))?
        }
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            let _ = stdout_task.await;
            let _ = stderr_task.await;
            return Err(format!(
                "{} exceeded the {timeout_ms}ms timeout and was terminated",
                spec.program
            ));
        }
    };
    let stdout = stdout_task
        .await
        .map_err(|error| format!("stdout reader failed: {error}"))??;
    let stderr = stderr_task
        .await
        .map_err(|error| format!("stderr reader failed: {error}"))??;
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
    validate_optional_path(&request.read_root, "read_root")?;
    validate_optional_path(&request.write_root, "write_root")?;
    validate_optional_path(&request.workspace, "workspace")?;
    let mode = request.mode.as_deref().unwrap_or("headless");
    if mode != "headless" && mode != "bridge" {
        return Err("mode must be `headless` or `bridge`".to_string());
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
            append_photo_roots(&mut args, request);
            "For bridge mode, start PhotoCraft with matching automation roots and control authentication as documented by PhotoCraft."
        }
        ("photocraft", _) => {
            append_photo_roots(&mut args, request);
            "Omit a PhotoCraft automation root to grant no read or write filesystem authority in headless mode."
        }
        ("pdfcraft", "bridge") => {
            return Err("PdfCraft documents only a headless stdio MCP server".to_string());
        }
        ("pdfcraft", _) => {
            if let Some(root) = &request.workspace {
                args.extend(["--root".to_string(), root.clone()]);
            }
            "Set workspace to confine every PdfCraft MCP path. Without it, the CLI is not confined."
        }
        ("designcraft", "bridge") => {
            args.extend(["--connect".to_string(), require_connect(request)?]);
            "Start DesignCraft with its control port before connecting."
        }
        ("designcraft", _) => "Headless DesignCraft starts an in-process engine session.",
        ("effectcraft", "bridge") => {
            args.extend(["--bridge".to_string(), require_connect(request)?]);
            "Start EffectCraft with its control port before connecting."
        }
        ("effectcraft", _) => {
            "Use --autosave only when you intentionally want EffectCraft headless checkpoints to persist between MCP restarts."
        }
        ("lightcraft", "bridge") => {
            args.extend(["--connect".to_string(), require_connect(request)?]);
            "Connect mode drives a running LightCraft desktop app and avoids a library-lock conflict."
        }
        ("lightcraft", _) => {
            if let Some(library) = &request.workspace {
                args.extend(["--library".to_string(), library.clone()]);
            }
            "Set workspace to a LightCraft library for persistence. A headless server cannot open a library already locked by the desktop app."
        }
        ("vectorcraft", "bridge") => {
            args.extend(["--connect".to_string(), require_connect(request)?]);
            "For bridge mode, start VectorCraft with matching automation roots."
        }
        ("vectorcraft", _) => {
            append_vector_roots(&mut args, request);
            "Omit a VectorCraft read or write root to deny that kind of filesystem access in headless mode."
        }
        _ => return Err("unsupported app/mode combination".to_string()),
    };
    let program = locate_program(&spec).unwrap_or_else(|| PathBuf::from(spec.program));
    Ok(json!({
        "mcpServers": {
            spec.id: {
                "command": program,
                "args": args
            }
        },
        "notes": notes,
        "first_party_docs": format!("{}/blob/main/docs", spec.repository)
    }))
}

fn require_connect(request: &NativeMcpConfigRequest) -> Result<String, String> {
    let value = request
        .connect
        .as_deref()
        .ok_or_else(|| "connect is required when mode is `bridge`".to_string())?;
    validate_token(value, "connect")?;
    Ok(value.to_string())
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

fn append_photo_roots(args: &mut Vec<String>, request: &NativeMcpConfigRequest) {
    if let Some(root) = &request.read_root {
        args.extend(["--automation-read-root".to_string(), root.clone()]);
    }
    if let Some(root) = &request.write_root {
        args.extend(["--automation-write-root".to_string(), root.clone()]);
    }
}

fn append_vector_roots(args: &mut Vec<String>, request: &NativeMcpConfigRequest) {
    if let Some(root) = &request.read_root {
        args.extend(["--automation-read-root".to_string(), root.clone()]);
    }
    if let Some(root) = &request.write_root {
        args.extend(["--automation-write-root".to_string(), root.clone()]);
    }
}

fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map_err(|error| format!("could not serialize result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nested_servers() {
        assert!(validate_invocation(&["mcp".to_string()]).is_err());
        assert!(validate_invocation(&["serve".to_string()]).is_err());
    }

    #[test]
    fn vector_config_keeps_read_and_write_roots_separate() {
        let request = NativeMcpConfigRequest {
            app: "vectorcraft".to_string(),
            mode: None,
            connect: None,
            read_root: Some("/work/in".to_string()),
            write_root: Some("/work/out".to_string()),
            workspace: None,
        };
        let value = native_mcp_config(&request);
        assert!(value.is_ok());
        let rendered = match value {
            Ok(value) => value.to_string(),
            Err(error) => error,
        };
        assert!(rendered.contains("--automation-read-root"));
        assert!(rendered.contains("/work/in"));
        assert!(rendered.contains("--automation-write-root"));
        assert!(rendered.contains("/work/out"));
    }

    #[test]
    fn rejects_relative_confinement_root() {
        let request = NativeMcpConfigRequest {
            app: "pdfcraft".to_string(),
            mode: None,
            connect: None,
            read_root: None,
            write_root: None,
            workspace: Some("relative".to_string()),
        };
        assert!(native_mcp_config(&request).is_err());
    }
}
