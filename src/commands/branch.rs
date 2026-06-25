use crate::error::*;
use crate::repo::Repo;
use crate::refs;

pub fn cmd_branch(name: Option<String>, delete: bool, repo: &Repo) -> Result<()> {
    if let Some(name) = name {
        if delete {
            cmd_delete_branch(&name, repo)
        } else {
            cmd_create_branch(&name, repo)
        }
    } else {
        cmd_list_branches(repo)
    }
}

fn cmd_list_branches(repo: &Repo) -> Result<()> {
    let current = refs::current_branch_name(&repo.git_dir)?;
    let branches = refs::list_local_branches(&repo.git_dir)?;

    match current {
        Some(ref cur) => {
            for branch in &branches {
                if branch == cur {
                    println!("* {}", branch);
                } else {
                    println!("  {}", branch);
                }
            }
        }
        None => {
            let head_sha = refs::resolve_head(repo)?;
            let short = head_sha.as_deref().map(|s| &s[..7]).unwrap_or("???");
            println!("* (HEAD detached at {})", short);
            for branch in &branches {
                println!("  {}", branch);
            }
        }
    }
    Ok(())
}

fn cmd_create_branch(name: &str, repo: &Repo) -> Result<()> {
    if refs::branch_exists(&repo.git_dir, name) {
        return Err(RitError::BranchAlreadyExists(name.to_string()));
    }
    let head_sha = refs::resolve_head(repo)?
        .ok_or_else(|| RitError::InvalidObjectName(name.to_string()))?;
    refs::write_ref(repo, &format!("refs/heads/{}", name), &head_sha)?;
    Ok(())
}

fn cmd_delete_branch(name: &str, repo: &Repo) -> Result<()> {
    if !refs::branch_exists(&repo.git_dir, name) {
        return Err(RitError::BranchNotFound(name.to_string()));
    }
    let current = refs::current_branch_name(&repo.git_dir)?;
    if current.as_deref() == Some(name) {
        let git_dir_str = repo.git_dir.to_string_lossy().to_string();
        return Err(RitError::BranchCheckedOut(name.to_string(), git_dir_str));
    }
    let sha = refs::read_ref(repo, &format!("refs/heads/{}", name))?
        .ok_or_else(|| RitError::BranchNotFound(name.to_string()))?;
    let short_sha = &sha[..7];
    refs::delete_branch(&repo.git_dir, name)?;
    println!("Deleted branch {} (was {}).", name, short_sha);
    Ok(())
}
