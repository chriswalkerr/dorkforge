use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedCertificate {
    pub raw: String,
    pub compact_hex: String,
    pub colon_separated: String,
}

pub fn parse_and_normalize_certificate(input: &str) -> Result<NormalizedCertificate> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "certificate".to_string(),
            value: input.to_string(),
            reason: "Certificate fingerprint cannot be empty".to_string(),
        });
    }

    let compact: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if compact.len() < 16 || compact.len() > 64 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "certificate".to_string(),
            value: input.to_string(),
            reason: format!(
                "Certificate fingerprint must be between 16 and 64 hex characters, found {}",
                compact.len()
            ),
        });
    }

    let colon_separated = compact
        .as_bytes()
        .chunks(2)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<_>>()
        .join(":");

    Ok(NormalizedCertificate {
        raw: input.to_string(),
        compact_hex: compact,
        colon_separated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_fingerprint() {
        let cert = parse_and_normalize_certificate(
            "3B:A0:6B:7C:F4:A8:12:44:89:10:00:11:22:33:44:55:66:77:88:99",
        )
        .unwrap();
        assert_eq!(cert.compact_hex, "3BA06B7CF4A81244891000112233445566778899");
        assert_eq!(
            cert.colon_separated,
            "3B:A0:6B:7C:F4:A8:12:44:89:10:00:11:22:33:44:55:66:77:88:99"
        );
    }
}
