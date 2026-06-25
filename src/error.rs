use thiserror::Error;

pub type Result<T> = std::result::Result<T, RitError>;

#[derive(Error, Debug)]
pub enum RitError {
    #[error("not a rit repository (or any parent up to mount point): {0}")]
    NotARepository(std::path::PathBuf),

    #[error("object not found: {0}")]
    ObjectNotFound(String),

    #[error("corrupt object: {0}")]
    CorruptObject(String),

    #[error("index checksum mismatch — index file is corrupt")]
    CorruptIndex,

    #[error("nothing added to commit but untracked files present")]
    NothingToCommit,

    #[error("your current branch '{0}' does not have any commits yet")]
    NoCommits(String),

    #[error("ambiguous object prefix '{0}' matches multiple objects")]
    AmbiguousPrefix(String),

    #[error("invalid object name '{0}'")]
    InvalidObjectName(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
