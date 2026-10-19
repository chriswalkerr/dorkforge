use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::error::{DorkForgeError, Result};
use crate::identifiers::{parse_identifier, IdentifierType, ParsedIdentifier};

#[derive(Debug, Clone, Deserialize)]
pub struct InputItem {
    #[serde(rename = "type")]
    pub identifier_type: Option<String>,
    pub value: String,
    pub country: Option<String>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedBatch {
    pub identifiers: Vec<ParsedIdentifier>,
    pub errors: Vec<(String, String)>,
}

pub struct InputParser {
    strict_mode: bool,
}

impl InputParser {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    pub fn parse_single(
        &self,
        value: &str,
        type_hint: Option<IdentifierType>,
        country: Option<&str>,
        network: Option<&str>,
    ) -> Result<ParsedIdentifier> {
        parse_identifier(type_hint, value, country, network)
    }

    pub fn parse_file(&self, path: &Path) -> Result<ParsedBatch> {
        if !path.exists() {
            return Err(DorkForgeError::IoError {
                path: path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "Input file not found"),
            });
        }

        let content = fs::read_to_string(path).map_err(|e| DorkForgeError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if extension.eq_ignore_ascii_case("json") {
            self.parse_json_str(&content)
        } else {
            self.parse_txt_str(&content)
        }
    }

    pub fn parse_json_str(&self, content: &str) -> Result<ParsedBatch> {
        let mut batch = ParsedBatch::default();

        // 1. Try parsing as Vec<InputItem>
        if let Ok(items) = serde_json::from_str::<Vec<InputItem>>(content) {
            for item in items {
                let id_type = match item.identifier_type {
                    Some(ref t) => match t.parse::<IdentifierType>() {
                        Ok(it) => Some(it),
                        Err(e) => {
                            if self.strict_mode {
                                return Err(e);
                            }
                            batch.errors.push((item.value.clone(), e.to_string()));
                            continue;
                        }
                    },
                    None => None,
                };

                match parse_identifier(
                    id_type,
                    &item.value,
                    item.country.as_deref(),
                    item.network.as_deref(),
                ) {
                    Ok(parsed) => batch.identifiers.push(parsed),
                    Err(e) => {
                        if self.strict_mode {
                            return Err(e);
                        }
                        batch.errors.push((item.value, e.to_string()));
                    }
                }
            }
            return Ok(batch);
        }

        // 2. Try parsing as Vec<String>
        if let Ok(strings) = serde_json::from_str::<Vec<String>>(content) {
            for val in strings {
                match self.parse_line(&val) {
                    Ok(parsed) => batch.identifiers.push(parsed),
                    Err(e) => {
                        if self.strict_mode {
                            return Err(e);
                        }
                        batch.errors.push((val, e.to_string()));
                    }
                }
            }
            return Ok(batch);
        }

        Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "batch_json".to_string(),
            value: content.chars().take(50).collect(),
            reason: "Expected JSON array of objects with 'value' or array of strings".to_string(),
        })
    }

    pub fn parse_txt_str(&self, content: &str) -> Result<ParsedBatch> {
        let mut batch = ParsedBatch::default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            match self.parse_line(trimmed) {
                Ok(parsed) => batch.identifiers.push(parsed),
                Err(e) => {
                    if self.strict_mode {
                        return Err(e);
                    }
                    batch.errors.push((trimmed.to_string(), e.to_string()));
                }
            }
        }

        Ok(batch)
    }

    fn parse_line(&self, line: &str) -> Result<ParsedIdentifier> {
        // Line can be: "type:value" or just "value"
        if let Some(colon_pos) = line.find(':') {
            let potential_type = &line[..colon_pos].trim();
            let remainder = &line[colon_pos + 1..].trim();

            if let Ok(id_type) = potential_type.parse::<IdentifierType>() {
                return parse_identifier(Some(id_type), remainder, None, None);
            }
        }

        // Auto-detect or default
        parse_identifier(None, line, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_json_parsing() {
        let json = r#"
        [
            {"type": "email", "value": "test@example.com"},
            {"type": "ip", "value": "192.168.1.1"},
            {"value": "invalid_ambiguous"}
        ]
        "#;

        let parser = InputParser::new(false);
        let batch = parser.parse_json_str(json).unwrap();
        assert_eq!(batch.identifiers.len(), 2);
        assert_eq!(batch.errors.len(), 1);
    }

    #[test]
    fn test_batch_txt_parsing() {
        let txt = "email:user@domain.com\n# comment\n1.1.1.1\n";
        let parser = InputParser::new(false);
        let batch = parser.parse_txt_str(txt).unwrap();
        assert_eq!(batch.identifiers.len(), 2);
    }
}
