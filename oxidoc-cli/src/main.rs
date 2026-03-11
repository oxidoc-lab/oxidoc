use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "oxidoc",
    about = "Blazing-fast documentation engine powered by Rust and WebAssembly"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the development server with hot module replacement
    Dev {
        /// Port to serve on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Build the documentation site for production (SSG)
    Build {
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: String,
    },
}

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Dev { port } => {
            tracing::info!("Starting dev server on port {port}");
            // TODO: spin up axum dev server with HMR via notify
        }
        Command::Build { output } => {
            tracing::info!("Building site to {output}/");
            // TODO: run oxidoc-core build pipeline
        }
    }

    Ok(())
}
