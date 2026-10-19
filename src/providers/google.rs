use std::collections::HashSet;

pub struct GoogleProvider;

impl GoogleProvider {
    pub const ID: &'static str = "google";
    pub const NAME: &'static str = "Google Search";

    pub const OPERATORS: &'static [&'static str] = &[
        "site:",
        "filetype:",
        "ext:",
        "inurl:",
        "allinurl:",
        "intitle:",
        "allintitle:",
        "intext:",
        "allintext:",
        "OR",
        "AND",
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
