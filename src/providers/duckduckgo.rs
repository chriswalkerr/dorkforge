use std::collections::HashSet;

pub struct DuckDuckGoProvider;

impl DuckDuckGoProvider {
    pub const ID: &'static str = "duckduckgo";
    pub const NAME: &'static str = "DuckDuckGo";

    pub const OPERATORS: &'static [&'static str] =
        &["site:", "filetype:", "inurl:", "intitle:", "-", "+", "\""];

    pub const CATEGORIES: &'static [&'static str] = super::SUPPORTED_CATEGORIES;

    pub fn supports_operator(op: &str) -> bool {
        let op_lower = op.to_ascii_lowercase();
        Self::OPERATORS
            .iter()
            .any(|&o| o.to_ascii_lowercase() == op_lower)
    }

    pub fn operator_set() -> HashSet<&'static str> {
        Self::OPERATORS.iter().copied().collect()
    }
}
