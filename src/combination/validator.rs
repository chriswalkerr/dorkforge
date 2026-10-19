use std::collections::HashSet;

use crate::combination::model::{CombinationSpec, ResolvedCombinationIdentifier};
use crate::error::{DorkForgeError, Result};
use crate::identifiers::{parse_identifier, IdentifierType};
use crate::variants::{VariantConfig, VariantGenerator};

/// Validates and normalizes all identifiers in a combination specification.
pub fn validate_and_resolve_combination(
    spec: &CombinationSpec,
    max_variants_per_identifier: usize,
    strict: bool,
) -> Result<Vec<ResolvedCombinationIdentifier>> {
    let combo_id = &spec.id;

    if spec.identifiers.len() < 2 {
        return Err(DorkForgeError::ValidationError(format!(
            "Combination '{combo_id}' must contain at least 2 identifiers (found {})",
            spec.identifiers.len()
        )));
    }

    let mut seen_pairs = HashSet::new();
    let mut resolved = Vec::new();
    let variant_gen = VariantGenerator::new(VariantConfig {
        max_variants_per_identifier,
    });

    for (index, input) in spec.identifiers.iter().enumerate() {
        if input.value.trim().is_empty() {
            return Err(DorkForgeError::ValidationError(format!(
                "Combination '{combo_id}'.identifiers[{index}]: value cannot be empty"
            )));
        }

        let id_type = input.r#type.parse::<IdentifierType>().map_err(|e| {
            DorkForgeError::ValidationError(format!(
                "Combination '{combo_id}'.identifiers[{index}]: {e}"
            ))
        })?;

        let pair_key = format!("{}:{}", id_type.as_str(), input.value.trim().to_lowercase());
        if !seen_pairs.insert(pair_key) {
            let msg = format!(
                "Combination '{combo_id}'.identifiers[{index}]: duplicate identifier '{}:{}'",
                id_type.as_str(),
                input.value
            );
            if strict {
                return Err(DorkForgeError::ValidationError(msg));
            } else {
                eprintln!("[DorkForge Warning] {msg}");
            }
        }

        let parsed = parse_identifier(Some(id_type), &input.value, None, None).map_err(|e| {
            DorkForgeError::ValidationError(format!(
                "Combination '{combo_id}'.identifiers[{index}]: {e}"
            ))
        })?;

        let variants = variant_gen.generate(&parsed);

        resolved.push(ResolvedCombinationIdentifier {
            id_type,
            original_value: input.value.clone(),
            parsed,
            variants,
        });
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combination::model::CombinationIdentifierInput;

    #[test]
    fn test_validate_valid_combination() {
        let spec = CombinationSpec {
            id: "test".to_string(),
            identifiers: vec![
                CombinationIdentifierInput {
                    r#type: "username".to_string(),
                    value: "sample_user".to_string(),
                },
                CombinationIdentifierInput {
                    r#type: "domain".to_string(),
                    value: "example.org".to_string(),
                },
            ],
            categories: None,
            providers: None,
        };

        let resolved = validate_and_resolve_combination(&spec, 5, true).unwrap();
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_reject_less_than_two_identifiers() {
        let spec = CombinationSpec {
            id: "too_short".to_string(),
            identifiers: vec![CombinationIdentifierInput {
                r#type: "username".to_string(),
                value: "sample_user".to_string(),
            }],
            categories: None,
            providers: None,
        };

        assert!(validate_and_resolve_combination(&spec, 5, true).is_err());
    }

    #[test]
    fn test_reject_empty_value() {
        let spec = CombinationSpec {
            id: "empty_val".to_string(),
            identifiers: vec![
                CombinationIdentifierInput {
                    r#type: "username".to_string(),
                    value: "".to_string(),
                },
                CombinationIdentifierInput {
                    r#type: "domain".to_string(),
                    value: "example.org".to_string(),
                },
            ],
            categories: None,
            providers: None,
        };

        assert!(validate_and_resolve_combination(&spec, 5, true).is_err());
    }
}
