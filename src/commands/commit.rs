use std::collections::HashMap;
use crate::error::*;
use crate::repo::Repo;
use crate::index::Index;
use crate::object::tree::{Tree, TreeEntry};
use crate::object::commit::Commit;
use crate::object::write_object;
use crate::refs;

pub fn cmd_commit(message: &str, repo: &Repo) -> Result<()> {
    let index = Index::read(repo)?;

    if index.is_empty() {
        return Err(RitError::NothingToCommit);
    }

    let tree_sha = build_tree_from_index(&index, repo)?;

    let author_name = get_author_name(repo);
    let author_email = get_author_email(repo);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = now.as_secs() as i64;
    let tz_offset = "+0000";

    let parent = refs::resolve_head(repo)?;

    let commit = Commit::new(
        tree_sha,
        parent.clone(),
        author_name,
        author_email,
        timestamp,
        tz_offset.to_string(),
        message.to_string(),
    );
    let raw = commit.serialize();
    let commit_sha = write_object(repo, &raw)?;

    refs::update_head(repo, &commit_sha)?;

    let short_sha = &commit_sha[..7];
    let branch = get_current_branch(repo);
    if parent.is_some() {
        println!("[{} {}] {}", branch, short_sha, message);
    } else {
        println!("[{} (root-commit) {}] {}", branch, short_sha, message);
    }

    Ok(())
}

fn build_tree_from_index(index: &Index, repo: &Repo) -> Result<String> {
    let mut dir_entries: HashMap<String, Vec<(String, String, String, bool)>> = HashMap::new();

    for entry in index.entries() {
        let path = &entry.path;
        let mode_str = format!("{:o}", entry.mode);
        if let Some((parent_dir, file_name)) = split_parent(path) {
            dir_entries.entry(parent_dir)
                .or_default()
                .push((file_name, entry.sha.clone(), mode_str, false));
        }
    }

    let mut tree_cache: HashMap<String, String> = HashMap::new();

    let mut dirs: Vec<String> = dir_entries.keys().cloned().collect();
    dirs.sort_by(|a, b| {
        let depth_a = a.split('/').count();
        let depth_b = b.split('/').count();
        depth_b.cmp(&depth_a)
    });

    dirs.push(String::new());
    dirs.dedup();

    for dir in &dirs {
        let entries = if dir.is_empty() {
            let mut root_entries: Vec<TreeEntry> = Vec::new();
            for entry in index.entries() {
                if !entry.path.contains('/') {
                    let mode_str = format!("{:o}", entry.mode);
                    root_entries.push(TreeEntry {
                        mode: mode_str,
                        name: entry.path.clone(),
                        sha: entry.sha.clone(),
                        is_tree: false,
                    });
                }
            }
            let subdirs: Vec<String> = tree_cache.keys()
                .filter(|k| !k.contains('/'))
                .cloned()
                .collect();
            for subdir in &subdirs {
                if let Some(sha) = tree_cache.get(subdir) {
                    root_entries.push(TreeEntry {
                        mode: "040000".to_string(),
                        name: subdir.clone(),
                        sha: sha.clone(),
                        is_tree: true,
                    });
                }
            }
            root_entries
        } else {
            let mut tree_entries: Vec<TreeEntry> = Vec::new();
            if let Some(files) = dir_entries.get(dir) {
                for (name, sha, mode, _) in files {
                    tree_entries.push(TreeEntry {
                        mode: mode.clone(),
                        name: name.clone(),
                        sha: sha.clone(),
                        is_tree: false,
                    });
                }
            }
            let prefix = format!("{}/", dir);
            let subdirs: Vec<String> = tree_cache.keys()
                .filter(|k| k.starts_with(&prefix) && !k[prefix.len()..].contains('/'))
                .map(|k| k[prefix.len()..].to_string())
                .collect();
            for subdir_name in &subdirs {
                let full_key = if dir.is_empty() {
                    subdir_name.clone()
                } else {
                    format!("{}/{}", dir, subdir_name)
                };
                if let Some(sha) = tree_cache.get(&full_key) {
                    tree_entries.push(TreeEntry {
                        mode: "040000".to_string(),
                        name: subdir_name.clone(),
                        sha: sha.clone(),
                        is_tree: true,
                    });
                }
            }
            tree_entries
        };

        if entries.is_empty() {
            continue;
        }

        let tree = Tree::from_entries(entries);
        let raw = tree.serialize();
        let sha = write_object(repo, &raw)?;
        tree_cache.insert(dir.clone(), sha);
    }

    tree_cache.get("")
        .cloned()
        .or_else(|| {
            let tree = Tree::from_entries(vec![]);
            write_object(repo, &tree.serialize()).ok()
        })
        .ok_or_else(|| RitError::CorruptObject("failed to build root tree".to_string()))
}

fn split_parent(path: &str) -> Option<(String, String)> {
    if let Some(pos) = path.rfind('/') {
        Some((path[..pos].to_string(), path[pos + 1..].to_string()))
    } else {
        None
    }
}

fn get_author_name(repo: &Repo) -> String {
    if let Ok(val) = std::env::var("GIT_AUTHOR_NAME") {
        return val;
    }
    if let Ok(config) = std::fs::read_to_string(repo.git_dir.join("config")) {
        for line in config.lines() {
            if let Some(val) = line.trim().strip_prefix("name = ") {
                return val.trim_matches('"').to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn get_author_email(repo: &Repo) -> String {
    if let Ok(val) = std::env::var("GIT_AUTHOR_EMAIL") {
        return val;
    }
    if let Ok(config) = std::fs::read_to_string(repo.git_dir.join("config")) {
        for line in config.lines() {
            if let Some(val) = line.trim().strip_prefix("email = ") {
                return val.trim_matches('"').to_string();
            }
        }
    }
    "unknown@localhost".to_string()
}

fn get_current_branch(repo: &Repo) -> String {
    if let Ok(head) = refs::read_head(repo) {
        if let Some(ref_path) = head.strip_prefix("ref: refs/heads/") {
            return ref_path.to_string();
        }
    }
    "HEAD".to_string()
}
