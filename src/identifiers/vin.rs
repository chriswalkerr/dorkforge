use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedVin {
    pub raw: String,
    pub vin: String,
    pub wmi: String, // World Manufacturer Identifier (first 3 chars)
    pub vds: String, // Vehicle Descriptor Section (chars 4-9)
    pub vis: String, // Vehicle Identifier Section (chars 10-17)
}

pub fn parse_and_normalize_vin(input: &str) -> Result<NormalizedVin> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "vin".to_string(),
            value: input.to_string(),
            reason: "VIN cannot be empty".to_string(),
        });
    }

    let compact: String = trimmed
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if compact.len() != 17 {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "vin".to_string(),
            value: input.to_string(),
            reason: format!("VIN must be exactly 17 characters, found {}", compact.len()),
        });
    }

    // VIN characters standard: excludes I, O, Q
    for c in compact.chars() {
        if c == 'I' || c == 'O' || c == 'Q' {
            return Err(DorkForgeError::InvalidInputFormat {
                identifier_type: "vin".to_string(),
                value: input.to_string(),
                reason: format!("VIN cannot contain illegal character '{c}' (I, O, Q disallowed)"),
            });
        }
    }

    let wmi = compact[..3].to_string();
    let vds = compact[3..9].to_string();
    let vis = compact[9..].to_string();

    Ok(NormalizedVin {
        raw: input.to_string(),
        vin: compact,
        wmi,
        vds,
        vis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_vin() {
        let vin_str = "1HGCR2F83HA000000";
        let v = parse_and_normalize_vin(vin_str).unwrap();
        assert_eq!(v.vin, vin_str);
        assert_eq!(v.wmi, "1HG");
    }

    #[test]
    fn test_invalid_vin() {
        assert!(parse_and_normalize_vin("1HGCR2F83HA00000I").is_err()); // contains 'I'
        assert!(parse_and_normalize_vin("12345").is_err()); // too short
    }
}
