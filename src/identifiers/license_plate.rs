use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedLicensePlate {
    pub raw: String,
    pub cleaned: String,
    pub country: Option<String>,
    pub region_code: Option<String>,
    pub digits: Option<String>,
    pub suffix: Option<String>,
}

/// Known Serbian municipality codes for license plates
pub const SERBIA_REGION_CODES: &[&str] = &[
    "BG", "NS", "NI", "KG", "SU", "ZR", "SO", "PA", "SM", "PO", "ZA", "CA", "KV", "UE", "VA", "SA",
    "PZ", "VR", "LE", "PI", "AR", "BB", "BO", "BP", "BT", "BČ", "ČA", "DE", "GL", "GM", "IC", "IN",
    "JA", "KA", "KC", "KL", "KM", "KO", "KŠ", "LB", "LU", "NG", "NP", "PE", "PK", "PR", "RA", "RU",
    "SC", "SD", "SJ", "SP", "ST", "SV", "TO", "TR", "TS", "TT", "UB", "VB", "VL", "VS", "VŠ",
];

pub fn parse_and_normalize_license_plate(
    input: &str,
    country_hint: Option<&str>,
) -> Result<NormalizedLicensePlate> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "license_plate".to_string(),
            value: input.to_string(),
            reason: "License plate cannot be empty".to_string(),
        });
    }

    let upper = trimmed.to_uppercase();

    // Check characters: alphanumeric, hyphen, space
    let valid_chars = upper
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == ' ');
    if !valid_chars {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "license_plate".to_string(),
            value: input.to_string(),
            reason: "License plate contains invalid characters".to_string(),
        });
    }

    let compact: String = upper.chars().filter(|c| c.is_alphanumeric()).collect();

    if compact.len() < 2 || compact.len() > 12 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "license_plate".to_string(),
            value: input.to_string(),
            reason: format!(
                "License plate must contain between 2 and 12 characters, found {}",
                compact.len()
            ),
        });
    }

    let country = country_hint.map(|c| c.trim().to_uppercase());

    let mut region_code = None;
    let mut digits = None;
    let mut suffix = None;

    // Detect format if Serbia (RS) or Serbian pattern
    let is_serbia = country.as_deref() == Some("RS") || country.as_deref() == Some("SRB");

    if (is_serbia || country.is_none()) && compact.len() >= 7 && compact.len() <= 8 {
        // e.g. BG123AA or BG1234AA
        let prefix2 = &compact[..2];
        if SERBIA_REGION_CODES.contains(&prefix2) {
            let rest = &compact[2..];
            let num_digits = if compact.len() == 7 { 3 } else { 4 };
            let dig_slice = &rest[..num_digits];
            let suf_slice = &rest[num_digits..];

            if dig_slice.chars().all(|c| c.is_ascii_digit())
                && suf_slice.chars().all(|c| c.is_alphabetic())
            {
                region_code = Some(prefix2.to_string());
                digits = Some(dig_slice.to_string());
                suffix = Some(suf_slice.to_string());
            }
        }
    }

    Ok(NormalizedLicensePlate {
        raw: input.to_string(),
        cleaned: compact,
        country,
        region_code,
        digits,
        suffix,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serbian_license_plate() {
        let norm = parse_and_normalize_license_plate("BG 123-AA", Some("RS")).unwrap();
        assert_eq!(norm.cleaned, "BG123AA");
        assert_eq!(norm.country.as_deref(), Some("RS"));
        assert_eq!(norm.region_code.as_deref(), Some("BG"));
        assert_eq!(norm.digits.as_deref(), Some("123"));
        assert_eq!(norm.suffix.as_deref(), Some("AA"));
    }

    #[test]
    fn test_generic_license_plate() {
        let norm = parse_and_normalize_plate_generic("B-MW-1234").unwrap();
        assert_eq!(norm.cleaned, "BMW1234");
    }

    fn parse_and_normalize_plate_generic(input: &str) -> Result<NormalizedLicensePlate> {
        parse_and_normalize_license_plate(input, None)
    }
}
