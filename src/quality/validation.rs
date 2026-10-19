use std::collections::HashSet;

use crate::error::{DorkForgeError, Result};
use crate::providers::Provider;

pub const DEFAULT_ALLOWED_PLACEHOLDERS: &[&str] = &["value", "domain", "country"];

/// Offline validation of a URL format without network access.
/// Checks scheme (must be http:// or https://), host presence, valid characters,
/// and no embedded whitespace.
pub fn validate_url_format(raw_url: &str) -> Result<()> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::ValidationError(
            "URL cannot be empty".to_string(),
        ));
    }

    if !trimmed.starts_with("https://") && !trimmed.starts_with("http://") {
        return Err(DorkForgeError::ValidationError(format!(
            "URL '{trimmed}' must start with http:// or https://"
        )));
    }

    if trimmed.chars().any(|c| c.is_whitespace()) {
        return Err(DorkForgeError::ValidationError(format!(
            "URL '{trimmed}' contains invalid whitespace"
        )));
    }

    let after_scheme = if let Some(rest) = trimmed.strip_prefix("https://") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        rest
    } else {
        trimmed
    };

    let host = after_scheme.split('/').next().unwrap_or("");
    let host = host.split('?').next().unwrap_or("");
    let host = host.split('#').next().unwrap_or("");

    if host.is_empty() || !host.contains('.') {
        return Err(DorkForgeError::ValidationError(format!(
            "URL '{trimmed}' does not have a valid host"
        )));
    }

    Ok(())
}

/// Extracts all placeholder names `{name}` from a template string.
pub fn extract_placeholders(template_str: &str) -> Result<Vec<String>> {
    let mut placeholders = Vec::new();
    let mut chars = template_str.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            let mut closed = false;
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '}' {
                    closed = true;
                    break;
                }
                name.push(next_c);
            }

            if !closed {
                return Err(DorkForgeError::ValidationError(
                    "Unclosed placeholder '{'".to_string(),
                ));
            }

            placeholders.push(name);
        }
    }
    Ok(placeholders)
}

/// Validates placeholders for a template string given a custom set of allowed placeholders.
pub fn validate_placeholders_with_allowed(
    template_id: &str,
    template_str: &str,
    allowed: &[&str],
) -> Result<()> {
    let found =
        extract_placeholders(template_str).map_err(|e| DorkForgeError::InvalidTemplate {
            id: template_id.to_string(),
            message: e.to_string(),
        })?;

    for placeholder in found {
        if !allowed.contains(&placeholder.as_str()) {
            return Err(DorkForgeError::UnknownPlaceholder {
                template_id: template_id.to_string(),
                placeholder,
            });
        }
    }

    Ok(())
}

/// Validates that an operator requirement is supported by the specified providers.
pub fn validate_operator_requirement_for_providers(
    template_id: &str,
    operator: &str,
    providers: &[Provider],
) -> Result<()> {
    for provider in providers {
        if !provider.supports_operator(operator) {
            return Err(DorkForgeError::InvalidTemplate {
                id: template_id.to_string(),
                message: format!(
                    "Required operator '{operator}' is not supported by provider '{}'",
                    provider.id()
                ),
            });
        }
    }
    Ok(())
}

/// Check duplicate IDs among templates.
pub fn check_duplicates<'a, I>(ids: I) -> Result<()>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(DorkForgeError::InvalidTemplate {
                id: id.to_string(),
                message: format!("Duplicate template ID: '{id}'"),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_url_format("https://support.google.com/websearch").is_ok());
        assert!(validate_url_format("http://example.org/path?q=test").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        assert!(validate_url_format("ftp://example.org").is_err());
        assert!(validate_url_format("https://").is_err());
        assert!(validate_url_format("https://nodot").is_err());
        assert!(validate_url_format("https://example .org").is_err());
    }

    #[test]
    fn test_extract_placeholders() {
        let extracted = extract_placeholders(r#"site:{domain} "{username}""#).unwrap();
        assert_eq!(extracted, vec!["domain", "username"]);
    }
}
