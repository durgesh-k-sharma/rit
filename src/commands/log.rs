use std::collections::HashMap;

use crate::error::*;
use crate::repo::Repo;
use crate::object::commit::parse_author_line;
use crate::object::read_object;
use crate::refs;

pub fn cmd_log(repo: &Repo) -> Result<()> {
    let head_sha = refs::resolve_head(repo)?;
    let head_sha = match head_sha {
        Some(sha) => sha,
        None => {
            let branch = get_current_branch(repo);
            return Err(RitError::NoCommits(branch));
        }
    };

    let mut ref_map: HashMap<String, Vec<String>> = HashMap::new();
    if let Ok(branches) = refs::list_local_branches(&repo.git_dir) {
        for branch in &branches {
            if let Some(sha) = refs::read_ref(repo, &format!("refs/heads/{}", branch))
                .ok().flatten()
            {
                ref_map.entry(sha).or_default().push(branch.clone());
            }
        }
    }

    let current_branch = refs::current_branch_name(&repo.git_dir)?;

    let mut current_sha = Some(head_sha.clone());

    while let Some(sha) = current_sha {
        let (_, content, _) = read_object(repo, &sha)?;
        let text = String::from_utf8_lossy(&content);
        let (headers, msg) = text.split_once("\n\n").unwrap_or((&text, ""));

        let mut _tree = String::new();
        let mut parent = None;
        let mut author_name = String::new();
        let mut author_email = String::new();
        let mut timestamp: i64 = 0;
        let mut tz_offset = String::new();

        for line in headers.lines() {
            if let Some(rest) = line.strip_prefix("tree ") {
                _tree = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("parent ") {
                parent = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("author ") {
                parse_author_line(rest, &mut author_name, &mut author_email,
                                  &mut timestamp, &mut tz_offset);
            }
        }

        let mut decorations: Vec<String> = Vec::new();
        if let Some(ref cur_branch) = current_branch
            && sha == head_sha
        {
            decorations.push(format!("HEAD -> {}", cur_branch));
        }
        if let Some(branches) = ref_map.get(&sha) {
            for branch in branches {
                if current_branch.as_deref() == Some(branch) && sha == head_sha {
                    continue;
                }
                decorations.push(branch.clone());
            }
        }

        let date_str = format_timestamp(timestamp, &tz_offset);

        if decorations.is_empty() {
            println!("commit {}", sha);
        } else {
            println!("commit {} ({})", sha, decorations.join(", "));
        }
        println!("Author: {} <{}>", author_name, author_email);
        println!("Date:   {}", date_str);
        println!();
        println!("    {}", msg.trim());
        println!();

        current_sha = parent;
    }
    Ok(())
}

fn format_timestamp(ts: i64, tz_offset: &str) -> String {
    use chrono::{DateTime, FixedOffset};

    let utc_dt = DateTime::from_timestamp(ts, 0).unwrap();
    let offset_secs = parse_tz_offset(tz_offset);
    let offset = FixedOffset::east_opt(offset_secs).unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
    let local_dt = utc_dt.with_timezone(&offset);
    local_dt.format("%a %b %e %H:%M:%S %Y %z").to_string()
}

fn parse_tz_offset(s: &str) -> i32 {
    if s.len() == 5 {
        let sign = if s.starts_with('-') { -1 } else { 1 };
        let hours: i32 = s[1..3].parse().unwrap_or(0);
        let minutes: i32 = s[3..5].parse().unwrap_or(0);
        sign * (hours * 3600 + minutes * 60)
    } else {
        0
    }
}

fn get_current_branch(repo: &Repo) -> String {
    if let Ok(head) = refs::read_head(repo) && let Some(ref_path) = head.strip_prefix("ref: refs/heads/") {
        return ref_path.to_string();
    }
    "HEAD".to_string()
}
