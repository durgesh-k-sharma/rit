use crate::error::*;
use crate::repo::Repo;
use crate::refs;
use crate::checkout;

pub fn cmd_checkout(
    target: &str,
    new_branch: bool,
    start_point: Option<&str>,
    force: bool,
    repo: &Repo,
) -> Result<()> {
    if new_branch {
        let start_sha = match start_point {
            Some(sp) => resolve_start_point(sp, repo)?,
            None => refs::resolve_head(repo)?
                .ok_or_else(|| RitError::InvalidObjectName("HEAD".to_string()))?,
        };

        if refs::branch_exists(&repo.git_dir, target) {
            return Err(RitError::BranchAlreadyExists(target.to_string()));
        }

        refs::write_ref(repo, &format!("refs/heads/{}", target), &start_sha)?;
        checkout::switch_to_branch(repo, target, &start_sha, force)?;
        println!("Switched to a new branch '{}'", target);
    } else {
        let head_content = refs::read_head(repo)?;
        let current_branch = head_content.strip_prefix("ref: refs/heads/")
            .map(|s| s.to_string());

        if current_branch.as_deref() == Some(target) {
            println!("Already on '{}'", target);
            let sha = refs::resolve_head(repo)?
                .ok_or_else(|| RitError::NoCommits(target.to_string()))?;
            checkout::switch_to_branch(repo, target, &sha, force)?;
            return Ok(());
        }

        let sha = refs::read_ref(repo, &format!("refs/heads/{}", target))?
            .ok_or_else(|| {
                if target.len() >= 4 && target.chars().all(|c| c.is_ascii_hexdigit()) {
                    return RitError::DetachedHead;
                }
                RitError::UnknownRef(target.to_string())
            })?;

        checkout::switch_to_branch(repo, target, &sha, force)?;
        println!("Switched to branch '{}'", target);
    }
    Ok(())
}

fn resolve_start_point(sp: &str, repo: &Repo) -> Result<String> {
    if let Some(sha) = refs::read_ref(repo, &format!("refs/heads/{}", sp))? {
        return Ok(sha);
    }
    let (_type, _content, full_sha) = crate::object::read_object(repo, sp)?;
    Ok(full_sha)
}
