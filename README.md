# Oxidoc

A next-generation documentation engine written entirely in Rust. Zero Node.js dependency. Sub-second builds. Interactive WebAssembly islands.

## Features

### Authoring & Content

- **RDX Format** — Author in Rust Document eXpressions: Markdown with embedded reactive components
- **Built-in Components** — Callout, Tabs, CodeBlock, CardGrid, Accordion, Steps, Badge, ThemedImage with full ARIA keyboard navigation
- **Custom Web Components** — Vanilla JS escape hatch for rapid UI iteration via `[components.custom]`
- **OpenAPI Native** — First-class `openapi.yaml`/`json` ingestion with auto-generated API reference pages

### Performance & Build

- **Static Site Generation** — Fast parallel builds with sub-second incremental rebuilds
- **Live Dev Server** — `oxidoc dev` with file watching and hot reload — edit `.rdx` or `oxidoc.toml`, browser refreshes instantly
- **Islands Architecture** — Partial hydration with three-binary Wasm code-splitting (registry, API playground, search)
- **Incremental Builds** — Hash-based change detection rebuilds only modified pages
- **Parallel Rendering** — Rayon-powered multi-core page processing
- **Asset Pipeline** — Content-hashed filenames, CSS minification via LightningCSS, HTML minification

### Search

- **Built-in Lexical Search** — BM25 scoring with tokenization, fast to build and query
- **Optional Semantic Search** — Enable `semantic = true` for hybrid BM25 + embedding search with RRF fusion (adds build time due to embedding generation)
- **Pluggable Providers** — Algolia, Typesense, Meilisearch, or custom script injection via `[search]` config

### Styling

- **CSS Variables** — Semantic design tokens for colors, fonts, spacing, and radii
- **CSS @layer** — Base styles scoped to `@layer oxidoc` for clean override semantics
- **Animations** — Entrance animations on landing pages, search dialog, and interactive components
- **Dual Light/Dark** — Light and dark palettes; toggle via system preference or manual switch
- **Customizable** — Override primary/accent colors and fonts via `[theme]` config; layer custom CSS on top

### Enterprise & SEO

- **OpenAPI Playground** — Interactive request builder with auth, code generation (curl/Python/JS/Rust), Shadow DOM isolation
- **SEO** — Open Graph, JSON-LD, canonical URLs, sitemap.xml, robots.txt
- **RSS/Atom Feed** — Auto-generated Atom feed from all documentation pages
- **Versioning** — Multi-version output (`/v1.0/`, `/v2.0/`) with version switcher
- **Security** — SRI hashes, content-hashed assets
- **Analytics** — Google Analytics or custom script injection
- **Redirects** — URL migration via `[redirects]` config
- **LLM-ready** — Auto-generated `llms.txt` and `llms-full.txt` for RAG pipelines

### Developer Experience

- **`oxidoc init`** — Scaffold a new project in seconds
- **Error Diagnostics** — miette-powered errors with source spans, file paths, and suggestions
- **Config Validation** — Unknown key detection with did-you-mean suggestions

### Accessibility

- **WCAG 2.1** — All components meet AA contrast, ARIA roles, keyboard navigation
- **Shadow DOM Bridging** — Focus trapping and ARIA sync across shadow boundaries

## Quick Start

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/oxidoc-lab/oxidoc/main/install.sh | sh
```

Or download a binary directly from [GitHub Releases](https://github.com/oxidoc-lab/oxidoc/releases).

### Create a Project

```bash
oxidoc init my-docs
cd my-docs
```

### Develop

```bash
oxidoc dev
```

Starts a local server at `http://localhost:3000` with hot reload — edit any `.rdx` file or `oxidoc.toml` and the browser refreshes automatically.

### Build for Production

```bash
oxidoc build
```

Outputs a static `dist/` directory ready to deploy to GitHub Pages, Vercel, Netlify, Cloudflare Pages, or any static host.

## Project Structure

```
my-docs/
├── oxidoc.toml              # Site configuration
├── docs/                    # Your .rdx content files
│   ├── intro.rdx
│   ├── quickstart.rdx
│   └── guides/
│       ├── installation.rdx
│       └── deployment.rdx
├── assets/                  # Static files (images, fonts, etc.)
│   └── logo.svg
└── openapi.yaml             # API spec (optional)
```

