use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedDomain {
    pub raw: String,
    pub domain: String,
    pub root_domain: Option<String>,
}

pub fn parse_and_normalize_domain(input: &str) -> Result<NormalizedDomain> {
    let mut cleaned = input.trim();
    if cleaned.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "domain".to_string(),
            value: input.to_string(),
            reason: "Domain cannot be empty".to_string(),
        });
    }

    // Strip scheme if present (e.g. https://example.com/ -> example.com)
    if let Some(pos) = cleaned.find("://") {
        cleaned = &cleaned[pos + 3..];
    }

    // Strip path or query or port if present
    if let Some(pos) = cleaned.find('/') {
        cleaned = &cleaned[..pos];
    }
    if let Some(pos) = cleaned.find(':') {
        cleaned = &cleaned[..pos];
    }

    // Strip trailing dot
    let cleaned = cleaned.trim_end_matches('.').to_ascii_lowercase();

    if cleaned.is_empty() || cleaned.len() > 253 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "domain".to_string(),
            value: input.to_string(),
            reason: "Domain length must be between 1 and 253 characters".to_string(),
        });
    }

    let labels: Vec<&str> = cleaned.split('.').collect();
    if labels.len() < 2 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "domain".to_string(),
            value: input.to_string(),
            reason: "Domain must contain at least a secondary level and top level domain"
                .to_string(),
        });
    }

    for label in &labels {
        if label.is_empty() || label.len() > 63 {
            return Err(DorkForgeError::InvalidInputFormat {
                identifier_type: "domain".to_string(),
                value: input.to_string(),
                reason: "Domain label must be between 1 and 63 characters".to_string(),
            });
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(DorkForgeError::InvalidInputFormat {
                identifier_type: "domain".to_string(),
                value: input.to_string(),
                reason: "Domain label cannot start or end with a hyphen".to_string(),
            });
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(DorkForgeError::InvalidInputFormat {
                identifier_type: "domain".to_string(),
                value: input.to_string(),
                reason: "Domain contains invalid characters".to_string(),
            });
        }
    }

    // Calculate root domain (last two labels, e.g. foo.bar.com -> bar.com)
    let root_domain = if labels.len() > 2 {
        Some(format!(
            "{}.{}",
            labels[labels.len() - 2],
            labels[labels.len() - 1]
        ))
    } else {
        None
    };

    Ok(NormalizedDomain {
        raw: input.to_string(),
        domain: cleaned,
        root_domain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_domain() {
        let norm = parse_and_normalize_domain("  HTTPS://WWW.Example.COM/test  ").unwrap();
        assert_eq!(norm.domain, "www.example.com");
        assert_eq!(norm.root_domain.as_deref(), Some("example.com"));
    }

    #[test]
    fn test_invalid_domain() {
        assert!(parse_and_normalize_domain("").is_err());
        assert!(parse_and_normalize_domain("nodot").is_err());
        assert!(parse_and_normalize_domain("-bad.com").is_err());
        assert!(parse_and_normalize_domain("bad-.com").is_err());
    }
}
