use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request(app: &str) -> NativeMcpConfigRequest {
    NativeMcpConfigRequest {
        app: app.to_string(),
        ..Default::default()
    }
}

fn root(name: &str) -> String {
    env::temp_dir()
        .join("getcraft-config-test")
        .join(name)
        .display()
        .to_string()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn config_args(request: &NativeMcpConfigRequest) -> Result<Vec<String>, String> {
    let config = native_mcp_config(request)?;
    serde_json::from_value(config["mcpServers"][&request.app]["args"].clone())
        .map_err(|error| error.to_string())
}

#[test]
fn rejects_persistent_servers_even_after_global_options() {
    for spec in APPS {
        for server in ["mcp", "serve", "denoise-serve"] {
            assert!(validate_invocation(spec.id, &strings(&[server])).is_err());
        }
    }
    for app in ["filmcraft", "effectcraft"] {
        for args in [
            strings(&["--demo", "mcp"]),
            strings(&["--bridge", "9876", "mcp"]),
            strings(&["--bridge=127.0.0.1:9876", "mcp"]),
            strings(&["--json", "--empty", "mcp"]),
            strings(&["--project", "mcp", "serve"]),
        ] {
            assert!(validate_invocation(app, &args).is_err(), "{app}: {args:?}");
        }
    }
    assert!(validate_invocation("filmcraft", &strings(&["--", "mcp"])).is_err());
    assert!(validate_invocation("effectcraft", &strings(&["scene.ecproj", "mcp"])).is_err());
    assert!(validate_invocation("effectcraft", &strings(&["mcp", "scene.ecproj"])).is_err());
    assert!(
        validate_invocation(
            "effectcraft",
            &strings(&["--automation-read-root", "/work", "scene.ecproj", "mcp"])
        )
        .is_err()
    );
}

#[test]
fn allows_literal_json_and_key_value_parameters() {
    assert!(
        validate_invocation(
            "filmcraft",
            &strings(&[
                "exec",
                "file.newBin",
                "name=mcp",
                r#"{"description":"serve; $(echo anything)"}"#,
            ])
        )
        .is_ok()
    );
}

#[test]
fn server_names_are_allowed_as_data_values() {
    for (app, args) in [
        (
            "photocraft",
            strings(&[
                "batch",
                "--actions",
                "/work/actions.atn",
                "--action",
                "serve",
                "--in",
                "/work/in",
                "--out",
                "/work/out",
            ]),
        ),
        (
            "filmcraft",
            strings(&["--project", "mcp", "inspect", "project"]),
        ),
        (
            "filmcraft",
            strings(&["--project=mcp", "inspect", "project"]),
        ),
        ("filmcraft", strings(&["render", "--out", "serve"])),
        ("filmcraft", strings(&["import", "--", "mcp"])),
        ("effectcraft", strings(&["--filter", "serve", "commands"])),
        ("effectcraft", strings(&["--filter=serve", "commands"])),
        ("effectcraft", strings(&["--project", "mcp", "inspect"])),
        ("effectcraft", strings(&["--expression", "mcp", "eval"])),
        (
            "effectcraft",
            strings(&["scene.ecproj", "commands", "--filter", "mcp"]),
        ),
    ] {
        assert!(validate_invocation(app, &args).is_ok(), "{app}: {args:?}");
    }
}

#[test]
fn native_subcommand_matches_positional_and_option_rules() {
    for (app, args, expected) in [
        ("filmcraft", strings(&["--project", "mcp"]), None),
        (
            "filmcraft",
            strings(&["--", "--project", "mcp"]),
            Some("--project"),
        ),
        ("effectcraft", strings(&["--", "mcp"]), Some("--")),
        (
            "effectcraft",
            strings(&["scene.ecproj", "mcp", "--project", "other.ecproj"]),
            Some("scene.ecproj"),
        ),
        (
            "effectcraft",
            strings(&["scene.ecproj", "other.ecproj", "mcp"]),
            Some("other.ecproj"),
        ),
        (
            "photocraft",
            strings(&["batch", "--action", "serve"]),
            Some("batch"),
        ),
    ] {
        assert_eq!(
            cli_args::subcommand(app, &args),
            expected,
            "{app}: {args:?}"
        );
    }
}

#[test]
fn enforces_argument_bounds() {
    assert!(validate_invocation("filmcraft", &vec!["a".to_string(); MAX_ARGS + 1]).is_err());
    assert!(validate_invocation("filmcraft", &["a\0b".to_string()]).is_err());
    assert!(validate_invocation("filmcraft", &[String::new()]).is_err());
    assert!(validate_invocation("filmcraft", &["a".repeat(8193)]).is_err());
    assert!(validate_invocation("filmcraft", &vec!["a".repeat(8192); 5]).is_err());
    assert!(validate_invocation("filmcraft", &strings(&["--version"])).is_ok());
}

#[test]
fn validates_timeout_bounds() {
    assert_eq!(checked_timeout(None), Ok(DEFAULT_TIMEOUT_MS));
    assert_eq!(checked_timeout(Some(100)), Ok(100));
    assert_eq!(checked_timeout(Some(MAX_TIMEOUT_MS)), Ok(MAX_TIMEOUT_MS));
    assert!(checked_timeout(Some(99)).is_err());
    assert!(checked_timeout(Some(MAX_TIMEOUT_MS + 1)).is_err());
}

#[test]
fn catalog_commands_request_json_and_effect_schemas() -> TestResult {
    assert_eq!(
        lookup_app("filmcraft")?.catalog_args,
        ["commands", "--json"]
    );
    assert_eq!(
        lookup_app("lightcraft")?.catalog_args,
        ["commands", "--json"]
    );
    assert_eq!(
        lookup_app("effectcraft")?.catalog_args,
        ["commands", "--schemas", "--json"]
    );
    assert_eq!(lookup_app("pdfcraft")?.catalog_args, ["tools"]);
    Ok(())
}

#[test]
fn all_apps_have_headless_stdio_configs() -> TestResult {
    for spec in APPS {
        let config = native_mcp_config(&request(spec.id))?;
        assert_eq!(config["mcpServers"][spec.id]["type"], "stdio");
        assert_eq!(config["mcpServers"][spec.id]["args"][0], "mcp");
    }
    Ok(())
}

#[test]
fn vector_headless_never_autoconnects_to_a_desktop() -> TestResult {
    assert_eq!(config_args(&request("vectorcraft"))?, ["mcp", "--headless"]);
    Ok(())
}

#[test]
fn supported_configs_preserve_separate_read_and_write_roots() -> TestResult {
    for app in [
        "photocraft",
        "pdfcraft",
        "designcraft",
        "effectcraft",
        "vectorcraft",
    ] {
        let mut config = request(app);
        config.read_root = Some(root("in"));
        config.write_root = Some(root("out"));
        let args = config_args(&config)?;
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--automation-read-root", &root("in")])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--automation-write-root", &root("out")])
        );
    }
    Ok(())
}

