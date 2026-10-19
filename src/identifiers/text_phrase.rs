use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedTextPhrase {
    pub raw: String,
    pub phrase: String,
}

pub fn parse_and_normalize_text_phrase(input: &str) -> Result<NormalizedTextPhrase> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "text_phrase".to_string(),
            value: input.to_string(),
            reason: "Text phrase cannot be empty".to_string(),
        });
    }

    let phrase = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

    Ok(NormalizedTextPhrase {
        raw: input.to_string(),
        phrase,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_phrase() {
        let p = parse_and_normalize_text_phrase("  CONFIDENTIAL   PROJECT   OMEGA  ").unwrap();
        assert_eq!(p.phrase, "CONFIDENTIAL PROJECT OMEGA");
    }
}
