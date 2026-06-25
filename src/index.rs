use std::fs;
use sha1_smol::Sha1;
use crate::error::*;
use crate::repo::Repo;

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub ctime_sec: u32,
    pub ctime_nsec: u32,
    pub mtime_sec: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub sha: String,
    pub path: String,
}

#[derive(Debug)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Index { entries: Vec::new() }
    }

    pub fn read(repo: &Repo) -> Result<Self> {
        let index_path = repo.git_dir.join("index");
        if !index_path.exists() {
            return Ok(Index::new());
        }
        let data = fs::read(&index_path)?;
        if data.len() < 12 {
            return Ok(Index::new());
        }

        let (body, checksum) = data.split_at(data.len() - 20);
        let mut sha1 = Sha1::new();
        sha1.update(body);
        let expected_checksum = hex::encode(sha1.digest().bytes());
        let actual_checksum = hex::encode(checksum);
        if expected_checksum != actual_checksum {
            return Err(RitError::CorruptIndex);
        }

        if &body[0..4] != b"DIRC" {
            return Err(RitError::CorruptIndex);
        }
        let _version = u32::from_be_bytes([body[4], body[5], body[6], body[7]]);
        let count = u32::from_be_bytes([body[8], body[9], body[10], body[11]]);

        let mut entries = Vec::with_capacity(count as usize);
        let mut offset: usize = 12;

        for _ in 0..count {
            if offset + 62 > body.len() {
                break;
            }
            let entry = parse_entry(&body[offset..])?;
            offset += entry.0;
            entries.push(entry.1);
        }

        Ok(Index { entries })
    }

    pub fn write(&self, repo: &Repo) -> Result<()> {
        let mut body = Vec::new();
        body.extend_from_slice(b"DIRC");
        body.extend_from_slice(&2u32.to_be_bytes());
        body.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());

        let mut sorted = self.entries.clone();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        let mut raw_entries = Vec::new();
        for entry in &sorted {
            let raw = serialize_entry(entry);
            raw_entries.push(raw);
        }

        let entry_offset = body.len();
        for raw in &raw_entries {
            body.extend_from_slice(raw);
            let pad = (8 - (body.len() - entry_offset) % 8) % 8;
            body.extend(std::iter::repeat(0u8).take(pad));
        }

        let mut sha1 = Sha1::new();
        sha1.update(&body);
        let checksum = hex::encode(sha1.digest().bytes());
        body.extend_from_slice(&hex::decode(&checksum).unwrap());

        let index_path = repo.git_dir.join("index");
        fs::write(&index_path, &body)?;
        Ok(())
    }

    pub fn upsert(&mut self, entry: IndexEntry) {
        if let Some(pos) = self.entries.iter().position(|e| e.path == entry.path) {
            self.entries[pos] = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn get(&self, path: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.path == path)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[IndexEntry] {
        &self.entries
    }
}

fn parse_entry(data: &[u8]) -> Result<(usize, IndexEntry)> {
    let ctime_sec = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let ctime_nsec = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let mtime_sec = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let mtime_nsec = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
    let dev = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let ino = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    let mode = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
    let uid = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
    let gid = u32::from_be_bytes([data[32], data[33], data[34], data[35]]);
    let file_size = u32::from_be_bytes([data[36], data[37], data[38], data[39]]);
    let sha = hex::encode(&data[40..60]);
    let flags = u16::from_be_bytes([data[60], data[61]]);
    let name_len = (flags & 0x0FFF) as usize;

    let path_start = 62;
    let path_end = path_start + name_len;
    let path = String::from_utf8_lossy(&data[path_start..path_end]).to_string();

    let entry_size = (62 + name_len + 7) & !7;
    Ok((entry_size, IndexEntry {
        ctime_sec, ctime_nsec, mtime_sec, mtime_nsec,
        dev, ino, mode, uid, gid, file_size, sha, path,
    }))
}

fn serialize_entry(entry: &IndexEntry) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&entry.ctime_sec.to_be_bytes());
    buf.extend_from_slice(&entry.ctime_nsec.to_be_bytes());
    buf.extend_from_slice(&entry.mtime_sec.to_be_bytes());
    buf.extend_from_slice(&entry.mtime_nsec.to_be_bytes());
    buf.extend_from_slice(&entry.dev.to_be_bytes());
    buf.extend_from_slice(&entry.ino.to_be_bytes());
    buf.extend_from_slice(&entry.mode.to_be_bytes());
    buf.extend_from_slice(&entry.uid.to_be_bytes());
    buf.extend_from_slice(&entry.gid.to_be_bytes());
    buf.extend_from_slice(&entry.file_size.to_be_bytes());
    buf.extend_from_slice(&hex::decode(&entry.sha).unwrap());
    let name_len = std::cmp::min(entry.path.len(), 0xFFF) as u16;
    buf.extend_from_slice(&name_len.to_be_bytes());
    buf.extend_from_slice(entry.path.as_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> (Repo, TempDir) {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        (Repo::new(git_dir, tmp.path().to_path_buf()), tmp)
    }

    #[test]
    fn test_index_write_read_roundtrip() {
        let (repo, _tmp) = setup_repo();
        let mut index = Index::new();
        index.upsert(IndexEntry {
            ctime_sec: 100, ctime_nsec: 200, mtime_sec: 300, mtime_nsec: 400,
            dev: 1, ino: 2, mode: 0o100644, uid: 1000, gid: 1000,
            file_size: 12,
            sha: "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".to_string(),
            path: "hello.txt".to_string(),
        });
        index.write(&repo).unwrap();

        let read_back = Index::read(&repo).unwrap();
        assert_eq!(read_back.entries.len(), 1);
        assert_eq!(read_back.entries[0].path, "hello.txt");
        assert_eq!(read_back.entries[0].sha, "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }

    #[test]
    fn test_empty_index() {
        let (repo, _tmp) = setup_repo();
        let index = Index::read(&repo).unwrap();
        assert!(index.is_empty());
    }
}
