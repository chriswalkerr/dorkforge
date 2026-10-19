use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedEmail {
    pub raw: String,
    pub local_part: String,
    pub domain: String,
    pub canonical: String,
}

pub fn parse_and_normalize_email(input: &str) -> Result<NormalizedEmail> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "email".to_string(),
            value: input.to_string(),
            reason: "Email cannot be empty".to_string(),
        });
    }

    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "email".to_string(),
            value: input.to_string(),
            reason: "Email must contain exactly one '@' character".to_string(),
        });
    }

    let local_part = parts[0].trim();
    let domain_part = parts[1].trim();

    if local_part.is_empty() || domain_part.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "email".to_string(),
            value: input.to_string(),
            reason: "Local part and domain cannot be empty".to_string(),
        });
    }

    // Basic domain validation
    if !domain_part.contains('.') || domain_part.starts_with('.') || domain_part.ends_with('.') {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "email".to_string(),
            value: input.to_string(),
            reason: "Domain part must contain valid dot-separated labels".to_string(),
        });
    }

    // Disallow whitespace in local_part or domain
    if local_part.chars().any(char::is_whitespace) || domain_part.chars().any(char::is_whitespace) {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "email".to_string(),
            value: input.to_string(),
            reason: "Email cannot contain whitespace".to_string(),
        });
    }

    let normalized_domain = domain_part.to_ascii_lowercase();
    let canonical = format!("{}@{}", local_part, normalized_domain);

    Ok(NormalizedEmail {
        raw: input.to_string(),
        local_part: local_part.to_string(),
        domain: normalized_domain,
        canonical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let norm = parse_and_normalize_email(" User@Example.COM ").unwrap();
        assert_eq!(norm.local_part, "User");
        assert_eq!(norm.domain, "example.com");
        assert_eq!(norm.canonical, "User@example.com");
    }

    #[test]
    fn test_invalid_email() {
        assert!(parse_and_normalize_email("").is_err());
        assert!(parse_and_normalize_email("notanemail").is_err());
        assert!(parse_and_normalize_email("user@nodot").is_err());
        assert!(parse_and_normalize_email("user@.com").is_err());
    }
}
