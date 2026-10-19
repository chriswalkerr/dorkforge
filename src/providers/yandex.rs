use std::collections::HashSet;

pub struct YandexProvider;

impl YandexProvider {
    pub const ID: &'static str = "yandex";
    pub const NAME: &'static str = "Yandex Search";

    pub const OPERATORS: &'static [&'static str] = &[
        "site:", "domain:", "host:", "mime:", "title:", "inurl:", "rhost:", "url:", "lang:",
        "date:", "&&", "|", "~~", "-", "\"",
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
