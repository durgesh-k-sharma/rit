use std::fs;
use std::path::Path;
use std::time::SystemTime;
use crate::error::*;
use crate::repo::Repo;
use crate::index::{Index, IndexEntry};
use crate::object::blob::Blob;
use crate::object::write_object;

fn is_ignored(path: &Path, repo: &Repo) -> bool {
    if path.components().any(|c| c.as_os_str() == ".git") {
        return true;
    }
    let gitignore_path = repo.work_dir.join(".gitignore");
    if gitignore_path.exists() {
        if let Ok(content) = fs::read_to_string(&gitignore_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.starts_with('!') {
                    continue;
                }
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if wildmatch(line, &file_name) {
                    return true;
                }
            }
        }
    }
    false
}

fn wildmatch(pattern: &str, name: &str) -> bool {
    if pattern == name {
        return true;
    }
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with("**/*") {
        let suffix = &pattern[4..];
        return name.ends_with(suffix);
    }
    if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        return name.ends_with(suffix);
    }
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }
    false
}

pub fn cmd_add(pathspecs: &[std::path::PathBuf], repo: &Repo) -> Result<()> {
    let mut index = Index::read(repo)?;

    for pathspec in pathspecs {
        let full_path = if pathspec.is_absolute() {
            pathspec.clone()
        } else {
            repo.work_dir.join(pathspec)
        };
        if !full_path.exists() {
            return Err(RitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("path '{}' does not exist", pathspec.display()),
            )));
        }
        if full_path.is_dir() {
            add_directory(&full_path, repo, &mut index)?;
        } else {
            add_file(&full_path, repo, &mut index)?;
        }
    }

    index.write(repo)?;
    Ok(())
}

fn add_directory(dir: &Path, repo: &Repo, index: &mut Index) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if is_ignored(&path, repo) {
            continue;
        }
        if path.is_dir() {
            add_directory(&path, repo, index)?;
        } else {
            add_file(&path, repo, index)?;
        }
    }
    Ok(())
}

fn add_file(path: &Path, repo: &Repo, index: &mut Index) -> Result<()> {
    if is_ignored(path, repo) {
        return Ok(());
    }
    let content = fs::read(path)?;
    let blob = Blob::from_content(content);
    let raw = blob.serialize();
    let sha = write_object(repo, &raw)?;

    let metadata = fs::metadata(path)?;
    let relative = path.strip_prefix(&repo.work_dir)
        .unwrap_or(path);
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    let is_exec = metadata.permissions().readonly() == false
        && cfg!(unix)
        && {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                false
            }
        };
    let mode: u32 = if is_exec { 0o100755 } else { 0o100644 };

    let epoch = std::time::UNIX_EPOCH;
    let ctime = metadata.created().unwrap_or(SystemTime::now());
    let mtime = metadata.modified().unwrap_or(SystemTime::now());
    let ctime_sec = ctime.duration_since(epoch).unwrap_or_default().as_secs() as u32;
    let ctime_nsec = ctime.duration_since(epoch).unwrap_or_default().subsec_nanos();
    let mtime_sec = mtime.duration_since(epoch).unwrap_or_default().as_secs() as u32;
    let mtime_nsec = mtime.duration_since(epoch).unwrap_or_default().subsec_nanos();

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let dev = metadata.dev() as u32;
        let ino = metadata.ino() as u32;
        let uid = metadata.uid();
        let gid = metadata.gid();
        let file_size = metadata.size() as u32;

        index.upsert(IndexEntry {
            ctime_sec, ctime_nsec, mtime_sec, mtime_nsec,
            dev, ino, mode, uid, gid, file_size,
            sha,
            path: relative_str,
        });
    }

    #[cfg(not(unix))]
    {
        index.upsert(IndexEntry {
            ctime_sec, ctime_nsec, mtime_sec, mtime_nsec,
            dev: 0, ino: 0, mode, uid: 0, gid: 0, file_size: metadata.len() as u32,
            sha,
            path: relative_str,
        });
    }

    Ok(())
}