#[test]
fn pdf_workspace_intersects_roots_and_supports_compact_catalog() -> TestResult {
    let mut config = request("pdfcraft");
    config.workspace = Some(root("workspace"));
    config.read_root = Some(root("workspace/in"));
    config.write_root = Some(root("workspace/out"));
    config.compact = Some(true);
    assert_eq!(
        config_args(&config)?,
        [
            "mcp",
            "--root",
            &root("workspace"),
            "--automation-read-root",
            &root("workspace/in"),
            "--automation-write-root",
            &root("workspace/out"),
            "--compact"
        ]
    );
    Ok(())
}

#[test]
fn older_binaries_cannot_silently_lose_requested_roots() -> TestResult {
    for app in ["pdfcraft", "designcraft"] {
        let spec = lookup_app(app)?;
        assert!(verify_root_help(spec, "mcp [--root DIR]", true, true, true).is_err());
        assert!(verify_root_help(spec, "--automation-read-root DIR", true, true, true).is_err());
        assert!(verify_root_help(spec, "--automation-read-root DIR", true, true, false).is_ok());
        assert!(
            verify_root_help(
                spec,
                "--automation-read-root --automation-write-root",
                false,
                true,
                true
            )
            .is_err()
        );
        assert!(
            verify_root_help(
                spec,
                "--automation-read-root --automation-write-root",
                true,
                true,
                true
            )
            .is_ok()
        );
    }
    Ok(())
}

#[test]
fn roots_are_not_silently_ignored_for_unsupported_apps_or_bridges() {
    for app in ["filmcraft", "lightcraft"] {
        let mut config = request(app);
        config.read_root = Some(root("in"));
        assert!(native_mcp_config(&config).is_err());
    }
    for app in ["photocraft", "designcraft", "vectorcraft"] {
        let mut config = request(app);
        config.mode = Some("bridge".to_string());
        config.connect = Some("7979".to_string());
        config.write_root = Some(root("out"));
        assert!(native_mcp_config(&config).is_err());
    }
}

