use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// rit — a minimal Git implementation
#[derive(Parser)]
#[command(name = "rit", author, version = env!("CARGO_PKG_VERSION"), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new repository
    Init {
        /// Where to create the repository (default: current directory)
        path: Option<PathBuf>,
    },
    /// Add file contents to the index
    Add {
        #[arg(required = true)]
        pathspecs: Vec<PathBuf>,
    },
    /// Record changes to the repository
    Commit {
        #[arg(short, long, required = true)]
        message: String,
    },
    /// Show the working tree status
    Status,
    /// Show commit logs
    Log,
    /// Provide content or type information for repository objects
    #[command(name = "cat-file")]
    CatFile {
        #[arg(short = 't', conflicts_with_all = ["pretty_print", "size"])]
        type_only: bool,
        #[arg(short = 'p', conflicts_with_all = ["type_only", "size"])]
        pretty_print: bool,
        #[arg(short = 's', conflicts_with_all = ["type_only", "pretty_print"])]
        size: bool,
        object: String,
    },
}
