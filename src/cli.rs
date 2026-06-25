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

    /// Manage branches
    Branch {
        /// Branch to create or delete
        name: Option<String>,

        /// Delete the branch
        #[arg(short = 'd', long = "delete")]
        delete: bool,
    },

    /// Switch branches or restore working tree files
    Checkout {
        /// Branch name to checkout
        target: String,

        /// Create and switch to a new branch
        #[arg(short = 'b')]
        new_branch: bool,

        /// Optional start point for -b
        start_point: Option<String>,

        /// Discard local changes (no safety check)
        #[arg(short = 'f', long = "force")]
        force: bool,
    },
}
