use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedUsername {
    pub raw: String,
    pub cleaned: String,
}

pub fn parse_and_normalize_username(input: &str) -> Result<NormalizedUsername> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "username".to_string(),
            value: input.to_string(),
            reason: "Username cannot be empty".to_string(),
        });
    }

    // Strip leading '@' if user provided '@username'
    let cleaned = trimmed.strip_prefix('@').unwrap_or(trimmed).trim();
    if cleaned.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "username".to_string(),
            value: input.to_string(),
            reason: "Username cannot be just '@'".to_string(),
        });
    }

    if cleaned.chars().any(char::is_whitespace) {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "username".to_string(),
            value: input.to_string(),
            reason: "Username cannot contain whitespace".to_string(),
        });
    }

    // Allow alphanumeric, underscore, hyphen, dot, plus unicode characters for handles
    let valid_chars = cleaned
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.');
    if !valid_chars {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "username".to_string(),
            value: input.to_string(),
            reason: "Username contains invalid special characters".to_string(),
        });
    }

    Ok(NormalizedUsername {
        raw: input.to_string(),
        cleaned: cleaned.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let norm = parse_and_normalize_username(" @john_doe ").unwrap();
        assert_eq!(norm.raw, " @john_doe ");
        assert_eq!(norm.cleaned, "john_doe");
    }

    #[test]
    fn test_invalid_username() {
        assert!(parse_and_normalize_username("").is_err());
        assert!(parse_and_normalize_username("@").is_err());
        assert!(parse_and_normalize_username("john doe").is_err());
        assert!(parse_and_normalize_username("john$doe").is_err());
    }
}
