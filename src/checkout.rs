use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::*;
use crate::repo::Repo;
use crate::index::{Index, IndexEntry};
use crate::object;

struct TreeDiff {
    to_create: Vec<(PathBuf, String, u32)>,
    to_delete: Vec<PathBuf>,
    to_update: Vec<(PathBuf, String, u32)>,
}

pub fn switch_to_branch(
    repo: &Repo,
    branch_name: &str,
    target_sha: &str,
    force: bool,
) -> Result<()> {
    let current_sha = crate::refs::resolve_head(repo)?;

    let current_files = match &current_sha {
        Some(sha) if !sha.is_empty() => {
            let (_otype, content, _) = object::read_object(repo, sha)?;
            let text = String::from_utf8_lossy(&content);
            let tree_sha = text.lines()
                .find_map(|l| l.strip_prefix("tree "))
                .map(|s| s.to_string())
                .ok_or_else(|| RitError::CorruptObject("missing tree".to_string()))?;
            object::flatten_tree(&repo.git_dir, &tree_sha, Path::new(""))?
        }
        _ => BTreeMap::new(),
    };

    let (_type, content, _) = object::read_object_raw(&repo.git_dir, target_sha)?;
    let text = String::from_utf8_lossy(&content);
    let target_tree_sha = text.lines()
        .find_map(|l| l.strip_prefix("tree "))
        .map(|s| s.to_string())
        .ok_or_else(|| RitError::CorruptObject("missing tree in target".to_string()))?;
    let target_files = object::flatten_tree(&repo.git_dir, &target_tree_sha, Path::new(""))?;

    let diff = compute_tree_diff(&current_files, &target_files);

    if !force {
        safety_check(repo, &diff)?;
    }

    apply_diff(repo, &diff)?;
    replace_index(repo, &target_files)?;
    update_head(repo, branch_name)?;

    Ok(())
}

fn compute_tree_diff(
    current: &BTreeMap<PathBuf, (String, u32)>,
    target: &BTreeMap<PathBuf, (String, u32)>,
) -> TreeDiff {
    let mut to_create = Vec::new();
    let mut to_delete = Vec::new();
    let mut to_update = Vec::new();

    let mut all_paths: Vec<&PathBuf> = current.keys().chain(target.keys()).collect();
    all_paths.sort();
    all_paths.dedup();

    for path in all_paths {
        match (current.get(path), target.get(path)) {
            (None, Some((sha, mode))) => to_create.push((path.clone(), sha.clone(), *mode)),
            (Some(_), None) => to_delete.push(path.clone()),
            (Some((old_s, old_m)), Some((new_s, new_m))) if old_s != new_s || old_m != new_m => {
                to_update.push((path.clone(), new_s.clone(), *new_m));
            }
            _ => {}
        }
    }

    TreeDiff { to_create, to_delete, to_update }
}

fn safety_check(repo: &Repo, diff: &TreeDiff) -> Result<()> {
    let index = Index::read(repo)?;
    let mut conflicts: Vec<String> = Vec::new();

    for path in diff.to_update.iter().map(|(p, _, _)| p).chain(diff.to_delete.iter()) {
        let full_path = repo.work_dir.join(path);
        if full_path.exists() {
            let content = fs::read(&full_path)?;
            let disk_sha = object::hash_blob(&content);
            let rel_str = path.to_string_lossy().to_string().replace('\\', "/");
            if let Some(entry) = index.get(&rel_str)
                && disk_sha != entry.sha {
                conflicts.push(rel_str);
            }
        }
    }

    for (path, _, _) in &diff.to_create {
        let full_path = repo.work_dir.join(path);
        let rel_str = path.to_string_lossy().to_string().replace('\\', "/");
        if full_path.exists() && index.get(&rel_str).is_none() {
            conflicts.push(rel_str);
        }
    }

    if !conflicts.is_empty() {
        return Err(RitError::CheckoutConflict(conflicts.join("\n\t")));
    }
    Ok(())
}

fn apply_diff(repo: &Repo, diff: &TreeDiff) -> Result<()> {
    for (path, sha, _mode) in diff.to_create.iter().chain(diff.to_update.iter()) {
        let (_type, content, _) = object::read_object_raw(&repo.git_dir, sha)?;
        let full_path = repo.work_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        atomic_write(&full_path, &content)?;
    }

    for path in &diff.to_delete {
        let full_path = repo.work_dir.join(path);
        if full_path.exists() {
            fs::remove_file(&full_path)?;
        }
        remove_empty_parents(path, &repo.work_dir);
    }

    Ok(())
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let rand_suffix: String = std::iter::repeat(())
        .map(|_| fastrand::alphanumeric())
        .take(8)
        .collect();
    let tmp_path = path.with_extension(format!("rit_tmp_{}", rand_suffix));
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn remove_empty_parents(file_path: &Path, work_dir: &Path) {
    let mut dir = file_path.parent();
    while let Some(d) = dir {
        if d == work_dir {
            break;
        }
        if d.read_dir().map(|mut i| i.next().is_none()).unwrap_or(false) {
            let _ = fs::remove_dir(d);
        } else {
            break;
        }
        dir = d.parent();
    }
}

fn replace_index(repo: &Repo, target_files: &BTreeMap<PathBuf, (String, u32)>) -> Result<()> {
    let mut index = Index::new();

    for (path, (sha, mode)) in target_files {
        let full_path = repo.work_dir.join(path);
        let metadata = fs::metadata(&full_path).or_else(|_| fs::metadata("."))?;

        let relative_str = path.to_string_lossy().replace('\\', "/");
        let epoch = std::time::UNIX_EPOCH;
        let ctime = metadata.created().unwrap_or(std::time::SystemTime::now());
        let mtime = metadata.modified().unwrap_or(std::time::SystemTime::now());
        let ctime_sec = ctime.duration_since(epoch).unwrap_or_default().as_secs() as u32;
        let ctime_nsec = ctime.duration_since(epoch).unwrap_or_default().subsec_nanos();
        let mtime_sec = mtime.duration_since(epoch).unwrap_or_default().as_secs() as u32;
        let mtime_nsec = mtime.duration_since(epoch).unwrap_or_default().subsec_nanos();

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            index.upsert(IndexEntry {
                ctime_sec, ctime_nsec, mtime_sec, mtime_nsec,
                dev: metadata.dev() as u32,
                ino: metadata.ino() as u32,
                mode: *mode,
                uid: metadata.uid(),
                gid: metadata.gid(),
                file_size: metadata.size() as u32,
                sha: sha.clone(),
                path: relative_str,
            });
        }

        #[cfg(not(unix))]
        {
            index.upsert(IndexEntry {
                ctime_sec, ctime_nsec, mtime_sec, mtime_nsec,
                dev: 0, ino: 0,
                mode: *mode,
                uid: 0, gid: 0,
                file_size: metadata.len() as u32,
                sha: sha.clone(),
                path: relative_str,
            });
        }
    }

    index.write(repo)?;
    Ok(())
}

fn update_head(repo: &Repo, branch_name: &str) -> Result<()> {
    fs::write(repo.head_path(), format!("ref: refs/heads/{}\n", branch_name))?;
    Ok(())
}
