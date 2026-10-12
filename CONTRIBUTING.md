# Contributing

[Português (Brasil)](CONTRIBUTING.pt-BR.md)

Thanks for helping make the Craft apps easier to use with agents. Contributions are welcome from anyone.

## Before you start

- Read the upstream application's current MCP/CLI documentation before changing an adapter, skill, or example.
- Keep the gateway independent. Do not copy implementation code, assets, or vendored dependencies from the upstream apps.
- Preserve the gateway's safety boundary: no shell execution from MCP input, no nested native MCP process through `craft_run_cli`, bounded command execution, and clear filesystem authority.
- Discuss a broad design or a breaking tool-schema change in an issue first.

## Development setup

```sh
git clone https://github.com/LuizGustavoWT/getcraft-apps-mcp
cd getcraft-apps-mcp
cargo test
```

Run these checks before opening a pull request:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
claude plugin validate ./plugins/getcraft-apps-mcp
```

Also test the Codex marketplace from a local checkout or GitHub source before changing its packaging.

## Pull requests

1. Fork the repository and create a focused branch.
2. Add or update tests for behavior changed in Rust.
3. Update the English and Portuguese documentation together.
4. Link official upstream documentation for any changed native CLI/MCP syntax.
5. State which Craft apps and client surfaces you tested.

Do not commit credentials, private project files, generated creative assets you do not own, or local application databases. Keep pull requests focused and explain any behavior that writes, overwrites, imports, exports, or persists files.

## Maintainer policy

The project accepts public issues and fork-based pull requests. `main` requires the repository owner's review through `CODEOWNERS`, and only `@LuizGustavoWT` has repository write/admin access. That account alone merges pull requests and closes issues opened by other users.

## Reporting security issues

Please do not disclose a vulnerability in a public issue. Use GitHub's private security advisory flow for this repository when it is available, or contact the repository owner through GitHub.
