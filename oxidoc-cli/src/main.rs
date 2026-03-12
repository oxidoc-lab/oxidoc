mod server;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "oxidoc",
    about = "Blazing-fast documentation engine powered by Rust and WebAssembly"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Project root directory (defaults to current directory)
    #[arg(short = 'C', long, global = true)]
    project: Option<PathBuf>,

    /// Enable verbose output (detailed build steps)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all output except errors (for CI)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Build the documentation site for production (SSG)
    Build {
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: String,
    },
    /// Start the development server with hot module replacement
    Dev {
        /// Port to serve on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Initialize a new Oxidoc project
    Init {
        /// Project directory name (defaults to current directory)
        name: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let log_level = if cli.quiet {
        tracing::Level::ERROR
    } else if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    let project_root = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    let result = match cli.command {
        Command::Build { output } => run_build(&project_root, &output),
        Command::Dev { port } => run_dev(&project_root, port),
        Command::Init { name } => {
            let target = match name {
                Some(ref n) => project_root.join(n),
                None => project_root.clone(),
            };
            run_init(&target, name.as_deref())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = e
                .downcast_ref::<oxidoc_core::error::OxidocError>()
                .is_some_and(|oe| oe.is_config_error());
            eprintln!("{e:?}");
            ExitCode::from(if code { 2 } else { 1 })
        }
    }
}

fn run_build(project_root: &std::path::Path, output: &str) -> miette::Result<()> {
    let output_dir = project_root.join(output);
    tracing::info!("Building site to {}/", output_dir.display());

    let start = std::time::Instant::now();
    let result = oxidoc_core::builder::build_site(project_root, &output_dir)?;
    tracing::info!(
        pages = result.pages_rendered,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "Build complete"
    );
    Ok(())
}

fn run_dev(project_root: &std::path::Path, port: u16) -> miette::Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| miette::miette!("Failed to create async runtime: {e}"))?;
    rt.block_on(server::run_dev_server(project_root.to_path_buf(), port))
}

