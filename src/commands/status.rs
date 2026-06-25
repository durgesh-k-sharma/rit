use std::collections::HashMap;
use std::fs;
use crate::error::*;
use crate::repo::Repo;
use crate::index::Index;
use crate::object::read_object;
use crate::refs;
use crate::commands::add::is_ignored;

#[derive(Debug, PartialEq)]
enum StatusChange {
    NewFile,
    Modified,
    Deleted,
}

pub fn cmd_status(repo: &Repo) -> Result<()> {
    let branch = get_current_branch(repo);
    println!("On branch {}", branch);

    let head_entries = get_head_tree_entries(repo)?;
    let index = Index::read(repo)?;

    let mut staged: Vec<(String, StatusChange)> = Vec::new();
    for entry in index.entries() {
        if let Some(head_sha) = head_entries.get(&entry.path) {
            if head_sha != &entry.sha {
                staged.push((entry.path.clone(), StatusChange::Modified));
            }
        } else {
            staged.push((entry.path.clone(), StatusChange::NewFile));
        }
    }
    for (path, _sha) in &head_entries {
        if index.get(path).is_none() {
            staged.push((path.clone(), StatusChange::Deleted));
        }
    }

    let mut unstaged: Vec<(String, StatusChange)> = Vec::new();
    for entry in index.entries() {
        let work_path = repo.work_dir.join(&entry.path);
        if !work_path.exists() {
            unstaged.push((entry.path.clone(), StatusChange::Deleted));
        } else if let Ok(content) = fs::read(&work_path) {
            let header = format!("blob {}\0", content.len());
            let mut sha1 = sha1_smol::Sha1::new();
            sha1.update(header.as_bytes());
            sha1.update(&content);
            let disk_sha = hex::encode(sha1.digest().bytes());
            if disk_sha != entry.sha {
                unstaged.push((entry.path.clone(), StatusChange::Modified));
            }
        }
    }

    let untracked = find_untracked_files(repo, &index);

    if !staged.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"rit reset HEAD <file>...\" to unstage)");
        println!();
        for (path, change) in &staged {
            match change {
                StatusChange::NewFile => println!("\tnew file:   {}", path),
                StatusChange::Modified => println!("\tmodified:   {}", path),
                StatusChange::Deleted => println!("\tdeleted:    {}", path),
            }
        }
        println!();
    }

    if !unstaged.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"rit add <file>...\" to update what will be committed)");
        println!();
        for (path, change) in &unstaged {
            match change {
                StatusChange::Modified => println!("\tmodified:   {}", path),
                StatusChange::Deleted => println!("\tdeleted:    {}", path),
                _ => {}
            }
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked files:");
        println!("  (use \"rit add <file>...\" to include in what will be committed)");
        println!();
        for path in &untracked {
            println!("\t{}", path);
        }
        println!();
    }

    Ok(())
}

fn get_head_tree_entries(repo: &Repo) -> Result<HashMap<String, String>> {
    let head_sha = refs::resolve_head(repo)?;
    let head_sha = match head_sha {
        Some(sha) => sha,
        None => return Ok(HashMap::new()),
    };

    let (_type, commit_content, _sha) = read_object(repo, &head_sha)?;
    let commit_text = String::from_utf8_lossy(&commit_content);
    let tree_sha = commit_text.lines()
        .find_map(|line| line.strip_prefix("tree "))
        .map(|s| s.to_string())
        .ok_or_else(|| RitError::CorruptObject("missing tree in commit".to_string()))?;

    read_tree_entries(repo, &tree_sha, "")
}

fn read_tree_entries(repo: &Repo, tree_sha: &str, prefix: &str) -> Result<HashMap<String, String>> {
    let (_type, content, _sha) = read_object(repo, tree_sha)?;
    let tree = crate::object::tree::Tree::parse(&content);
    let mut entries = HashMap::new();

    for entry in &tree.entries {
        let full_path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        if entry.is_tree {
            let sub_entries = read_tree_entries(repo, &entry.sha, &full_path)?;
            entries.extend(sub_entries);
        } else {
            entries.insert(full_path, entry.sha.clone());
        }
    }

    Ok(entries)
}

fn find_untracked_files(repo: &Repo, index: &Index) -> Vec<String> {
    let mut untracked = Vec::new();
    walk_for_untracked(repo, &repo.work_dir, "", index, &mut untracked);
    untracked.sort();
    untracked
}

fn walk_for_untracked(repo: &Repo, dir: &std::path::Path, prefix: &str, index: &Index, untracked: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_ignored(&path, repo) {
                continue;
            }
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            let relative = if prefix.is_empty() {
                name_str.clone()
            } else {
                format!("{}/{}", prefix, name_str)
            };

            if path.is_dir() {
                if index.get(&relative).is_none() {
                    walk_for_untracked(repo, &path, &relative, index, untracked);
                }
            } else if index.get(&relative).is_none() {
                untracked.push(relative);
            }
        }
    }
}

fn get_current_branch(repo: &Repo) -> String {
    if let Ok(head) = refs::read_head(repo) {
        if let Some(ref_path) = head.strip_prefix("ref: refs/heads/") {
            return ref_path.to_string();
        }
    }
    "HEAD".to_string()
}
