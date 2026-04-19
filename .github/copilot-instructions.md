# Copilot Instructions for celguard

## Project Overview

**celguard** is a Traefik plugin written in Rust that filters HTTP requests using the Common Expression Language (CEL). It's compiled to WebAssembly (WASM) and deployed as a Traefik middleware.

### Architecture

The plugin is structured around three core modules:

1. **Config** (`src/config/`) - Parses YAML configuration from Traefik with:
   - `Rule` definitions: name, tests (CEL expressions), optional action
   - `Action` definitions: optional response (status, headers, body) and continue flag
   - Deserialization helpers for CEL programs

2. **Matcher** (`src/matcher/`) - Evaluates requests against rules:
   - `Matcher` struct evaluates all rules in order, returns `Outcome::Match` or `NoMatch`
   - `Request` struct (wrapped as CEL opaque type) exposes request properties (path, method, version, headers)
   - Custom CEL functions: `to_lower`, `equals` for string operations
   - Outcome enum can optionally return matched action name

3. **Handler** (`src/handler/`) - Executes matched actions:
   - `Handler` applies response modifications (status, headers, body)
   - Returns boolean indicating whether to continue processing

The main `Plugin` struct in `main.rs` orchestrates: register plugin → read config → match request → execute action → return control to Traefik.

## Build and Test

### Build Targets

The project targets **wasm32-wasip1** (WebAssembly for Traefik):

```bash
# Debug build
cargo build --target wasm32-wasip1

# Release build
cargo build --release --target wasm32-wasip1
```

Output: `target/wasm32-wasip1/{debug|release}/celguard.wasm`

### Testing

**Unit Tests** (29 tests covering config parsing, matcher logic, and request handling):

```bash
cargo test
```

Individual test modules can be run with:

```bash
cargo test matcher::        # Matcher tests
cargo test config::         # Config tests  
cargo test config::tests::test_read  # Specific test
```

Tests use `#[test_log::test]` and `TestResult` for logging support.

**Integration Testing** via `./run.sh` (requires Podman/buildah):
- Builds WASM, creates Traefik container, deploys whoami service
- Test endpoints: `curl -H "user-agent: curl" http://localhost:8080 -H "Host: whoami.localhost"`
- Rules are loaded from `config/rules.yaml` (hot-reload via volume mount)

### Formatting

Max line width: 100 characters (see `.rustfmt.toml`). Format with:

```bash
cargo fmt
```

## Key Conventions

- **CEL Integration**: Rules are compiled at config load time into `Program` objects. Custom functions are registered in the `Matcher::new()` method. String operations (matches, contains, etc.) are inherited from CEL library.
- **Error Handling**: Matcher and handler errors return sensible defaults: matcher errors return `NoMatch`, handler errors execute default action (403 response). Logged with `log::error!`.
- **Config Path Mount**: Traefik passes config via `paths` array (filesystem mount point). Alternatively, inline config via `config` field in host settings.
- **WASM Constraints**: Plugin runs in WASM sandbox with limited I/O. Use `http_wasm_guest` crate for host interaction (request/response objects, logging).
- **Opaque Types**: The `Request` struct implements CEL's `Opaque` trait to expose custom properties to CEL expressions. HashMap fields are exposed as subscriptable objects in CEL (e.g., `request.header['user-agent']`).

## Dependencies

### Runtime
- **http-wasm-guest** (0.11): WASM plugin interface for Traefik
- **cel** (0.13): CEL expression evaluation
- **serde** (1.0): Serialization framework with `derive` feature
- **serde-saphyr** (0.0.23): YAML deserialization
- **anyhow** (1.0): Error handling
- **log** (0.4): Logging (Traefik provides the logging backend)

### Dev
- **test-log** (0.2): Logging support in tests
- **testresult** (0.4.1): Test result types for better error messages

## Configuration Format

Rules and actions are YAML. See `config/rules.yaml` for a complete working example. Key points:
- Actions are optional and keyed by name (e.g., `allow`, `block`, `method_not_allowed`)
- Rules are evaluated in order; first match wins
- Each test in a rule is a CEL expression; any test matching triggers the rule (OR logic)
- `continue: true` allows request to proceed through Traefik without blocking
- `log` field sets log level: `off` (default), `debug`, `info`, `warn`, `error`

Example structure:
```yaml
actions:
  block:
    response: { status: 403, body: blocked }
  allow:
    continue: true
rules:
  - name: method
    log: warn
    tests:
      - request.method in ['GET','HEAD'] != true
    action: method_not_allowed
  - name: default
    tests:
      - true
    action: block
```