### Routing

By default, Oxidoc discovers all `.rdx` files in `docs/` and orders them alphabetically. Use numeric prefixes to control order:

```
docs/
├── 01-intro.rdx        → /intro
├── 02-quickstart.rdx   → /quickstart
└── 03-advanced.rdx     → /advanced
```

For explicit control, define navigation groups in `oxidoc.toml`:

```toml
[routing]
navigation = [
  { group = "Getting Started", pages = ["intro", "quickstart"] },
  { group = "Guides", pages = ["guides/installation", "guides/deployment"] },
  { group = "API Reference", openapi = "./openapi.yaml" }
]
```

Each entry in `pages` maps to a file in `docs/` — `"intro"` resolves to `docs/intro.rdx`. Groups with `openapi` auto-generate API reference pages from the spec.

## Configuration

```toml
[project]
name = "My Documentation"
description = "Beautiful API docs"
logo = "/assets/logo.svg"
base_url = "https://docs.example.com"

[theme]
primary = "#2563eb"                # optional color override
dark_mode = "system"               # "system", "light", "dark"
custom_css = ["assets/custom.css"] # optional, layered on top

[routing]
navigation = [
  { group = "Getting Started", pages = ["intro", "quickstart"] },
  { group = "API Reference", openapi = "./openapi.yaml" }
]

[search]
provider = "oxidoc"
```

See [docs/configuration.md](docs/configuration.md) for the full reference.

## Workspace

| Crate               | Type    | Purpose                                                   |
| :------------------ | :------ | :-------------------------------------------------------- |
| `oxidoc-cli`        | Binary  | CLI commands: `dev`, `build`, `init`                      |
| `oxidoc-core`       | Library | Build engine: config, parsing, rendering, search indexing |
| `oxidoc-island`     | Library | `OxidocIsland` trait definition                           |
| `oxidoc-components` | Library | Built-in Leptos components                                |
| `oxidoc-registry`   | cdylib  | Wasm entry point: DOM scanning + hydration                |
| `oxidoc-openapi`    | cdylib  | Wasm: API playground island                               |
| `oxidoc-search`     | cdylib  | Wasm: search island (lexical + optional semantic)         |
| `oxidoc-text`       | Library | Shared tokenization pipeline for search                   |

## Development Setup

After cloning the repository, download the embedding model used for semantic search:

```bash
mkdir -p oxidoc-cli/assets/models
curl -L -o oxidoc-cli/assets/models/bge-micro-v2.gguf \
  https://huggingface.co/fs90/bge-micro-v2-i1-GGUF/resolve/main/bge-micro-v2.i1-Q4_K_M.gguf
```

This ~17MB GGUF model is embedded into the binary at compile time via `include_bytes!()` and is required for `cargo build` to succeed. Pre-built binaries from [GitHub Releases](https://github.com/oxidoc-lab/oxidoc/releases) include it already.

## Architecture

```
        .rdx files + oxidoc.toml
                   │
                   ▼
┌──────────────────────────────────────┐
│           oxidoc-core                │
│  Config → Parse → Render → dist/    │
│  Static HTML + <oxidoc-island> tags  │
└──────────────────┬───────────────────┘
                   │
        Browser loads HTML
                   │
┌──────────────────▼───────────────────┐
│         oxidoc-registry.wasm         │
│  Scans DOM → hydrates Leptos islands │
│  Lazy-loads openapi/search Wasm      │
└──────────────────────────────────────┘
```

## RDX Language

RDX (Rust Document eXpressions) is a standalone language project. See [github.com/rdx-lang/rdx](https://github.com/rdx-lang/rdx).

## Documentation

- [Configuration Reference](docs/configuration.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Guide](docs/security.md)

## License

[FSL-1.1-ALv2](LICENSE.md) — Functional Source License, Version 1.1, with Apache License 2.0 future grant.

You're free to use oxidoc for internal use, public-facing documentation, education, and research. The license restricts offering oxidoc itself as a competing commercial product or managed service.

For commercial licensing inquiries, contact [@farhan-syah](https://github.com/farhan-syah).
