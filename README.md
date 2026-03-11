# Oxidoc

A next-generation documentation engine written entirely in Rust. Generate fast static documentation sites with interactive WebAssembly islands, powered by Leptos.

## Features

- **Static Site Generation (SSG)**: Sub-second builds for 1,000+ page sites with zero Node.js dependency
- **Islands Architecture**: Partial hydration with multi-binary Wasm code-splitting for minimal client payload
- **RDX Format**: Author in Rust Document eXpressions — embed reactive components directly in Markdown
- **OpenAPI Native**: First-class support for interactive API playgrounds with Shadow DOM isolation
- **WebAssembly Components**: Responsive Leptos-based islands (Callout, Tabs, CodeBlock, Accordion, and more)
- **Accessibility (a11y)**: Enterprise-grade ARIA support and focus management across Shadow DOM boundaries
- **Incremental Builds**: Smart caching system detects changed content and only rebuilds what's necessary

## Quick Start

### Installation

```bash
cargo install oxidoc-cli
```

### Create a New Documentation Site

```bash
oxidoc init my-docs
cd my-docs
oxidoc dev
```

The dev server runs at `http://localhost:3000` with Hot Module Reload (HMR).

### Build for Production

```bash
oxidoc build
```

Output is a static `/dist` directory ready to deploy to any static host.

## Project Structure

Oxidoc is organized as a Cargo workspace with focused crates:

| Crate | Purpose |
|:---|:---|
| `oxidoc-cli` | Binary: `oxidoc dev` and `oxidoc build` commands |
| `oxidoc-core` | Build engine: config, RDX parsing, HTML generation |
| `oxidoc-island` | Library: island component trait definition |
| `oxidoc-components` | Built-in Leptos components (Callout, Tabs, etc.) |
| `oxidoc-registry` | Wasm: DOM scanning and hydration entry point |

## Configuration

Create an `oxidoc.toml` file in your project root:

```toml
[project]
name = "My Documentation"
description = "Beautiful API docs"
logo = "/assets/logo.svg"
base_url = "https://docs.example.com"

[theme]
primary = "#2563eb"
dark_mode = "system"

[routing]
navigation = [
  { group = "Getting Started", pages = ["intro", "quickstart"] },
  { group = "API Reference", openapi = "./openapi.yaml" }
]

[search]
provider = "oxidoc-boostr"
```

For a complete configuration reference, see [docs/configuration.md](docs/configuration.md).

## Build Commands

```bash
# Development server with HMR
oxidoc dev

# Production build
oxidoc build

# Help
oxidoc --help
```

## Documentation

- [Configuration Reference](docs/configuration.md) — All config options explained
- [Contributing Guide](CONTRIBUTING.md) — Development setup and code conventions
- [Security Guide](docs/security.md) — Content-Security-Policy and Wasm security

## Architecture

Oxidoc operates across two environments:

**Build Engine (Host):**
- Reads `oxidoc.toml` and `.rdx` files
- Parses with the standalone `rdx-parser` crate
- Generates static HTML with `<oxidoc-island>` placeholders
- Ships pre-compiled Wasm binaries (no cargo build during user builds)

**Frontend Runtime (Browser):**
- `oxidoc-registry.wasm` scans DOM and hydrates components
- Island components load on-demand via Leptos
- Shadow DOM isolates complex components (like ApiPlayground)

## RDX Language

RDX (Rust Document eXpressions) is a separate language project. Learn more at [github.com/rdx-lang/rdx](https://github.com/rdx-lang/rdx).

## License

Licensed under the Apache License, Version 2.0. See LICENSE file for details.
