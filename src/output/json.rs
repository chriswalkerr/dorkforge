use crate::error::{DorkForgeError, Result};
use crate::generator::GenerationResult;

pub fn format_json(result: &GenerationResult, pretty: bool) -> Result<String> {
    if pretty {
        serde_json::to_string_pretty(result)
            .map_err(|e| DorkForgeError::SerializationError(e.to_string()))
    } else {
        serde_json::to_string(result).map_err(|e| DorkForgeError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationParams;
    use crate::generator::GenerationStats;
    use std::collections::BTreeMap;

    #[test]
    fn test_format_json() {
        let res = GenerationResult {
            format_version: "1.0".to_string(),
            generator_version: "1.0.0".to_string(),
            parameters: GenerationParams {
                providers: vec!["google".to_string()],
                categories: None,
                max_variants: 10,
                max_queries_per_identifier: 50,
                max_queries_per_provider: 100,
                country: None,
                network_or_asset: None,
            },
            inputs: vec!["test@test.com".to_string()],
            queries: vec![],
            stats: GenerationStats {
                total_queries: 0,
                unique_query_texts: 0,
                unique_provider_queries: 0,
                templates_used: 0,
                queries_by_validation_status: BTreeMap::new(),
                queries_by_provider: BTreeMap::new(),
                queries_by_category: BTreeMap::new(),
                warnings: vec![],
            },
        };

        let json_str = format_json(&res, true).unwrap();
        assert!(json_str.contains("\"format_version\": \"1.0\""));
        assert!(json_str.contains("\"generator_version\": \"1.0.0\""));
    }
}
