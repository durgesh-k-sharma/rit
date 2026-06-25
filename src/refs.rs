use crate::error::*;
use crate::repo::Repo;
use std::fs;
use std::path::Path;

pub fn read_head(repo: &Repo) -> Result<String> {
    let content = fs::read_to_string(repo.head_path())?;
    Ok(content.trim().to_string())
}

pub fn resolve_head(repo: &Repo) -> Result<Option<String>> {
    let head = read_head(repo)?;
    if let Some(ref_path) = head.strip_prefix("ref: ") {
        let ref_file = repo.git_dir.join(ref_path);
        if ref_file.exists() {
            let content = fs::read_to_string(ref_file)?;
            Ok(Some(content.trim().to_string()))
        } else {
            Ok(None)
        }
    } else {
        // Detached HEAD
        Ok(Some(head))
    }
}

#[allow(dead_code)]
pub fn read_ref(repo: &Repo, ref_path: &str) -> Result<Option<String>> {
    let ref_file = repo.git_dir.join(ref_path);
    if ref_file.exists() {
        let content = fs::read_to_string(ref_file)?;
        Ok(Some(content.trim().to_string()))
    } else {
        Ok(None)
    }
}

pub fn write_ref(repo: &Repo, ref_path: &str, sha: &str) -> Result<()> {
    let ref_file = repo.git_dir.join(ref_path);
    if let Some(parent) = ref_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&ref_file, format!("{}\n", sha))?;
    Ok(())
}

pub fn update_head(repo: &Repo, sha: &str) -> Result<()> {
    let head_content = read_head(repo)?;
    if let Some(ref_path) = head_content.strip_prefix("ref: ") {
        write_ref(repo, ref_path, sha)?;
    } else {
        // Detached HEAD — write SHA directly
        fs::write(repo.head_path(), format!("{}\n", sha))?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn update_head_ref(repo: &Repo, sha: &str) -> Result<()> {
    // Convenience: resolve the symbolic ref and write SHA to the branch file
    update_head(repo, sha)?;
    Ok(())
}

#[allow(dead_code)]
pub fn list_local_branches(git_dir: &Path) -> Result<Vec<String>> {
    let heads_dir = git_dir.join("refs").join("heads");
    if !heads_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut branches: Vec<String> = Vec::new();
    for entry in fs::read_dir(&heads_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && let Some(name) = entry.file_name().to_str() {
            branches.push(name.to_string());
        }
    }
    branches.sort();
    Ok(branches)
}

#[allow(dead_code)]
pub fn branch_exists(git_dir: &Path, name: &str) -> bool {
    git_dir.join("refs").join("heads").join(name).exists()
}

#[allow(dead_code)]
pub fn delete_branch(git_dir: &Path, name: &str) -> Result<()> {
    let path = git_dir.join("refs").join("heads").join(name);
    if !path.exists() {
        return Err(RitError::BranchNotFound(name.to_string()));
    }
    fs::remove_file(path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn current_branch_name(git_dir: &Path) -> Result<Option<String>> {
    let head_path = git_dir.join("HEAD");
    if !head_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(head_path)?;
    let trimmed = content.trim();
    if let Some(ref_path) = trimmed.strip_prefix("ref: refs/heads/") {
        Ok(Some(ref_path.to_string()))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> (Repo, TempDir) {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(git_dir.join("refs").join("heads")).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        (Repo::new(git_dir, tmp.path().to_path_buf()), tmp)
    }

    #[test]
    fn test_read_head_symbolic() {
        let (repo, _tmp) = setup_repo();
        let head = read_head(&repo).unwrap();
        assert_eq!(head, "ref: refs/heads/main");
    }

    #[test]
    fn test_write_and_resolve_ref() {
        let (repo, _tmp) = setup_repo();
        let sha = "abc123def456abc123def456abc123def456abc1";
        write_ref(&repo, "refs/heads/main", sha).unwrap();
        let resolved = resolve_head(&repo).unwrap();
        assert_eq!(resolved, Some(sha.to_string()));
    }

    #[test]
    fn test_update_head() {
        let (repo, _tmp) = setup_repo();
        let sha = "abc123def456abc123def456abc123def456abc1";
        update_head(&repo, sha).unwrap();
        let resolved = resolve_head(&repo).unwrap();
        assert_eq!(resolved, Some(sha.to_string()));
    }

    #[test]
    fn test_list_local_branches_sorted() {
        use std::fs;
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        let heads = git_dir.join("refs").join("heads");
        fs::create_dir_all(&heads).unwrap();
        fs::write(heads.join("zebra"), "abc\n").unwrap();
        fs::write(heads.join("alpha"), "def\n").unwrap();
        fs::write(heads.join("beta"), "ghi\n").unwrap();
        let branches = list_local_branches(&git_dir).unwrap();
        assert_eq!(branches, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_delete_branch_not_found() {
        let (repo, _tmp) = setup_repo();
        assert!(delete_branch(&repo.git_dir, "nonexistent").is_err());
    }

    #[test]
    fn test_current_branch_name_detached() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(git_dir.join("refs").join("heads")).unwrap();
        let sha = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        fs::write(git_dir.join("HEAD"), format!("{}\n", sha)).unwrap();
        let result = current_branch_name(&git_dir).unwrap();
        assert_eq!(result, None);
    }
}
