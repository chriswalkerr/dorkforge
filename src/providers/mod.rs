pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod google;
pub mod yandex;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::str::FromStr;

use crate::error::{DorkForgeError, Result};
pub use bing::BingProvider;
pub use brave::BraveProvider;
pub use duckduckgo::DuckDuckGoProvider;
pub use google::GoogleProvider;
pub use yandex::YandexProvider;

const SUPPORTED_CATEGORIES: &[&str] = &[
    "exact_match",
    "mentions",
    "documents",
    "public_profiles",
    "forums",
    "code_repositories",
    "domain_pages",
    "subdomains",
    "archives",
    "public_records",
    "company_records",
    "security_reports",
    "technology_fingerprint",
    "marketplaces",
    "custom",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Google,
    Bing,
    Yandex,
    Brave,
    #[serde(rename = "duckduckgo", alias = "ddg", alias = "duck_duck_go")]
    DuckDuckGo,
}

impl Provider {
    pub const ALL: &'static [Provider] = &[
        Provider::Google,
        Provider::Bing,
        Provider::Yandex,
        Provider::Brave,
        Provider::DuckDuckGo,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Provider::Google => GoogleProvider::ID,
            Provider::Bing => BingProvider::ID,
            Provider::Yandex => YandexProvider::ID,
            Provider::Brave => BraveProvider::ID,
            Provider::DuckDuckGo => DuckDuckGoProvider::ID,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Provider::Google => GoogleProvider::NAME,
            Provider::Bing => BingProvider::NAME,
            Provider::Yandex => YandexProvider::NAME,
            Provider::Brave => BraveProvider::NAME,
            Provider::DuckDuckGo => DuckDuckGoProvider::NAME,
        }
    }

    pub fn supported_operators(&self) -> &'static [&'static str] {
        match self {
            Provider::Google => GoogleProvider::OPERATORS,
            Provider::Bing => BingProvider::OPERATORS,
            Provider::Yandex => YandexProvider::OPERATORS,
            Provider::Brave => BraveProvider::OPERATORS,
            Provider::DuckDuckGo => DuckDuckGoProvider::OPERATORS,
        }
    }

    pub fn supported_categories(&self) -> &'static [&'static str] {
        SUPPORTED_CATEGORIES
    }

    pub fn supports_operator(&self, op: &str) -> bool {
        match self {
            Provider::Google => GoogleProvider::supports_operator(op),
            Provider::Bing => BingProvider::supports_operator(op),
            Provider::Yandex => YandexProvider::supports_operator(op),
            Provider::Brave => BraveProvider::supports_operator(op),
            Provider::DuckDuckGo => DuckDuckGoProvider::supports_operator(op),
        }
    }

    pub fn supports_category(&self, category: &str) -> bool {
        let cat_lower = category.to_ascii_lowercase();
        self.supported_categories()
            .iter()
            .any(|&c| c.to_ascii_lowercase() == cat_lower)
    }

    /// Validates if all operator tokens in a template query string are supported by this provider.
    pub fn validate_template_operators(
        &self,
        template_str: &str,
    ) -> std::result::Result<(), String> {
        for operator in Self::template_operators(template_str) {
            if !self.supports_operator(&operator) {
                return Err(format!(
                    "Operator '{}' is not supported by provider '{}'",
                    operator,
                    self.id()
                ));
            }
        }

        Ok(())
    }

    fn template_operators(template_str: &str) -> HashSet<String> {
        let has_quotes = template_str.contains('"');
        let mut outside_quotes = String::with_capacity(template_str.len());
        let mut in_quotes = false;
        for character in template_str.chars() {
            if character == '"' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                outside_quotes.push(character);
            }
        }

        let mut operators = HashSet::new();
        if has_quotes {
            operators.insert("\"".to_string());
        }

        for token in outside_quotes.split_whitespace() {
            let token = token.trim_matches(|character| character == '(' || character == ')');
            let mut token = token;
            if token.starts_with('-') && token.len() > 1 {
                operators.insert("-".to_string());
                token = &token[1..];
            }
            if token.starts_with('+') && token.len() > 1 {
                operators.insert("+".to_string());
                token = &token[1..];
            }

            if matches!(token, "OR" | "AND" | "NOT") {
                operators.insert(token.to_string());
            }
            for operator in ["&&", "~~", "|"] {
                if token.contains(operator) {
                    operators.insert(operator.to_string());
                }
            }

            if let Some(colon_index) = token.find(':') {
                let prefix = &token[..=colon_index];
                if prefix.len() > 1
                    && prefix[..prefix.len() - 1]
                        .chars()
                        .all(|character| character.is_ascii_alphabetic())
                {
                    operators.insert(prefix.to_string());
                }
            }
        }

        operators
    }

    /// Returns the base search URL pattern for building clickable query links.
    pub fn search_base_url(&self) -> &'static str {
        match self {
            Provider::Google => "https://www.google.com/search?q=",
            Provider::Bing => "https://www.bing.com/search?q=",
            Provider::Yandex => "https://yandex.com/search/?text=",
            Provider::Brave => "https://search.brave.com/search?q=",
            Provider::DuckDuckGo => "https://duckduckgo.com/?q=",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl FromStr for Provider {
    type Err = DorkForgeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "google" => Ok(Provider::Google),
            "bing" => Ok(Provider::Bing),
            "yandex" => Ok(Provider::Yandex),
            "brave" => Ok(Provider::Brave),
            "duckduckgo" | "ddg" => Ok(Provider::DuckDuckGo),
            other => Err(DorkForgeError::UnknownProvider(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub supported_operators: Vec<String>,
    pub supported_categories: Vec<String>,
    #[serde(default)]
    pub documentation_urls: Vec<String>,
    #[serde(default)]
    pub last_checked: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub operator_metadata: BTreeMap<String, OperatorMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorDocumentationStatus {
    Documented,
    Experimental,
    Unknown,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorTestStatus {
    Untested,
    Passed,
    Partial,
    Failed,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorMetadata {
    pub documentation_status: OperatorDocumentationStatus,
    pub test_status: OperatorTestStatus,
    pub description: String,
    #[serde(default)]
    pub limitations: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersFile {
    pub providers: Vec<ProviderConfig>,
}

impl Default for ProvidersFile {
    fn default() -> Self {
        Self {
            providers: Provider::ALL
                .iter()
                .map(|p| ProviderConfig {
                    id: p.id().to_string(),
                    name: p.display_name().to_string(),
                    enabled: true,
                    supported_operators: p
                        .supported_operators()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    supported_categories: p
                        .supported_categories()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    documentation_urls: Vec::new(),
                    last_checked: None,
                    notes: None,
                    operator_metadata: BTreeMap::new(),
                })
                .collect(),
        }
    }
}
