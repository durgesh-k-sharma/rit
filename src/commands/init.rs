use std::fs;
use std::path::PathBuf;
use crate::error::*;

pub fn cmd_init(path: Option<PathBuf>) -> Result<()> {
    let root = path.unwrap_or_else(|| std::env::current_dir().unwrap());
    let git_dir = root.join(".git");

    if git_dir.exists() {
        return Err(RitError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("rit repository already exists at {}", git_dir.display()),
        )));
    }

    let dirs = [
        git_dir.join("objects").join("info"),
        git_dir.join("objects").join("pack"),
        git_dir.join("refs").join("heads"),
        git_dir.join("refs").join("tags"),
    ];
    for d in &dirs {
        fs::create_dir_all(d)?;
    }

    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
    fs::write(git_dir.join("config"), "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n")?;
    fs::write(git_dir.join("description"), "Unnamed repository; edit this file to 'name' the repository.\n")?;

    let abs = std::fs::canonicalize(&git_dir)?;
    println!("Initialized empty rit repository in {}", abs.display());
    Ok(())
}
