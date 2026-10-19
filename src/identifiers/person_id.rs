use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPersonId {
    pub raw: String,
    pub digits_or_compact: String,
    pub country: Option<String>,
    pub id_type_hint: Option<String>,
    pub canonical: String,
}

pub fn parse_and_normalize_person_id(
    input: &str,
    country_hint: Option<&str>,
) -> Result<NormalizedPersonId> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "person_id".to_string(),
            value: input.to_string(),
            reason: "Person / National ID cannot be empty".to_string(),
        });
    }

    let compact: String = trimmed.chars().filter(|c| c.is_alphanumeric()).collect();

    if compact.len() < 4 || compact.len() > 30 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "person_id".to_string(),
            value: input.to_string(),
            reason: format!(
                "Person ID must contain between 4 and 30 characters, found {}",
                compact.len()
            ),
        });
    }

    let country = country_hint.map(|c| c.trim().to_uppercase());

    let mut id_type_hint = None;
    if let Some(ref c) = country {
        match c.as_str() {
            "RU" => {
                if compact.len() == 10 || compact.len() == 12 {
                    id_type_hint = Some("INN".to_string());
                } else if compact.len() == 13 || compact.len() == 15 {
                    id_type_hint = Some("OGRN".to_string());
                }
            }
            "RS" => {
                if compact.len() == 13 {
                    id_type_hint = Some("JMBG".to_string());
                } else if compact.len() == 8 || compact.len() == 9 {
                    id_type_hint = Some("PIB".to_string());
                }
            }
            "US" if compact.len() == 9 => {
                id_type_hint = Some("SSN/EIN".to_string());
            }
            _ => {}
        }
    }

    let canonical = compact.to_uppercase();

    Ok(NormalizedPersonId {
        raw: input.to_string(),
        digits_or_compact: compact,
        country,
        id_type_hint,
        canonical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_id_parsing() {
        let id = parse_and_normalize_person_id("7707083893", Some("RU")).unwrap();
        assert_eq!(id.id_type_hint.as_deref(), Some("INN"));
        assert_eq!(id.canonical, "7707083893");

        let ssn = parse_and_normalize_person_id("123-45-6789", Some("US")).unwrap();
        assert_eq!(ssn.digits_or_compact, "123456789");
        assert_eq!(ssn.id_type_hint.as_deref(), Some("SSN/EIN"));
    }
}
