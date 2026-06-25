mod cli;
mod error;
mod repo;
mod object;
mod refs;
mod index;
mod commands;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Init { path } => commands::init::cmd_init(path),
        _ => {
            let _repo = repo::Repo::find_repository()?;
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
    Ok(())
}
