use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedUrl {
    pub raw: String,
    pub host: Option<String>,
    pub path: String,
    pub filename: Option<String>,
    pub canonical: String,
}

pub fn parse_and_normalize_url(input: &str) -> Result<NormalizedUrl> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "url".to_string(),
            value: input.to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }

    let mut rest = trimmed;
    let mut host = None;

    if let Some(pos) = rest.find("://") {
        rest = &rest[pos + 3..];
        if let Some(slash_idx) = rest.find('/') {
            host = Some(rest[..slash_idx].to_ascii_lowercase());
            rest = &rest[slash_idx..];
        } else {
            host = Some(rest.to_ascii_lowercase());
            rest = "/";
        }
    } else if let Some(slash_idx) = rest.find('/') {
        if slash_idx > 0 && !rest.starts_with('/') && rest[..slash_idx].contains('.') {
            host = Some(rest[..slash_idx].to_ascii_lowercase());
            rest = &rest[slash_idx..];
        }
    }

    let path = rest.to_string();
    let filename = path
        .split('/')
        .rfind(|segment| !segment.is_empty() && segment.contains('.'))
        .map(|s| s.to_string());

    let canonical = if let Some(ref h) = host {
        format!("{h}{path}")
    } else {
        path.clone()
    };

    Ok(NormalizedUrl {
        raw: input.to_string(),
        host,
        path,
        filename,
        canonical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normalization() {
        let u = parse_and_normalize_url("https://example.com/secret/passwords.txt").unwrap();
        assert_eq!(u.host.as_deref(), Some("example.com"));
        assert_eq!(u.filename.as_deref(), Some("passwords.txt"));
    }
}
