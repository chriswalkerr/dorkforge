use std::collections::BTreeMap;

use crate::combination::model::{CombinationVariantBinding, ResolvedCombinationIdentifier};
use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, Copy)]
pub struct CombinationPlannerLimits {
    pub max_variant_combinations: usize,
    pub max_combination_queries: usize,
    pub strict: bool,
}

impl Default for CombinationPlannerLimits {
    fn default() -> Self {
        Self {
            max_variant_combinations: 25,
            max_combination_queries: 100,
            strict: false,
        }
    }
}

/// Generates a deterministic cartesian product of variant bindings for a combination,
/// bounded by configured limits.
pub fn plan_variant_combinations(
    resolved: &[ResolvedCombinationIdentifier],
    limits: CombinationPlannerLimits,
    warnings: &mut Vec<String>,
) -> Result<Vec<CombinationVariantBinding>> {
    if resolved.is_empty() {
        return Ok(Vec::new());
    }

    // Calculate total theoretical combinations
    let mut total_theoretical: usize = 1;
    for r in resolved {
        let count = r.variants.len().max(1);
        total_theoretical = total_theoretical.saturating_mul(count);
    }

    if total_theoretical > limits.max_variant_combinations {
        let msg = format!(
            "Combination variant product ({total_theoretical}) exceeds limit of {}",
            limits.max_variant_combinations
        );
        if limits.strict {
            return Err(DorkForgeError::LimitExceeded { message: msg });
        } else {
            warnings.push(format!("{msg}; truncating combination variants"));
        }
    }

    let mut current_bindings: Vec<CombinationVariantBinding> = vec![CombinationVariantBinding {
        values_by_type: BTreeMap::new(),
        variants_by_type: BTreeMap::new(),
    }];

    for r in resolved {
        let mut next_bindings = Vec::new();
        for b in &current_bindings {
            for v in &r.variants {
                let mut new_vals = b.values_by_type.clone();
                let mut new_vars = b.variants_by_type.clone();

                new_vals.insert(r.id_type.as_str().to_string(), v.clone());
                new_vars.insert(r.id_type.as_str().to_string(), v.clone());

                next_bindings.push(CombinationVariantBinding {
                    values_by_type: new_vals,
                    variants_by_type: new_vars,
                });

                if next_bindings.len() >= limits.max_variant_combinations {
                    break;
                }
            }
            if next_bindings.len() >= limits.max_variant_combinations {
                break;
            }
        }
        current_bindings = next_bindings;
    }

    Ok(current_bindings)
}

/// Substitute bound combination variants into a template string.
pub fn substitute_combination_template(
    template_str: &str,
    binding: &CombinationVariantBinding,
) -> String {
    let mut result = template_str.to_string();

    for (type_name, val) in &binding.values_by_type {
        let placeholder = format!("{{{type_name}}}");
        result = result.replace(&placeholder, val);
    }

    // Also support {domain} if present in binding
    if let Some(domain_val) = binding.values_by_type.get("domain") {
        result = result.replace("{domain}", domain_val);
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::IdentifierType;

    #[test]
    fn test_cartesian_product_planning() {
        let r1 = ResolvedCombinationIdentifier {
            id_type: IdentifierType::Username,
            original_value: "alice".to_string(),
            parsed: crate::identifiers::ParsedIdentifier::Username(
                crate::identifiers::username::NormalizedUsername {
                    raw: "alice".to_string(),
                    cleaned: "alice".to_string(),
                },
            ),
            variants: vec!["alice".to_string(), "@alice".to_string()],
        };

        let r2 = ResolvedCombinationIdentifier {
            id_type: IdentifierType::Domain,
            original_value: "example.org".to_string(),
            parsed: crate::identifiers::ParsedIdentifier::Domain(
                crate::identifiers::domain::NormalizedDomain {
                    raw: "example.org".to_string(),
                    domain: "example.org".to_string(),
                    root_domain: None,
                },
            ),
            variants: vec!["example.org".to_string()],
        };

        let mut warnings = Vec::new();
        let bindings = plan_variant_combinations(
            &[r1, r2],
            CombinationPlannerLimits::default(),
            &mut warnings,
        )
        .unwrap();

        assert_eq!(bindings.len(), 2);

        let rendered =
            substitute_combination_template(r#"site:{domain} "{username}""#, &bindings[0]);
        assert_eq!(rendered, "site:example.org \"alice\"");
    }
}
