use std::collections::HashSet;

pub struct BingProvider;

impl BingProvider {
    pub const ID: &'static str = "bing";
    pub const NAME: &'static str = "Microsoft Bing";

    pub const OPERATORS: &'static [&'static str] = &[
        "site:",
        "filetype:",
        "ext:",
        "inurl:",
        "intitle:",
        "inbody:",
        "inanchor:",
        "feed:",
        "hasfeed:",
        "ip:",
        "loc:",
        "language:",
        "contains:",
        "url:",
        "OR",
        "AND",
        "NOT",
        "-",
        "\"",
    ];

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
