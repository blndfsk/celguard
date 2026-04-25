# AGENTS.md

## Build & Test

```bash
# Build WASM plugin
cargo build --target wasm32-wasip1 --release

# Run tests
cargo test

# Lint (fmt + clippy)
cargo fmt --check
cargo clippy --target wasm32-wasip1
```

## Local Development

`./run.sh` builds the plugin, starts Traefik with the plugin, and runs a whoami backend. Requires podman and buildah.

## Key Constraints

- **Edition**: 2024, requires Rust 1.88.0+
- **Target**: wasm32-wasip1 (WASI Preview 1)
- **Max line width**: 100 (see `.rustfmt.toml`)
- **Memory efficiency**: Minimize allocs, avoid copies/clones where possible