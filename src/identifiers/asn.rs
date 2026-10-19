use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedAsn {
    pub raw: String,
    pub number: u32,
    pub canonical: String,
}

pub fn parse_and_normalize_asn(input: &str) -> Result<NormalizedAsn> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "asn".to_string(),
            value: input.to_string(),
            reason: "ASN cannot be empty".to_string(),
        });
    }

    let mut clean = trimmed;
    if clean.to_ascii_uppercase().starts_with("AS") {
        clean = clean[2..].trim();
    }

    match clean.parse::<u32>() {
        Ok(num) if num > 0 => Ok(NormalizedAsn {
            raw: input.to_string(),
            number: num,
            canonical: format!("AS{num}"),
        }),
        _ => Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "asn".to_string(),
            value: input.to_string(),
            reason: "ASN must be a valid positive 16-bit or 32-bit autonomous system number"
                .to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asn() {
        let a1 = parse_and_normalize_asn("AS15169").unwrap();
        assert_eq!(a1.number, 15169);
        assert_eq!(a1.canonical, "AS15169");

        let a2 = parse_and_normalize_asn(" 13335 ").unwrap();
        assert_eq!(a2.number, 13335);
        assert_eq!(a2.canonical, "AS13335");
    }
}
