use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    Init,
}

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let project_root = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    match cli.command {
        Command::Build { output } => {
            let output_dir = project_root.join(&output);
            tracing::info!("Building site to {}/", output_dir.display());

            let result = oxidoc_core::builder::build_site(&project_root, &output_dir)?;
            tracing::info!(
                pages = result.pages_rendered,
                output = %result.output_dir,
                "Build complete"
            );
        }
        Command::Dev { port } => {
            tracing::info!("Starting dev server on port {port}");
            // TODO: spin up axum dev server with HMR via notify
        }
        Command::Init => {
            init_project(&project_root)?;
        }
    }

    Ok(())
}

fn init_project(project_root: &std::path::Path) -> miette::Result<()> {
    let config_path = project_root.join("oxidoc.toml");
    if config_path.exists() {
        miette::bail!("oxidoc.toml already exists in this directory");
    }

    let docs_dir = project_root.join("docs");
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| miette::miette!("Failed to create docs/ directory: {e}"))?;

    std::fs::write(
        &config_path,
        r##"[project]
name = "My Documentation"

[theme]
primary = "#3b82f6"
dark_mode = "system"
"##,
    )
    .map_err(|e| miette::miette!("Failed to write oxidoc.toml: {e}"))?;

    std::fs::write(
        docs_dir.join("intro.rdx"),
        r#"# Welcome

Welcome to your new documentation site, powered by **Oxidoc**.

## Getting Started

Edit this file at `docs/intro.rdx` to get started.

<Callout type="info">
This is an interactive island component. It will be hydrated by WebAssembly in the browser.
</Callout>
"#,
    )
    .map_err(|e| miette::miette!("Failed to write intro.rdx: {e}"))?;

    tracing::info!("Initialized new Oxidoc project");
    tracing::info!("  Created oxidoc.toml");
    tracing::info!("  Created docs/intro.rdx");
    tracing::info!("  Run `oxidoc build` to generate your site");

    Ok(())
}
