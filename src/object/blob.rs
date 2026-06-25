use sha1_smol::Sha1;

#[derive(Debug, Clone)]
pub struct Blob {
    pub content: Vec<u8>,
}

impl Blob {
    #[allow(dead_code)]
    pub fn hash(&self) -> String {
        let header = format!("blob {}\0", self.content.len());
        let mut sha1 = Sha1::new();
        sha1.update(header.as_bytes());
        sha1.update(&self.content);
        hex::encode(sha1.digest().bytes())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let header = format!("blob {}\0", self.content.len());
        [header.as_bytes(), &self.content].concat()
    }

    pub fn from_content(content: Vec<u8>) -> Self {
        Blob { content }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_hash() {
        let blob = Blob::from_content(b"hello world\n".to_vec());
        let hash = blob.hash();
        assert_eq!(hash, "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    }

    #[test]
    fn test_blob_serialize_roundtrip() {
        let blob = Blob::from_content(b"test content".to_vec());
        let serialized = blob.serialize();
        assert!(serialized.starts_with(b"blob 12\0"));
        assert_eq!(&serialized[serialized.len() - 12..], b"test content");
    }
}
