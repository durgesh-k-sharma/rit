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
        Command::Add { pathspecs } => {
            let repo = repo::Repo::find_repository()?;
            commands::add::cmd_add(&pathspecs, &repo)
        }
        Command::Commit { message } => {
            let repo = repo::Repo::find_repository()?;
            commands::commit::cmd_commit(&message, &repo)
        }
        Command::Status => {
            let repo = repo::Repo::find_repository()?;
            commands::status::cmd_status(&repo)
        }
        Command::Log => {
            let repo = repo::Repo::find_repository()?;
            commands::log::cmd_log(&repo)
        }
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
