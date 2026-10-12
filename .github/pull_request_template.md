## Summary

<!-- Explain the user-visible change. -->

## Verification

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] Plugin manifests validate for the target client(s)

## Safety and compatibility

- [ ] This change does not invoke a shell from MCP input.
- [ ] Native CLI/MCP syntax was checked against the linked upstream documentation.
