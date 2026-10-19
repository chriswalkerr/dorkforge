use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedCompany {
    pub raw: String,
    pub clean_name: String,
    pub legal_form: Option<String>,
    pub canonical: String,
}

const COMMON_LEGAL_FORMS: &[&str] = &[
    "ltd",
    "limited",
    "inc",
    "incorporated",
    "corp",
    "corporation",
    "llc",
    "llp",
    "plc",
    "gmbh",
    "ag",
    "sarl",
    "sa",
    "bv",
    "nv",
    "ооо",
    "зао",
    "оао",
    "пао",
    "ип",
    "doo",
    "d.o.o.",
    "ad",
    "a.d.",
];

pub fn parse_and_normalize_company(input: &str) -> Result<NormalizedCompany> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "company".to_string(),
            value: input.to_string(),
            reason: "Company name cannot be empty".to_string(),
        });
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "company".to_string(),
            value: input.to_string(),
            reason: "Company name cannot be empty".to_string(),
        });
    }

    let mut detected_legal = None;
    let mut name_tokens = Vec::new();

    for token in &tokens {
        let clean_token = token
            .trim_matches(|c: char| {
                c == ',' || c == '.' || c == '"' || c == '\'' || c == '«' || c == '»'
            })
            .to_lowercase();

        if COMMON_LEGAL_FORMS.contains(&clean_token.as_str()) {
            detected_legal = Some(token.to_string());
        } else {
            name_tokens.push(*token);
        }
    }

    let clean_name = if name_tokens.is_empty() {
        trimmed.to_string()
    } else {
        name_tokens.join(" ")
    };

    let canonical = clean_name
        .trim_matches(|c: char| c == '"' || c == '\'' || c == '«' || c == '»')
        .to_string();

    Ok(NormalizedCompany {
        raw: input.to_string(),
        clean_name,
        legal_form: detected_legal,
        canonical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_normalization() {
        let c = parse_and_normalize_company("Acme Corporation, LLC").unwrap();
        assert_eq!(c.clean_name, "Acme");
        assert!(c.legal_form.is_some());

        let c2 = parse_and_normalize_company("«Яндекс» ООО").unwrap();
        assert_eq!(c2.canonical, "Яндекс");
    }
}
