use crate::generator::GenerationResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TxtMode {
    Clean,
    #[default]
    Readable,
}

pub fn format_txt(result: &GenerationResult, mode: TxtMode) -> String {
    match mode {
        TxtMode::Clean => {
            let mut lines = Vec::new();
            for query in &result.queries {
                lines.push(query.query.clone());
            }
            lines.join("\n")
        }
        TxtMode::Readable => {
            let mut out = String::new();
            out.push_str("# DorkForge Generation Results\n");
            out.push_str(&format!(
                "# Total queries: {}, Inputs: {}\n\n",
                result.stats.total_queries,
                result.inputs.join(", ")
            ));

            let mut current_provider = "";
            let mut current_category = "";
            let mut current_variant = "";

            for query in &result.queries {
                if query.provider != current_provider {
                    current_provider = &query.provider;
                    current_category = "";
                    current_variant = "";
                    out.push_str(&format!(
                        "\n## Provider: {}\n",
                        current_provider.to_uppercase()
                    ));
                }

                if query.category != current_category {
                    current_category = &query.category;
                    current_variant = "";
                    out.push_str(&format!("### Category: {}\n", current_category));
                }

                if query.variant != current_variant {
                    current_variant = &query.variant;
                    out.push_str(&format!("#### Variant: {}\n", current_variant));
                }

                out.push_str(&query.query);
                out.push('\n');
            }

            out.trim_end().to_string()
        }
    }
}

/// Render results as clickable search engine URLs instead of raw dork strings.
/// Requires a parsed Provider lookup to get the base URL for each query's provider.
pub fn format_urls(result: &GenerationResult) -> String {
    let mut lines = Vec::new();
    for query in &result.queries {
        lines.push(super::search_url(query));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{DorkQuery, GenerationParams, GenerationStats};
    use std::collections::BTreeMap;

    #[test]
    fn test_format_clean_txt() {
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
            queries: vec![DorkQuery {
                query_id: "test1_google_abc12345".to_string(),
                record_id: "dfq1_test".to_string(),
                id: "test1".to_string(),
                template_id: "test1".to_string(),
                template_version: None,
                validation_status: crate::templates::ValidationStatus::Untested,
                limitations: None,
                fingerprint: "abc12345".to_string(),
                identifier_type: "email".to_string(),
                identifier: "test@test.com".to_string(),
                variant: "test@test.com".to_string(),
                provider: "google".to_string(),
                category: "exact_match".to_string(),
                query: "\"test@test.com\"".to_string(),
                tags: vec![],
                identifiers: None,
            }],
            stats: GenerationStats {
                total_queries: 1,
                unique_query_texts: 1,
                unique_provider_queries: 1,
                templates_used: 1,
                queries_by_validation_status: BTreeMap::from([("untested".to_string(), 1)]),
                queries_by_provider: BTreeMap::new(),
                queries_by_category: BTreeMap::new(),
                warnings: vec![],
            },
        };

        let clean = format_txt(&res, TxtMode::Clean);
        assert_eq!(clean, "\"test@test.com\"");

        let readable = format_txt(&res, TxtMode::Readable);
        assert!(readable.contains("## Provider: GOOGLE"));
        assert!(readable.contains("\"test@test.com\""));
    }
}
