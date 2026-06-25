use crate::error::*;
use crate::repo::Repo;
use crate::object::read_object;
use crate::object::tree::Tree;
use std::io::Write;

pub fn cmd_cat_file(
    type_only: bool,
    pretty_print: bool,
    size: bool,
    object: &str,
    repo: &Repo,
) -> Result<()> {
    let (obj_type, content, _full_sha) = read_object(repo, object)?;

    if type_only {
        println!("{}", obj_type);
    } else if size {
        println!("{}", content.len());
    } else if pretty_print {
        match obj_type.as_str() {
            "blob" => {
                std::io::stdout().write_all(&content)?;
            }
            "tree" => {
                let tree = Tree::parse(&content);
                for entry in &tree.entries {
                    let entry_type = if entry.is_tree { "tree" } else { "blob" };
                    println!("{} {} {}\t{}", entry.mode, entry_type, entry.sha, entry.name);
                }
            }
            "commit" => {
                print!("{}", String::from_utf8_lossy(&content));
                if !content.ends_with(b"\n") {
                    println!();
                }
            }
            _ => {
                print!("{}", String::from_utf8_lossy(&content));
            }
        }
    }

    Ok(())
}