#[test]
fn effectcraft_repeatable_roots_work_in_both_modes() -> TestResult {
    for mode in ["headless", "bridge"] {
        let mut config = request("effectcraft");
        config.mode = Some(mode.to_string());
        config.connect = (mode == "bridge").then(|| "9877".to_string());
        config.read_root = Some(root("a"));
        config.read_roots = Some(vec![root("a"), root("b")]);
        config.write_roots = Some(vec![root("out-a"), root("out-b")]);
        let args = config_args(&config)?;
        assert_eq!(
            args.iter()
                .filter(|arg| *arg == "--automation-read-root")
                .count(),
            2
        );
        assert_eq!(
            args.iter()
                .filter(|arg| *arg == "--automation-write-root")
                .count(),
            2
        );
    }
    let mut other = request("photocraft");
    other.read_roots = Some(vec![root("a")]);
    assert!(native_mcp_config(&other).is_err());
    Ok(())
}

#[test]
fn effectcraft_autosave_is_explicit_and_headless_only() -> TestResult {
    let mut config = request("effectcraft");
    assert!(!config_args(&config)?.contains(&"--autosave".to_string()));
    config.autosave = Some(true);
    assert!(config_args(&config)?.contains(&"--autosave".to_string()));
    config.mode = Some("bridge".to_string());
    config.connect = Some("9877".to_string());
    assert!(native_mcp_config(&config).is_err());
    Ok(())
}

#[test]
fn photo_bridge_references_token_file_without_reading_it() -> TestResult {
    let mut config = request("photocraft");
    config.mode = Some("bridge".to_string());
    config.connect = Some("9876".to_string());
    config.control_token_file = Some(root("token-file-does-not-exist"));
    let args = config_args(&config)?;
    assert_eq!(
        args,
        [
            "mcp",
            "--bridge",
            "127.0.0.1:9876",
            "--control-token-file",
            &root("token-file-does-not-exist")
        ]
    );
    config.mode = None;
    config.connect = None;
    assert!(native_mcp_config(&config).is_err());
    Ok(())
}

#[test]
fn compact_is_supported_only_for_pdf_and_light() -> TestResult {
    for spec in APPS {
        let mut config = request(spec.id);
        config.compact = Some(true);
        if matches!(spec.id, "pdfcraft" | "lightcraft") {
            assert!(config_args(&config)?.contains(&"--compact".to_string()));
        } else {
            assert!(native_mcp_config(&config).is_err());
        }
    }
    let mut light = request("lightcraft");
    light.workspace = Some(root("library"));
    light.compact = Some(true);
    assert_eq!(
        config_args(&light)?,
        ["mcp", "--library", &root("library"), "--compact"]
    );
    Ok(())
}

#[test]
fn bridge_requires_an_explicit_loopback_endpoint() -> TestResult {
    let mut config = request("filmcraft");
    config.mode = Some("bridge".to_string());
    assert!(native_mcp_config(&config).is_err());
    for invalid in [
        "example.org:9876",
        "192.168.1.4:9876",
        "0",
        "65536",
        "127.0.0.1:0",
        "0.0.0.0:9876",
        "::1",
        "--help",
    ] {
        config.connect = Some(invalid.to_string());
        assert!(native_mcp_config(&config).is_err(), "{invalid}");
    }
    for valid in ["9876", "localhost:9876", "127.0.0.1:9876"] {
        config.connect = Some(valid.to_string());
        assert_eq!(config_args(&config)?, ["mcp", "--bridge", "127.0.0.1:9876"]);
    }
    config.connect = Some("[::1]:9876".to_string());
    assert_eq!(require_connect(&config)?, "[::1]:9876");
    Ok(())
}

#[test]
fn rejects_invalid_or_ignored_config_options() {
    let mut config = request("pdfcraft");
    config.mode = Some("bridge".to_string());
    config.connect = Some("9876".to_string());
    assert!(native_mcp_config(&config).is_err());
    config.mode = None;
    assert!(native_mcp_config(&config).is_err());
    config.connect = None;
    config.workspace = Some("relative".to_string());
    assert!(native_mcp_config(&config).is_err());
    let mut film = request("filmcraft");
    film.workspace = Some(root("workspace"));
    assert!(native_mcp_config(&film).is_err());
    assert!(
        serde_json::from_value::<NativeMcpConfigRequest>(
            json!({"app":"pdfcraft","read_rooot":"/work"})
        )
        .is_err()
    );
}

#[test]
fn unknown_apps_are_rejected() {
    assert!(lookup_app("arbitrary-program").is_err());
}

