use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::{DorkForgeError, Result};
use crate::generator::{DorkQuery, GenerationResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffResult {
    pub format_version: String,
    pub before: String,
    pub after: String,
    pub added: Vec<DorkQuery>,
    pub removed: Vec<DorkQuery>,
    pub unchanged: usize,
}

pub fn read_generation_result(path: &Path) -> Result<GenerationResult> {
    let contents = fs::read_to_string(path).map_err(|source| DorkForgeError::IoError {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|err| DorkForgeError::ConfigError {
        path: path.to_path_buf(),
        message: format!("Failed to parse generation result JSON: {err}"),
    })
}

pub fn diff_results(before: &GenerationResult, after: &GenerationResult) -> DiffResult {
    let before_by_key: BTreeMap<String, &DorkQuery> = before
        .queries
        .iter()
        .map(|query| (query_key(query), query))
        .collect();
    let after_by_key: BTreeMap<String, &DorkQuery> = after
        .queries
        .iter()
        .map(|query| (query_key(query), query))
        .collect();

    let added = after_by_key
        .iter()
        .filter(|(key, _)| !before_by_key.contains_key(*key))
        .map(|(_, query)| (*query).clone())
        .collect();
    let removed = before_by_key
        .iter()
        .filter(|(key, _)| !after_by_key.contains_key(*key))
        .map(|(_, query)| (*query).clone())
        .collect();
    let unchanged = after_by_key
        .keys()
        .filter(|key| before_by_key.contains_key(*key))
        .count();

    DiffResult {
        format_version: "1.0".to_string(),
        before: before.generator_version.clone(),
        after: after.generator_version.clone(),
        added,
        removed,
        unchanged,
    }
}

pub fn format_diff_txt(diff: &DiffResult) -> String {
    let mut out = format!(
        "# DorkForge result diff\n# Added: {}, Removed: {}, Unchanged: {}\n",
        diff.added.len(),
        diff.removed.len(),
        diff.unchanged
    );
    append_queries(&mut out, "Added", &diff.added);
    append_queries(&mut out, "Removed", &diff.removed);
    out.trim_end().to_string()
}

fn append_queries(out: &mut String, heading: &str, queries: &[DorkQuery]) {
    out.push_str(&format!("\n## {heading}\n"));
    if queries.is_empty() {
        out.push_str("(none)\n");
        return;
    }
    for query in queries {
        out.push_str(&format!(
            "- [{} / {} / {}] {}\n",
            query.provider, query.category, query.fingerprint, query.query
        ));
    }
}

fn query_key(query: &DorkQuery) -> String {
    // The full provider/query pair is collision-free for this generated result;
    // the short fingerprint is intentionally only a display aid.
    format!("{}\0{}", query.provider, query.query)
}

#[cfg(test)]
mod tests {
    use super::diff_results;
    use crate::generator::{DorkQuery, GenerationParams, GenerationResult, GenerationStats};
    use std::collections::BTreeMap;

    fn result(queries: Vec<DorkQuery>) -> GenerationResult {
        GenerationResult {
            format_version: "1.0".to_string(),
            generator_version: "test".to_string(),
            parameters: GenerationParams {
                providers: vec![],
                categories: None,
                max_variants: 1,
                max_queries_per_identifier: 1,
                max_queries_per_provider: 1,
                country: None,
                network_or_asset: None,
            },
            inputs: vec![],
            queries,
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
        }
    }

    fn query(provider: &str, text: &str) -> DorkQuery {
        DorkQuery {
            query_id: text.to_string(),
            record_id: String::new(),
            id: "id".to_string(),
            template_id: "id".to_string(),
            template_version: None,
            validation_status: crate::templates::ValidationStatus::Untested,
            limitations: None,
            fingerprint: "deadbeef".to_string(),
            identifier_type: "domain".to_string(),
            identifier: "example.org".to_string(),
            variant: "example.org".to_string(),
            provider: provider.to_string(),
            category: "mentions".to_string(),
            query: text.to_string(),
            tags: vec![],
            identifiers: None,
        }
    }

    #[test]
    fn compares_using_full_query_not_short_fingerprint() {
        let diff = diff_results(
            &result(vec![query("google", "old")]),
            &result(vec![query("google", "new")]),
        );
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.unchanged, 0);
    }
}
