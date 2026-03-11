# Contributing to Oxidoc

We welcome contributions to Oxidoc! This guide covers development setup, code conventions, and the pull request process.

## Development Setup

### Prerequisites

- Rust 1.70+ (check with `rustc --version`)
- Cargo
- For Wasm targets: `rustup target add wasm32-unknown-unknown`

### Clone and Build

```bash
git clone https://github.com/oxidoc/oxidoc.git
cd oxidoc
cargo build
cargo test --workspace
```

### Development Commands

```bash
# Build all crates
cargo build -p oxidoc-cli -p oxidoc-core

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint with strict warnings
cargo clippy --all-targets -- -D warnings

# Run benchmarks (no-build mode)
cargo bench -p oxidoc-core --no-run
```

## Workspace Architecture

| Crate | Type | Responsibility |
|:---|:---|:---|
| `oxidoc-cli` | Binary | CLI commands, dev server, HMR integration |
| `oxidoc-core` | Library | Config parsing, RDX→HTML rendering, build engine |
| `oxidoc-island` | Library | Island component trait definition |
| `oxidoc-components` | Library | Built-in Leptos components (Callout, Tabs, etc.) |
| `oxidoc-registry` | cdylib | Wasm hydration entry point (browser runtime) |

## Code Conventions

### No `.unwrap()` in Library Code

Library crates must propagate errors instead of panicking:

```rust
// Bad
let config = toml::from_str(content).unwrap();

// Good
let config: OxidocConfig = toml::from_str(content)
    .map_err(|e| OxidocError::ConfigParse { /* ... */ })?;
```

Benchmarks and examples can use `.unwrap()` since they are dev-only.

### mod.rs Organization

Every `mod.rs` file MUST contain only `pub mod` and `pub use` declarations:

```rust
// src/lib.rs or src/some_module/mod.rs
pub mod builder;
pub mod config;
pub mod renderer;

pub use builder::build_site;
pub use config::{OxidocConfig, load_config};
```

No traits, types, impl blocks, or logic in mod.rs files.

### File Size Limits

| Type | Soft Limit | Hard Limit | Action |
|:---|:---|:---|:---|
| Type definitions | 100 lines | 200 lines | Split by domain |
| Component implementations | 200 lines | 400 lines | Split by concern |
| Config/parsing logic | 200 lines | 400 lines | Split by section |

### Environment Separation

Strict isolation between host and client environments:

- **Host crates** (`oxidoc-cli`, `oxidoc-core`): NO `web-sys`, `wasm-bindgen`, or browser APIs
- **Client crates** (`oxidoc-island`, `oxidoc-components`, `oxidoc-registry`): NO `tokio`, `axum`, or file I/O

## Testing Strategy

### Unit Tests (in-file)

Place tests in the same file as the code being tested using `#[cfg(test)] mod tests { ... }`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_minimal() {
        let toml = "[project]\nname = \"Test\"";
        let config = parse_config(toml).unwrap();
        assert_eq!(config.project.name, "Test");
    }
}
```

Benefits: Test private functions, low overhead, tight coupling with implementation.

### Integration Tests

Create `tests/` directory at the project root for high-level API tests:

```
tests/
├── integration_test.rs      # Standalone integration test
├── api/
│   ├── mod.rs               # Test group organization
│   └── build_test.rs        # Specific API tests
```

Benefits: Test public API only, loose coupling, high-level scenarios.

## Pull Request Process

1. Fork the repository and create a feature branch
2. Make changes following code conventions above
3. Write tests: unit tests in-file, integration tests in `tests/`
4. Run the full test suite and checks:
   ```bash
   cargo test --workspace
   cargo fmt --all --check
   cargo clippy --all-targets -- -D warnings
   cargo bench -p oxidoc-core --no-run
   ```
5. Submit a PR with a clear description of what changed and why
6. Respond to review feedback promptly

## Pre-Commit Checklist

- [ ] Host crates have zero `web-sys` / `wasm-bindgen` imports
- [ ] Client crates have zero `tokio` / `axum` imports
- [ ] All mod.rs files contain ONLY `pub mod` + `pub use`
- [ ] No file exceeds 400 lines
- [ ] No `.unwrap()` in library code
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all --check` passes

## Questions?

Open an issue or discussion on GitHub. We're here to help!
