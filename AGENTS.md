# AGENTS.md

## Project Overview

A Traefik Wasm plugin (Rust) that evaluates incoming requests against CEL rules and executes the matching action (e.g. blocking the request).

- Wasm host API: `http-wasm-guest`, registered in `src/main.rs`
- Rule config: YAML via `serde-saphyr`, example in `config/rules.yaml`
- Request flow: `src/main.rs` → `Plugin::handle_request` → `matcher::Matcher::evaluate` → action
- Version control: git

## Tools

- Use `fd` instead of `find` — e.g. `fd PATTERN`
- Use `rg` instead of `grep` — e.g. `rg PATTERN`

## Commands

| Task | Command |
| --- | --- |
| Build Wasm plugin (release) | `cargo build --target wasm32-wasip1 --release` |
| Run tests | `cargo test` |
| Check formatting | `cargo fmt --check` |
| Lint | `cargo clippy --target wasm32-wasip1` |

## Constraints

- **Rust**: edition 2024, minimum 1.88.0
- **Target**: `wasm32-wasip1` (WASI Preview 1)
- **Formatting**: max line width 100 (see `.rustfmt.toml`)
- **Error handling**: clippy denies `panic!` and `unwrap()` (see `[lints.clippy]` in `Cargo.toml`) — propagate errors via `Result`/`anyhow` instead. Exception are tests.
- **Memory**: minimize heap allocations; avoid copies/clones where possible
