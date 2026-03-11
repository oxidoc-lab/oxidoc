# CLAUDE.md - Oxidoc Project

---

## Project Overview

Oxidoc is a **next-generation documentation engine** written entirely in Rust. It generates blazing-fast static documentation sites with interactive WebAssembly islands, powered by Leptos. It consumes `.rdx` files (via the `rdx-parser` crate) and produces pre-rendered HTML with `<oxidoc-island>` hydration placeholders.

**Architecture Position:**

```
User writes .rdx files
        │
        ▼
┌──────────────────────────────────────────────────────┐
│                    oxidoc-cli                         │
│   Commands: `oxidoc dev` (HMR) · `oxidoc build` (SSG)│
└──────────────────────┬───────────────────────────────┘
                       │ delegates to
┌──────────────────────▼───────────────────────────────┐
│                   oxidoc-core                         │
│   Reads oxidoc.toml · Parses .rdx via rdx-parser     │
│   Converts AST → static HTML + <oxidoc-island> tags  │
└──────────────────────────────────────────────────────┘

Browser loads HTML → downloads oxidoc-registry.wasm
        │
┌───────▼──────────────────────────────────────────────┐
│                 oxidoc-registry                       │
│   Scans DOM for <oxidoc-island> · hydrates Leptos    │
│   components from oxidoc-components                  │
└──────────────────────────────────────────────────────┘
```

---

## Workspace Structure

```
oxidoc/                             # Cargo workspace root
├── Cargo.toml                      # workspace members + shared deps
├── CLAUDE.md                       # This file
├── .cargo/config.toml              # Local rdx path overrides (gitignored)
├── .gitignore
│
├── oxidoc-cli/                     # Binary crate: `oxidoc` CLI
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                 # Entry point (minimal — delegates to oxidoc-core)
│
├── oxidoc-core/                    # Library: build engine brain
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # pub mod + pub use
│       ├── config.rs               # oxidoc.toml deserialization
│       └── renderer.rs             # RDX AST → HTML + island placeholders
│
├── oxidoc-island/                  # Library: OxidocIsland trait definition
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                  # pub trait OxidocIsland
│
├── oxidoc-components/              # Library: built-in island implementations
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # pub mod declarations
│       ├── callout.rs              # <Callout> component
│       ├── tabs.rs                 # <Tabs> component
│       └── code_block.rs           # <CodeBlock> component
│
├── oxidoc-registry/                # cdylib: Wasm entry point for browser
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                  # DOM scanning + island hydration
│
└── resource/                       # Local reference materials (gitignored)
    └── TDD.md
```

---

## Two Execution Environments

### Build Engine (Host — native Rust)

Crates: `oxidoc-cli`, `oxidoc-core`

- Runs natively via `cargo install oxidoc-cli`
- Reads `oxidoc.toml`, crawls `.rdx` files, invokes `rdx-parser`
- Produces static HTML with `<oxidoc-island>` placeholders + JSON props
- Ships pre-compiled Wasm — **never** invokes `cargo build` during user builds
- Zero Node.js dependency (Tailwind CSS Standalone for styling)

### Frontend Runtime (Client — WebAssembly)

Crates: `oxidoc-island`, `oxidoc-components`, `oxidoc-registry`

- `oxidoc-registry` compiles to `cdylib` Wasm, loaded by the browser
- Scans DOM for `<oxidoc-island>` elements, hydrates with Leptos components
- Aggressive code-splitting: only download Wasm chunks for islands on the page
- Shadow DOM isolation for complex islands (ApiPlayground)

---

## Rules for New Code

1. **mod.rs = ONLY `pub mod` + `pub use`** — no traits, types, impl, or logic
2. **One concern = one file** — `callout.rs`, `renderer.rs`, NOT `components.rs`
3. **Environment separation is absolute** — host crates never depend on `web-sys`/`wasm-bindgen`; client crates never depend on `tokio`/`axum`
4. **No `.unwrap()` in library code** — use `thiserror`/`miette` for errors
5. **100 small files > 1 large file** — split at logical boundaries

