use std::fs;
use std::path::Path;

use crate::combination::model::{CombinationIdentifierInput, CombinationSpec, CombinationsFile};
use crate::error::{DorkForgeError, Result};

/// Parse a CLI string formatted as `<type>:<value>` (e.g. `username:sample_user`).
pub fn parse_cli_combine_arg(raw: &str) -> Result<CombinationIdentifierInput> {
    let trimmed = raw.trim();
    if let Some((type_part, value_part)) = trimmed.split_once(':') {
        let t = type_part.trim().to_string();
        let v = value_part.trim().to_string();
        if t.is_empty() || v.is_empty() {
            return Err(DorkForgeError::ValidationError(format!(
                "Invalid --combine argument '{raw}': type and value must not be empty"
            )));
        }
        Ok(CombinationIdentifierInput {
            r#type: t,
            value: v,
        })
    } else {
        Err(DorkForgeError::ValidationError(format!(
            "Invalid --combine argument '{raw}': expected format '<type>:<value>' (e.g. username:sample_user)"
        )))
    }
}

/// Parse a JSON file containing a `"combinations": [ ... ]` array.
pub fn parse_combinations_json(json_str: &str) -> Result<Vec<CombinationSpec>> {
    let parsed: CombinationsFile = serde_json::from_str(json_str).map_err(|e| {
        DorkForgeError::SerializationError(format!("Failed to parse combinations JSON: {e}"))
    })?;
    Ok(parsed.combinations)
}

/// Detect whether a JSON string is a combinations file.
pub fn is_combinations_json(json_str: &str) -> bool {
    json_str.contains("\"combinations\"")
}

/// Parse combinations from a file path.
pub fn load_combinations_from_file(path: &Path) -> Result<Vec<CombinationSpec>> {
    let content = fs::read_to_string(path).map_err(|e| DorkForgeError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_combinations_json(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_combine_arg() {
        let res = parse_cli_combine_arg("username:test_user").unwrap();
        assert_eq!(res.r#type, "username");
        assert_eq!(res.value, "test_user");

        assert!(parse_cli_combine_arg("invalid_no_colon").is_err());
        assert!(parse_cli_combine_arg(":value").is_err());
        assert!(parse_cli_combine_arg("type:").is_err());
    }

    #[test]
    fn test_parse_combinations_json() {
        let json = r#"{
            "combinations": [
                {
                    "id": "user_domain_test",
                    "identifiers": [
                        {"type": "username", "value": "test_user"},
                        {"type": "domain", "value": "example.org"}
                    ],
                    "categories": ["public_profiles"],
                    "providers": ["google"]
                }
            ]
        }"#;
        let combos = parse_combinations_json(json).unwrap();
        assert_eq!(combos.len(), 1);
        assert_eq!(combos[0].id, "user_domain_test");
        assert_eq!(combos[0].identifiers.len(), 2);
    }
}
