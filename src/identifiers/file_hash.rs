use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedFileHash {
    pub raw: String,
    pub hash: String,
    pub algorithm: HashAlgorithm,
}

pub fn parse_and_normalize_file_hash(input: &str) -> Result<NormalizedFileHash> {
    let mut trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "file_hash".to_string(),
            value: input.to_string(),
            reason: "File hash cannot be empty".to_string(),
        });
    }

    if let Some(pos) = trimmed.find(':') {
        trimmed = trimmed[pos + 1..].trim();
    }

    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "file_hash".to_string(),
            value: input.to_string(),
            reason: "File hash must be a valid hexadecimal string".to_string(),
        });
    }

    let algorithm = match trimmed.len() {
        32 => HashAlgorithm::Md5,
        40 => HashAlgorithm::Sha1,
        64 => HashAlgorithm::Sha256,
        _ => HashAlgorithm::Unknown,
    };

    if algorithm == HashAlgorithm::Unknown && (trimmed.len() < 16 || trimmed.len() > 128) {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "file_hash".to_string(),
            value: input.to_string(),
            reason: format!("Unexpected hash length: {}", trimmed.len()),
        });
    }

    Ok(NormalizedFileHash {
        raw: input.to_string(),
        hash: trimmed.to_ascii_lowercase(),
        algorithm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_hash_parsing() {
        let md5 = parse_and_normalize_file_hash("MD5: e4d909c290d0fb1ca068ffaddf22cbd0").unwrap();
        assert_eq!(md5.algorithm, HashAlgorithm::Md5);
        assert_eq!(md5.hash, "e4d909c290d0fb1ca068ffaddf22cbd0");
    }
}
