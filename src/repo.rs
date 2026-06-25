use std::path::{Path, PathBuf};
use crate::error::*;

pub struct Repo {
    pub git_dir: PathBuf,
    pub work_dir: PathBuf,
}

impl Repo {
    pub fn new(git_dir: PathBuf, work_dir: PathBuf) -> Self {
        Repo { git_dir, work_dir }
    }

    pub fn find_repository() -> Result<Repo> {
        let cwd = std::env::current_dir()?;
        let mut dir = Some(&cwd as &Path);
        while let Some(d) = dir {
            let candidate = d.join(".git");
            if candidate.is_dir() {
                return Ok(Repo::new(candidate.canonicalize()?, d.to_path_buf()));
            }
            dir = d.parent();
        }
        Err(RitError::NotARepository(cwd))
    }

    pub fn objects_path(&self) -> PathBuf {
        self.git_dir.join("objects")
    }

    #[allow(dead_code)]
    pub fn refs_path(&self) -> PathBuf {
        self.git_dir.join("refs")
    }

    pub fn head_path(&self) -> PathBuf {
        self.git_dir.join("HEAD")
    }
}
