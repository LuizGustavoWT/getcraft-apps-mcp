# Contributing

[Português (Brasil)](CONTRIBUTING.pt-BR.md)

Thanks for helping make the Craft apps easier to use with agents. Contributions are welcome from anyone.

## Before you start

- Read the upstream application's current MCP/CLI documentation before changing an adapter, skill, or example.
- Compare release tags with audited `main` and installed help: a feature in source is not necessarily shipped. Update [upstream-versions.json](upstream-versions.json) with exact commits, tested versions and unreleased-feature notes; never label a source-only audit as runtime coverage.
- Keep the gateway independent. Do not copy implementation code, assets, or vendored dependencies from the upstream apps.
- Preserve the safety boundary: no shell execution, null child stdin, no persistent servers through `craft_run_cli`, bounded execution, loopback-only bridges, and clear filesystem authority. Unsupported requested restrictions must fail, never disappear silently. Do not read authentication-token contents into logs or test output.
- Discuss a broad design or a breaking tool-schema change in an issue first.

## Development setup

```sh
git clone https://github.com/LuizGustavoWT/getcraft-apps-mcp
cd getcraft-apps-mcp
cargo test --locked
```

Run these checks before opening a pull request (Python 3.11+, standard library only):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_package.py
cargo build --locked
python3 scripts/smoke_mcp.py target/debug/getcraft-mcp
```

The default smoke test exercises MCP initialization/tool discovery without requiring Craft CLIs. With all seven native CLIs installed, also run:

```sh
python3 scripts/smoke_mcp.py target/debug/getcraft-mcp --native
```

Native checks are read-only; test document-changing behavior only with disposable fixtures and explicit output paths. Cover catalog JSON/schema preservation, search pagination, unsupported options, server-subcommand rejection, timeouts and confinement errors when those behaviors change.

When available, run `claude plugin validate ./plugins/getcraft-apps-mcp` and test Codex installation separately. Do not report a missing client's validation as passed. Keep all manifests and Cargo versions aligned. Git marketplace examples must include both `.agents/plugins` and `plugins` in sparse checkouts, or use a full checkout. See the README for the normal Git marketplace update flow; local development caches are separate from published versions.

## Pull requests

1. Fork the repository and create a focused branch.
2. Add or update tests for behavior changed in Rust.
3. Update English and Portuguese documentation together, plus `CHANGELOG.md` and the audit snapshot when applicable.
4. Link official upstream documentation for any changed native CLI/MCP syntax.
5. State which Craft app versions and client surfaces you tested, and separate mock, native read-only, and actual editing coverage.

Do not commit credentials, private project files, generated creative assets you do not own, or local application databases. Keep pull requests focused and explain any behavior that writes, overwrites, imports, exports, or persists files.

## Maintainer policy

The project accepts public issues and fork-based pull requests. `main` requires the repository owner's review through `CODEOWNERS`, and only `@LuizGustavoWT` has repository write/admin access. That account alone merges pull requests and closes issues opened by other users.

## Reporting security issues

Please do not disclose a vulnerability in a public issue. Use GitHub's private security advisory flow for this repository when it is available, or contact the repository owner through GitHub.