---

## Dependencies

### RDX Integration

```toml
# Cargo.toml uses published crate versions
rdx-parser = "0.1.0"
rdx-ast = "0.1.0"

# .cargo/config.toml patches to local paths for development (gitignored)
[patch.crates-io]
rdx-parser = { path = "/home/farhan/Projects/rdx-lang/rdx/rdx-parser" }
rdx-ast = { path = "/home/farhan/Projects/rdx-lang/rdx/rdx-ast" }
```

When enhancing rdx for Oxidoc purposes, modify the local rdx-lang checkout. The `.cargo/config.toml` patch ensures Oxidoc always builds against the local version during development.

### Key Dependencies

| Crate | Purpose |
|:---|:---|
| `rdx-parser` / `rdx-ast` | RDX file parsing and AST types |
| `leptos` | Reactive UI for Wasm island components |
| `axum` + `tokio` | Dev server with HMR (`oxidoc dev`) |
| `pulldown-cmark` | Markdown portion of RDX parsing |
| `openapiv3` | OpenAPI spec ingestion |
| `miette` | Beautiful terminal error diagnostics |
| `clap` | CLI argument parsing |
| `notify` | File system watching for HMR |
| `wasm-bindgen` + `web-sys` | Wasm ↔ browser DOM interop |

---

## The Island Protocol

All communication between the build engine and the Wasm runtime goes through HTML:

```html
<oxidoc-island data-island-type="callout" data-props='{"kind":"warning","content":"..."}'></oxidoc-island>
```

- **Build engine** produces these placeholders (serializes props to JSON)
- **Registry** scans for them and calls `OxidocIsland::mount()` on the matching component
- **Shadow DOM** is used for complex islands to prevent CSS leakage

### Event Bridging (Static HTML ↔ Wasm)

| Event | Source | Payload |
|:---|:---|:---|
| `oxidoc:nav_sync` | Static sidebar | `{ "target_id": "..." }` |
| `oxidoc:update_playground` | Static code example | `{ "method": "GET", "params": {...} }` |
| `oxidoc:copy_code` | Wasm CodeBlock | `{ "text": "..." }` |

---

## Build Commands

```bash
# Check everything compiles (host crates only — no wasm target needed)
cargo build -p oxidoc-cli -p oxidoc-core

# Check wasm crates (requires wasm32-unknown-unknown target)
cargo build -p oxidoc-registry --target wasm32-unknown-unknown

# Run all tests
cargo test --workspace

# Checks
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings

# Specific crate
cargo test -p oxidoc-core
cargo test -p oxidoc-island
```

---

## File Size Limits

| File Type | Soft Limit | Hard Limit | Action |
|:---|:---|:---|:---|
| Type definitions | 100 lines | 200 lines | Split by domain |
| Component implementations | 200 lines | 400 lines | Split by concern |
| Config / parsing logic | 200 lines | 400 lines | Split by section |
| mod.rs files | 20 lines | 50 lines | ONLY pub mod + pub use |

---

## Testing Strategy

### Unit Tests (in-file `#[cfg(test)]`)

- Config deserialization roundtrips
- Island placeholder HTML generation
- Component props parsing
- RDX tag interception logic

### Integration Tests (`tests/`)

- Full `.rdx` file → HTML pipeline
- `oxidoc.toml` loading + validation
- Registry hydration (requires wasm testing harness)

---

## Pre-Commit Checklist

- [ ] Host crates (`oxidoc-cli`, `oxidoc-core`) have zero `web-sys` / `wasm-bindgen` imports
- [ ] Client crates (`oxidoc-island`, `oxidoc-components`, `oxidoc-registry`) have zero `tokio` / `axum` imports
- [ ] All mod.rs files contain ONLY `pub mod` + `pub use`
- [ ] No file exceeds 400 lines
- [ ] No `.unwrap()` in library code
- [ ] Tests pass: `cargo test --workspace`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo fmt --all --check`
