use thiserror::Error;

pub type Result<T> = std::result::Result<T, RitError>;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum RitError {
    #[error("not a rit repository (or any parent up to mount point): {0}")]
    NotARepository(std::path::PathBuf),

    #[error("object not found: {0}")]
    ObjectNotFound(String),

    #[error("corrupt object: {0}")]
    CorruptObject(String),

    #[error("index checksum mismatch — index file is corrupt")]
    CorruptIndex,

    #[error("nothing to commit")]
    NothingToCommit,

    #[error("your current branch '{0}' does not have any commits yet")]
    NoCommits(String),

    #[error("ambiguous object prefix '{0}' matches multiple objects")]
    AmbiguousPrefix(String),

    #[error("invalid object name '{0}'")]
    InvalidObjectName(String),

    #[error("branch '{0}' not found")]
    BranchNotFound(String),

    #[error("a branch named '{0}' already exists")]
    BranchAlreadyExists(String),

    #[error("cannot delete branch '{0}' checked out at '{1}'")]
    BranchCheckedOut(String, String),

    #[error("pathspec '{0}' did not match any file(s) known to rit")]
    UnknownRef(String),

    #[error("your local changes to the following files would be overwritten by checkout:\n{0}\nPlease commit your changes or stash them before you switch branches.\nAborting")]
    CheckoutConflict(String),

    #[error("detached HEAD checkout not yet implemented")]
    DetachedHead,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
