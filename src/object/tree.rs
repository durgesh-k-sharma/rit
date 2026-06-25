use sha1_smol::Sha1;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub sha: String,
    pub is_tree: bool,
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl TreeEntry {
    fn sort_key(&self) -> String {
        if self.is_tree {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        }
    }
}

impl Tree {
    pub fn from_entries(mut entries: Vec<TreeEntry>) -> Self {
        entries.sort_by(|a, b| {
            let key_a = a.sort_key();
            let key_b = b.sort_key();
            key_a.cmp(&key_b)
        });
        Tree { entries }
    }

    pub fn hash(&self) -> String {
        let content = self.serialize_content();
        let header = format!("tree {}\0", content.len());
        let mut sha1 = Sha1::new();
        sha1.update(header.as_bytes());
        sha1.update(&content);
        hex::encode(sha1.digest().bytes())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let content = self.serialize_content();
        let header = format!("tree {}\0", content.len());
        [header.as_bytes(), &content].concat()
    }

    fn serialize_content(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for entry in &self.entries {
            buf.extend_from_slice(entry.mode.as_bytes());
            buf.push(b' ');
            buf.extend_from_slice(entry.name.as_bytes());
            buf.push(0);
            let sha_bytes = hex::decode(&entry.sha).expect("valid hex sha");
            buf.extend_from_slice(&sha_bytes);
        }
        buf
    }

    pub fn parse(content: &[u8]) -> Self {
        let mut entries = Vec::new();
        let mut i = 0;
        while i < content.len() {
            let mode_end = content[i..].iter().position(|&b| b == b' ').unwrap();
            let mode = String::from_utf8_lossy(&content[i..i + mode_end]).to_string();
            i += mode_end + 1;

            let name_end = content[i..].iter().position(|&b| b == 0).unwrap();
            let name = String::from_utf8_lossy(&content[i..i + name_end]).to_string();
            i += name_end + 1;

            let sha = hex::encode(&content[i..i + 20]);
            i += 20;

            let is_tree = mode == "040000";
            entries.push(TreeEntry { mode, name, sha, is_tree });
        }
        Tree { entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sort_order() {
        let entries = vec![
            TreeEntry { mode: "100644".into(), name: "zebra.txt".into(), sha: "a".repeat(40), is_tree: false },
            TreeEntry { mode: "100644".into(), name: "apple.txt".into(), sha: "b".repeat(40), is_tree: false },
            TreeEntry { mode: "040000".into(), name: "beta".into(), sha: "c".repeat(40), is_tree: true },
        ];
        let tree = Tree::from_entries(entries);
        assert_eq!(tree.entries[0].name, "apple.txt");
        assert_eq!(tree.entries[1].name, "beta");
        assert_eq!(tree.entries[2].name, "zebra.txt");
    }

    #[test]
    fn test_tree_serialize_roundtrip() {
        let entries = vec![
            TreeEntry { mode: "100644".into(), name: "a.txt".into(), sha: "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".into(), is_tree: false },
        ];
        let tree = Tree::from_entries(entries);
        let serialized = tree.serialize();
        assert!(serialized.starts_with(b"tree "));
    }
}
