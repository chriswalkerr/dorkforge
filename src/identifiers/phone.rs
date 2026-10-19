use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPhone {
    pub raw: String,
    pub digits_only: String,
    pub international_prefix: Option<String>,
    pub e164: Option<String>,
}

pub fn parse_and_normalize_phone(
    input: &str,
    country_code_hint: Option<&str>,
) -> Result<NormalizedPhone> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "phone".to_string(),
            value: input.to_string(),
            reason: "Phone number cannot be empty".to_string(),
        });
    }

    // Check characters: only allow '+', digits, spaces, hyphens, parentheses, dots
    let valid_chars = trimmed.chars().all(|c| {
        c.is_ascii_digit() || c == '+' || c == ' ' || c == '-' || c == '(' || c == ')' || c == '.'
    });
    if !valid_chars {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "phone".to_string(),
            value: input.to_string(),
            reason: "Phone number contains invalid characters".to_string(),
        });
    }

    let digits_only: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits_only.len() < 5 || digits_only.len() > 15 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "phone".to_string(),
            value: input.to_string(),
            reason: format!(
                "Phone number must contain between 5 and 15 digits, found {}",
                digits_only.len()
            ),
        });
    }

    let starts_with_plus = trimmed.starts_with('+');
    let international_prefix = if starts_with_plus {
        // has plus
        Some("+".to_string())
    } else {
        country_code_hint.map(|hint| {
            let clean = hint.trim();
            if clean.starts_with('+') {
                clean.to_string()
            } else {
                format!("+{clean}")
            }
        })
    };

    let e164 = if starts_with_plus {
        Some(format!("+{digits_only}"))
    } else if let Some(ref pfx) = international_prefix {
        let pfx_digits: String = pfx.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits_only.starts_with(&pfx_digits) {
            Some(format!("+{digits_only}"))
        } else {
            Some(format!("+{pfx_digits}{digits_only}"))
        }
    } else {
        None
    };

    Ok(NormalizedPhone {
        raw: input.to_string(),
        digits_only,
        international_prefix,
        e164,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_phone_with_plus() {
        let norm = parse_and_normalize_phone("+1 (555) 123-4567", None).unwrap();
        assert_eq!(norm.digits_only, "15551234567");
        assert_eq!(norm.e164.as_deref(), Some("+15551234567"));
    }

    #[test]
    fn test_valid_phone_with_country_hint() {
        let norm = parse_and_normalize_phone("555-123-4567", Some("+1")).unwrap();
        assert_eq!(norm.digits_only, "5551234567");
        assert_eq!(norm.e164.as_deref(), Some("+15551234567"));
    }

    #[test]
    fn test_invalid_phone() {
        assert!(parse_and_normalize_phone("", None).is_err());
        assert!(parse_and_normalize_phone("abcde", None).is_err());
        assert!(parse_and_normalize_phone("123", None).is_err()); // too short
    }
}
