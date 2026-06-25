use crate::error::*;
use crate::repo::Repo;
use std::fs;

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

pub fn update_head_ref(repo: &Repo, sha: &str) -> Result<()> {
    // Convenience: resolve the symbolic ref and write SHA to the branch file
    update_head(repo, sha)?;
    Ok(())
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
}
