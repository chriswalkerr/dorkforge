use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DedupPolicy {
    #[default]
    Strict,
    Exact,
}

pub struct QueryDeduplicator {
    policy: DedupPolicy,
    seen_queries: HashSet<String>,
}

impl QueryDeduplicator {
    pub fn new(policy: DedupPolicy) -> Self {
        Self {
            policy,
            seen_queries: HashSet::new(),
        }
    }

    pub fn normalize_query_key(&self, provider: &str, query: &str) -> String {
        match self.policy {
            DedupPolicy::Exact => format!("{provider}::{query}"),
            DedupPolicy::Strict => {
                let normalized: String = query.split_whitespace().collect::<Vec<_>>().join(" ");
                format!("{}::{}", provider.to_ascii_lowercase(), normalized)
            }
        }
    }

    /// Returns true if the query is unique (not seen before), and marks it as seen.
    pub fn is_unique_and_mark(&mut self, provider: &str, query: &str) -> bool {
        let key = self.normalize_query_key(provider, query);
        self.seen_queries.insert(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_dedup() {
        let mut dedup = QueryDeduplicator::new(DedupPolicy::Strict);
        assert!(dedup.is_unique_and_mark("google", "site:example.com   filetype:pdf"));
        assert!(!dedup.is_unique_and_mark("google", "site:example.com filetype:pdf"));
        assert!(dedup.is_unique_and_mark("bing", "site:example.com filetype:pdf"));
    }

    #[test]
    fn test_strict_dedup_preserves_query_value_case() {
        let mut dedup = QueryDeduplicator::new(DedupPolicy::Strict);
        assert!(dedup.is_unique_and_mark("google", "\"Alice Smith\""));
        assert!(dedup.is_unique_and_mark("google", "\"alice smith\""));
        assert!(!dedup.is_unique_and_mark("google", "  \"Alice   Smith\"  "));
    }
}
