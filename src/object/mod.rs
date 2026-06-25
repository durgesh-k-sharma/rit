pub mod blob;
pub mod tree;
pub mod commit;

use std::fs;
use std::io::Read;
use std::io::Write;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use crate::error::*;
use crate::repo::Repo;

#[allow(dead_code)]
pub enum GitObject {
    Blob(blob::Blob),
    Tree(tree::Tree),
    Commit(commit::Commit),
}

pub fn hash_object(raw: &[u8]) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(raw);
    hex::encode(sha1.digest().bytes())
}

pub fn write_object(repo: &Repo, raw: &[u8]) -> Result<String> {
    let sha = hash_object(raw);
    let (dir, file) = object_path(repo, &sha);
    if file.exists() {
        return Ok(sha);
    }
    fs::create_dir_all(&dir)?;
    let compressed = compress(raw)?;
    let tmp_path = dir.join(format!(".{}.tmp", &sha[2..]));
    fs::write(&tmp_path, &compressed)?;
    fs::rename(&tmp_path, &file)?;
    Ok(sha)
}

pub fn read_object(repo: &Repo, sha_or_prefix: &str) -> Result<(String, Vec<u8>, String)> {
    let full_sha = resolve_prefix(repo, sha_or_prefix)?;
    let (_dir, file) = object_path(repo, &full_sha);
    let compressed = fs::read(&file).map_err(|_| RitError::ObjectNotFound(sha_or_prefix.to_string()))?;
    let decompressed = decompress(&compressed)?;
    let null_pos = decompressed.iter().position(|&b| b == 0)
        .ok_or_else(|| RitError::CorruptObject(sha_or_prefix.to_string()))?;
    let header = String::from_utf8_lossy(&decompressed[..null_pos]).to_string();
    let content = decompressed[null_pos + 1..].to_vec();
    let obj_type = header.split_whitespace().next()
        .ok_or_else(|| RitError::CorruptObject(sha_or_prefix.to_string()))?
        .to_string();
    Ok((obj_type, content, full_sha))
}

fn resolve_prefix(repo: &Repo, prefix: &str) -> Result<String> {
    if prefix.len() >= 40 {
        return Ok(prefix[..40].to_string());
    }
    if prefix.len() < 4 || !prefix.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(RitError::InvalidObjectName(prefix.to_string()));
    }
    let dir_name = &prefix[..2];
    let dir_path = repo.objects_path().join(dir_name);
    if !dir_path.is_dir() {
        return Err(RitError::ObjectNotFound(prefix.to_string()));
    }
    let mut matches: Vec<String> = Vec::new();
    for entry in fs::read_dir(&dir_path)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let full = format!("{}{}", dir_name, name_str);
        if full.starts_with(prefix) {
            matches.push(full);
        }
    }
    match matches.len() {
        0 => Err(RitError::ObjectNotFound(prefix.to_string())),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(RitError::AmbiguousPrefix(prefix.to_string())),
    }
}

fn object_path(repo: &Repo, sha: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = repo.objects_path().join(&sha[..2]);
    let file = dir.join(&sha[2..]);
    (dir, file)
}

fn compress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    encoder.finish()
}

fn decompress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_consistency() {
        let raw = b"blob 12\0hello world\n";
        let h1 = hash_object(raw);
        let h2 = hash_object(raw);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_write_and_read_object() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(git_dir.join("objects").join("xx")).unwrap();
        let repo = Repo::new(git_dir, tmp.path().to_path_buf());
        let raw = b"blob 12\0hello world\n";
        let sha = write_object(&repo, raw).unwrap();
        assert_eq!(sha.len(), 40);
        let (obj_type, content, _) = read_object(&repo, &sha).unwrap();
        assert_eq!(obj_type, "blob");
        assert_eq!(content, b"hello world\n");
    }
}
