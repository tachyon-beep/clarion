mod cli;
mod install;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    init_tracing();
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Command::Install { force, path } => install::run(&path, force),
        cli::Command::Analyze { path: _ } => {
            // Task 7 implements this. Stubbed so `clarion analyze` is reachable.
            anyhow::bail!("clarion analyze — unimplemented (landing in Task 7)");
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
