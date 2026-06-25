use sha1_smol::Sha1;

#[derive(Debug, Clone)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub tz_offset: String,
    pub message: String,
}

impl Commit {
    pub fn new(
        tree: String,
        parent: Option<String>,
        author_name: String,
        author_email: String,
        timestamp: i64,
        tz_offset: String,
        message: String,
    ) -> Self {
        Commit { tree, parent, author_name, author_email, timestamp, tz_offset, message }
    }

    pub fn hash(&self) -> String {
        let content = self.serialize_content();
        let header = format!("commit {}\0", content.len());
        let mut sha1 = Sha1::new();
        sha1.update(header.as_bytes());
        sha1.update(&content);
        hex::encode(sha1.digest().bytes())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let content = self.serialize_content();
        let header = format!("commit {}\0", content.len());
        [header.as_bytes(), &content].concat()
    }

    fn serialize_content(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(format!("tree {}\n", self.tree).as_bytes());
        if let Some(ref parent) = self.parent {
            buf.extend_from_slice(format!("parent {}\n", parent).as_bytes());
        }
        buf.extend_from_slice(
            format!("author {} <{}> {} {}\n", self.author_name, self.author_email, self.timestamp, self.tz_offset).as_bytes()
        );
        buf.extend_from_slice(
            format!("committer {} <{}> {} {}\n", self.author_name, self.author_email, self.timestamp, self.tz_offset).as_bytes()
        );
        buf.push(b'\n');
        buf.extend_from_slice(self.message.as_bytes());
        buf.push(b'\n');
        buf
    }

    pub fn parse(content: &[u8]) -> Self {
        let text = String::from_utf8_lossy(content);
        let mut tree = String::new();
        let mut parent = None;
        let mut author_name = String::new();
        let mut author_email = String::new();
        let mut timestamp: i64 = 0;
        let mut tz_offset = String::new();
        let mut message = String::new();

        let mut header_done = false;

        for line in text.lines() {
            if !header_done && line.is_empty() {
                header_done = true;
                continue;
            }
            if !header_done {
                if let Some(rest) = line.strip_prefix("tree ") {
                    tree = rest.to_string();
                } else if let Some(rest) = line.strip_prefix("parent ") {
                    parent = Some(rest.to_string());
                } else if let Some(rest) = line.strip_prefix("author ") {
                    parse_author_line(rest, &mut author_name, &mut author_email, &mut timestamp, &mut tz_offset);
                } else if let Some(_rest) = line.strip_prefix("committer ") {
                }
            } else {
                if !message.is_empty() {
                    message.push('\n');
                }
                message.push_str(line);
            }
        }

        Commit { tree, parent, author_name, author_email, timestamp, tz_offset, message }
    }
}

fn parse_author_line(line: &str, name: &mut String, email: &mut String, timestamp: &mut i64, tz_offset: &mut String) {
    if let Some(rest) = line.split_once(" <") {
        *name = rest.0.to_string();
        if let Some(rest2) = rest.1.rsplit_once("> ") {
            *email = rest2.0.to_string();
            let ts_part = rest2.1;
            if let Some((ts, tz)) = ts_part.split_once(' ') {
                *timestamp = ts.parse().unwrap_or(0);
                *tz_offset = tz.to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_roundtrip() {
        let commit = Commit::new(
            "abc123".to_string(),
            None,
            "Test User".to_string(),
            "test@example.com".to_string(),
            1700000000,
            "+0000".to_string(),
            "initial commit".to_string(),
        );
        let serialized = commit.serialize();
        let parsed = Commit::parse(&serialized[serialized.iter().position(|&b| b == 0).unwrap() + 1..]);
        assert_eq!(parsed.tree, commit.tree);
        assert_eq!(parsed.author_name, commit.author_name);
        assert_eq!(parsed.message, commit.message);
    }
}