fn run_init(target: &std::path::Path, name: Option<&str>) -> miette::Result<()> {
    std::fs::create_dir_all(target)
        .map_err(|e| miette::miette!("Failed to create project directory: {e}"))?;

    let config_path = target.join("oxidoc.toml");
    if config_path.exists() {
        miette::bail!("oxidoc.toml already exists in {}", target.display());
    }

    let project_name = name
        .map(|n| {
            std::path::Path::new(n)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(n)
        })
        .unwrap_or("My Documentation");

    // Create directories
    let docs_dir = target.join("docs");
    let assets_dir = target.join("assets");
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| miette::miette!("Failed to create docs/: {e}"))?;
    std::fs::create_dir_all(&assets_dir)
        .map_err(|e| miette::miette!("Failed to create assets/: {e}"))?;

    // Write default logo
    std::fs::write(
        assets_dir.join("logo.svg"),
        include_str!("../assets/logo.svg"),
    )
    .map_err(|e| miette::miette!("Failed to write assets/logo.svg: {e}"))?;

    // Write .gitignore
    std::fs::write(
        target.join(".gitignore"),
        "# Oxidoc build artifacts\n.oxidoc-dev/\ndist/\n",
    )
    .map_err(|e| miette::miette!("Failed to write .gitignore: {e}"))?;

    // Write oxidoc.toml
    std::fs::write(
        &config_path,
        format!(
            r##"[project]
name = "{project_name}"
# description = "Your project description"
logo = "/assets/logo.svg"
# base_url = "https://docs.example.com"

[theme]
primary = "#3b82f6"
dark_mode = "system"

[routing]
homepage = "intro"
navigation = [
  {{ group = "Getting Started", pages = ["intro", "quickstart"] }},
]

# [search]
# provider = "oxidoc"

# [footer]
# copyright = "© 2024 {project_name}"
"##
        ),
    )
    .map_err(|e| miette::miette!("Failed to write oxidoc.toml: {e}"))?;

    // Write intro.rdx
    std::fs::write(
        docs_dir.join("intro.rdx"),
        r#"# Welcome

Welcome to your new documentation site, powered by **Oxidoc**.

<Callout kind="info" title="What is Oxidoc?">
Oxidoc is a documentation engine written in Rust. It generates fast static sites with interactive WebAssembly components.
</Callout>

## Features

<CardGrid>
<Card title="Fast Builds" href="/quickstart">
Sub-second builds for thousands of pages with incremental rebuilds.
</Card>
<Card title="Interactive Islands" href="/quickstart">
Wasm-powered components like tabs, code blocks, and API playgrounds.
</Card>
<Card title="Search Built-in" href="/quickstart">
Hybrid lexical + semantic search with zero external dependencies.
</Card>
</CardGrid>

## Next Steps

Check out the [Quickstart Guide](/quickstart) to create your first page.
"#,
    )
    .map_err(|e| miette::miette!("Failed to write intro.rdx: {e}"))?;

    // Write quickstart.rdx
    std::fs::write(
        docs_dir.join("quickstart.rdx"),
        r#"# Quickstart

## Writing Content

Create `.rdx` files in the `docs/` directory. RDX is Markdown with embedded components:

<Tabs>
<Tab title="Markdown">
```markdown
# My Page

Regular Markdown works as expected — headings, **bold**, *italic*,
links, lists, code blocks, and more.
```
</Tab>
<Tab title="With Components">
```markdown
# My Page

<Callout kind="warning" title="Heads up">
Components are embedded directly in your Markdown content.
</Callout>
```
</Tab>
</Tabs>

## Available Components

<Accordion title="Callout">
Display info, warnings, errors, or tips:

```markdown
<Callout kind="info" title="Note">
Your message here.
</Callout>
```

Variants: `info`, `warning`, `error`, `tip`
</Accordion>

<Accordion title="Tabs">
Group content into switchable tabs:

```markdown
<Tabs>
<Tab title="JavaScript">
console.log("hello");
</Tab>
<Tab title="Python">
print("hello")
</Tab>
</Tabs>
```
</Accordion>

<Accordion title="CodeBlock">
Code with line numbers, highlighting, and copy button:

```markdown
<CodeBlock language="rust" filename="main.rs" highlight="2">
fn main() {
    println!("Hello, Oxidoc!");
}
</CodeBlock>
```
</Accordion>

<Accordion title="CardGrid">
Responsive grid of linked cards:

```markdown
<CardGrid>
<Card title="First" href="/page-1">Description here.</Card>
<Card title="Second" href="/page-2">Another card.</Card>
</CardGrid>
```
</Accordion>

## Development Server

Run `oxidoc dev` to start a local server with hot reload:

<CodeBlock language="bash">
oxidoc dev
</CodeBlock>

Edit any `.rdx` file and the browser refreshes automatically.

## Building for Production

<CodeBlock language="bash">
oxidoc build
</CodeBlock>

Your site is generated in `dist/` — deploy it anywhere.
"#,
    )
    .map_err(|e| miette::miette!("Failed to write quickstart.rdx: {e}"))?;

    if let Some(n) = name {
        eprintln!("Initialized Oxidoc project in {n}/");
    } else {
        eprintln!("Initialized Oxidoc project");
    }
    eprintln!();
    eprintln!("  oxidoc.toml          — site configuration");
    eprintln!("  assets/logo.svg      — default logo (replace with your own)");
    eprintln!("  docs/intro.rdx       — landing page");
    eprintln!("  docs/quickstart.rdx  — getting started guide");
    eprintln!();
    if let Some(n) = name {
        eprintln!("Get started:");
        eprintln!("  cd {n} && oxidoc dev");
    } else {
        eprintln!("Get started:");
        eprintln!("  oxidoc dev");
    }

    Ok(())
}
