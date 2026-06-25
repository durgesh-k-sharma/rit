use crate::error::*;
use crate::repo::Repo;
use crate::object::read_object;
use crate::refs;

pub fn cmd_log(repo: &Repo) -> Result<()> {
    let head_sha = refs::resolve_head(repo)?;
    let head_sha = match head_sha {
        Some(sha) => sha,
        None => {
            let branch = get_current_branch(repo);
            eprintln!("fatal: your current branch '{}' does not have any commits yet", branch);
            std::process::exit(1);
        }
    };

    let mut current_sha = Some(head_sha);

    while let Some(sha) = current_sha {
        let (_type, content, _) = read_object(repo, &sha)?;
        let text = String::from_utf8_lossy(&content);
        let (headers, msg) = text.split_once("\n\n").unwrap_or((&text, ""));

        // Parse headers
        let mut _tree = String::new();
        let mut parent = None;
        let mut author_name = String::new();
        let mut author_email = String::new();
        let mut timestamp: i64 = 0;

        for line in headers.lines() {
            if let Some(rest) = line.strip_prefix("tree ") {
                _tree = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("parent ") {
                parent = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("author ") {
                parse_author_line(rest, &mut author_name, &mut author_email, &mut timestamp);
            }
        }

        let date_str = format_timestamp(timestamp);

        println!("commit {}", sha);
        println!("Author: {} <{}>", author_name, author_email);
        println!("Date:   {}", date_str);
        println!();
        println!("    {}", msg.trim());
        println!();

        current_sha = parent;
    }

    Ok(())
}

fn parse_author_line(line: &str, name: &mut String, email: &mut String, timestamp: &mut i64) {
    if let Some(rest) = line.split_once(" <") {
        *name = rest.0.to_string();
        if let Some(rest2) = rest.1.rsplit_once("> ") {
            *email = rest2.0.to_string();
            let ts_part = rest2.1;
            if let Some((ts, _tz)) = ts_part.split_once(' ') {
                *timestamp = ts.parse().unwrap_or(0);
            }
        }
    }
}

fn format_timestamp(ts: i64) -> String {
    use chrono::DateTime;
    let secs = if ts >= 0 { ts as u64 } else { 0 };
    let dt = DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_default();
    dt.format("%a %b %e %H:%M:%S %Y %z").to_string()
}

fn get_current_branch(repo: &Repo) -> String {
    if let Ok(head) = refs::read_head(repo) {
        if let Some(ref_path) = head.strip_prefix("ref: refs/heads/") {
            return ref_path.to_string();
        }
    }
    "HEAD".to_string()
}
