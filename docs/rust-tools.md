# Rust Tools

Cargo tools used in Rust development projects for reference.

## Necessary

```bash
cargo install cargo-audit cargo-cache cargo-edit cargo-llvm-cov cargo-make cargo-modules cargo-outdated cargo-tree cargo-update cargo-watch
```

- `cargo-audit` - Scan dependencies for known security vulnerabilities
- `cargo-cache` - Manage Cargo cache directory, display size, and clean unnecessary files
- `cargo-edit` - Add, remove, and update dependencies in Cargo.toml from command line
- `cargo-llvm-cov` - Measure code coverage and generate reports
- `cargo-make` - Task runner/build tool for automating complex build flows and tasks
- `cargo-modules` - Visualize project module structure
- `cargo-outdated` - List available dependency updates
- `cargo-tree` - Display dependency tree structure
- `cargo-update` - Batch update installed Cargo binary crates
- `cargo-watch` - Monitor source code changes and automatically run commands on change

## Optional

```bash
cargo install cargo-release tauri-cli create-tauri-app
```

- `cargo-release` - Automate the release process for new versions
- `tauri-cli` - Tauri application development CLI
- `create-tauri-app` - Scaffolding tool for Tauri applications

## Additional Tools

- `sccache` - Cache compilation results to reduce build times
- `sea-orm-cli` - SeaORM CLI for migrations and entity generation
- `taplo-cli` - TOML formatter and linter
- `trunk` - Build tool for Rust + WebAssembly applications
- `wasm-bindgen-cli` - Generate bindings between Rust and JavaScript for WebAssembly
