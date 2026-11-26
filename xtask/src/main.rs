#![expect(clippy::print_stdout, reason = "xtask is a CLI build orchestrator using stdout")]

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod util;

use commands::codegen_spec;
use util::find_workspace_root;

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Matter specification code generator & workspace orchestrator"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Autogenerate strongly-typed Matter cluster domain modules from XML specifications.
    CodegenSpec {
        /// Git repository URL (defaults to upstream connectedhomeip).
        #[arg(long, default_value = "https://github.com/project-chip/connectedhomeip.git")]
        repo: String,
        /// Subfolder path inside repository containing ZCL cluster XML definitions.
        #[arg(long, default_value = "src/app/zap-templates/zcl/data-model/chip")]
        path: String,
        /// Query upstream repository for the latest Matter specification versions, download missing XMLs, prune older
        /// cached family versions, and generate code (requires network access).
        #[arg(long, conflicts_with = "pin")]
        fetch: bool,
        /// Fetch and pin one or more specific 4-component Matter specification versions from upstream (e.g. "1.0.0.0"
        /// or "v1.0.0.0"), replacing the existing cached version for that major.minor family before generating code
        /// (requires network access).
        #[arg(
            short = 'p',
            long = "pin",
            value_name = "VERSION",
            value_delimiter = ',',
            conflicts_with_all = ["fetch", "list"]
        )]
        pin: Vec<String>,
        /// Force re-downloading XML specifications from upstream even if already cached locally (only valid with
        /// --fetch or --pin).
        #[arg(long)]
        force: bool,
        /// List locally cached Matter specification versions (or combine with --fetch to query and list remote
        /// upstream versions) without generating code.
        #[arg(long, conflicts_with = "force")]
        list: bool,
        /// Output results as machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = find_workspace_root()?;

    match cli.command {
        Commands::CodegenSpec {
            repo,
            path,
            fetch,
            pin,
            force,
            list,
            json,
        } => {
            let options = codegen_spec::CodegenSpecOptions {
                repo,
                path,
                fetch,
                pin,
                force,
                list,
                json,
            };
            codegen_spec::run_codegen_spec(&root, &options)?;
        }
    }

    Ok(())
}