#[test]
fn search_preserves_schemas_and_paginates_matches() -> TestResult {
    let catalog = json!([
        {"id":"file.open", "params":"path"},
        {"id":"file.export", "params":{"format":{"enum":["SVG", "PNG"]}}},
        {"name":"doc_export", "input_schema":{"type":"object"}},
        {"id":"export.video", "label":"Render"}
    ])
    .to_string();
    let mut search = SearchRequest {
        app: "vectorcraft".to_string(),
        query: Some("EXPORT".to_string()),
        limit: Some(2),
        ..Default::default()
    };
    let result = search_catalog(&search, &catalog)?;
    assert_eq!(result["total"], 4);
    assert_eq!(result["matched"], 3);
    assert_eq!(result["next_offset"], 2);
    assert_eq!(result["entries"][0]["params"]["format"]["enum"][0], "SVG");
    search.offset = Some(2);
    let result = search_catalog(&search, &catalog)?;
    assert_eq!(result["entries"][0]["id"], "export.video");
    assert!(result["next_offset"].is_null());
    search.offset = Some(usize::MAX);
    assert_eq!(search_catalog(&search, &catalog)?["entries"], json!([]));
    Ok(())
}

#[test]
fn search_validates_limits_kinds_and_native_json() -> TestResult {
    let mut search = SearchRequest {
        app: "lightcraft".to_string(),
        ..Default::default()
    };
    let light = lookup_app("lightcraft")?;
    search.kind = Some("controls".to_string());
    assert_eq!(search_catalog_args(light, &search)?, ["controls", "--json"]);
    assert!(search_catalog_args(lookup_app("filmcraft")?, &search).is_err());
    search.limit = Some(0);
    assert!(search_catalog_args(light, &search).is_err());
    search.limit = Some(101);
    assert!(search_catalog_args(light, &search).is_err());
    search.limit = None;
    search.kind = Some("execute".to_string());
    assert!(search_catalog_args(light, &search).is_err());
    assert!(search_catalog(&search, "not json").is_err());
    assert!(search_catalog(&search, "{}").is_err());
    Ok(())
}

#[test]
fn audit_snapshot_covers_every_app_with_pinned_sources() {
    for app in APPS {
        let audit = compatibility::for_app(app.id);
        assert_eq!(audit["repository"], app.repository);
        let commit = audit["audited_commit"].as_str().unwrap_or_default();
        assert_eq!(commit.len(), 40);
        assert!(audit["docs"].as_str().unwrap_or_default().contains(commit));
        assert!(
            audit["latest_release"]["tag"]
                .as_str()
                .is_some_and(|tag| tag.starts_with('v'))
        );
        assert!(audit["checked_on"].as_str().is_some());
    }
}

#[tokio::test]
async fn output_capture_drains_beyond_the_memory_limit() -> TestResult {
    let bytes = vec![b'x'; OUTPUT_LIMIT_BYTES + 500];
    let captured = capture_output(bytes.as_slice()).await?;
    assert_eq!(captured.text.len(), OUTPUT_LIMIT_BYTES);
    assert!(captured.truncated);
    let small = capture_output(b"hello".as_slice()).await?;
    assert_eq!(small.text, "hello");
    assert!(!small.truncated);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn child_stdin_is_closed_and_arguments_are_literal() -> TestResult {
    let result = invoke_program(
        lookup_app("filmcraft")?,
        PathBuf::from("/bin/sh"),
        strings(&[
            "-c",
            "read -r line || printf 'eof:'; printf '%s' \"$1\"",
            "test",
            "$(echo not-evaluated); mcp",
        ]),
        2000,
    )
    .await?;
    assert!(result.success);
    assert_eq!(result.stdout, "eof:$(echo not-evaluated); mcp");
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn output_pipe_drain_is_covered_by_timeout() -> TestResult {
    let started = std::time::Instant::now();
    let result = invoke_program(
        lookup_app("filmcraft")?,
        PathBuf::from("/bin/sh"),
        strings(&["-c", "sleep 0.5 & exit 0"]),
        100,
    )
    .await;
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(2));
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn timed_out_child_is_terminated_with_partial_output_warning() -> TestResult {
    let result = invoke_program(
        lookup_app("filmcraft")?,
        PathBuf::from("/bin/sh"),
        strings(&["-c", "exec sleep 10"]),
        100,
    )
    .await;
    assert!(result.is_err());
    assert!(result.err().unwrap_or_default().contains("may be partial"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn discovery_requires_an_executable_file() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("not-executable");
    std::fs::write(&path, "test fixture")?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    assert!(!is_executable(&path));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
    assert!(is_executable(&path));
    assert!(!is_executable(directory.path()));
    Ok(())
}
