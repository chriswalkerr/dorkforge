use std::collections::HashSet;

use crate::error::{DorkForgeError, Result};
use crate::identifiers::IdentifierType;
use crate::providers::Provider;

pub const ALLOWED_PLACEHOLDERS: &[&str] = &["value", "domain", "country"];

/// Validates placeholders in a template string.
pub fn validate_placeholders(
    template_id: &str,
    template_str: &str,
    allowed: &[&str],
) -> Result<()> {
    let mut chars = template_str.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut placeholder = String::new();
            let mut closed = false;
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '}' {
                    closed = true;
                    break;
                }
                placeholder.push(next_c);
            }

            if !closed {
                return Err(DorkForgeError::InvalidTemplate {
                    id: template_id.to_string(),
                    message: "Unclosed placeholder '{'".to_string(),
                });
            }

            if !allowed.contains(&placeholder.as_str()) {
                return Err(DorkForgeError::UnknownPlaceholder {
                    template_id: template_id.to_string(),
                    placeholder,
                });
            }
        }
    }

    Ok(())
}

/// Validates that the template category is applicable to the identifier type.
pub fn validate_category_for_type(
    template_id: &str,
    id_type: IdentifierType,
    category: &str,
) -> Result<()> {
    let applicable = id_type.applicable_categories();
    let cat_lower = category.to_ascii_lowercase();

    if !applicable
        .iter()
        .any(|&c| c.to_ascii_lowercase() == cat_lower)
    {
        return Err(DorkForgeError::InvalidTemplate {
            id: template_id.to_string(),
            message: format!(
                "Category '{category}' is not applicable to identifier type '{id_type}'"
            ),
        });
    }

    Ok(())
}

/// Validates template operators against the specified providers.
pub fn validate_template_for_providers(
    template_id: &str,
    template_str: &str,
    providers: &[Provider],
) -> Result<()> {
    for provider in providers {
        if let Err(msg) = provider.validate_template_operators(template_str) {
            return Err(DorkForgeError::InvalidTemplate {
                id: template_id.to_string(),
                message: format!("Provider '{}' incompatibility: {}", provider.id(), msg),
            });
        }
    }
    Ok(())
}

/// Validates a list of template definitions for duplicate IDs.
pub fn check_duplicate_template_ids<'a, I>(ids: I) -> Result<()>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(DorkForgeError::InvalidTemplate {
                id: id.to_string(),
                message: "Duplicate template ID".to_string(),
            });
        }
    }
    Ok(())
}
