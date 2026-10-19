use std::net::IpAddr;
use std::str::FromStr;

use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IpVersion {
    V4,
    V6,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedIp {
    pub raw: String,
    pub ip: IpAddr,
    pub canonical: String,
    pub version: IpVersion,
}

pub fn parse_and_normalize_ip(input: &str) -> Result<NormalizedIp> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "ip".to_string(),
            value: input.to_string(),
            reason: "IP address cannot be empty".to_string(),
        });
    }

    // Strip port if present in IPv4 (e.g. 192.168.1.1:8080) or [::1]:8080
    let ip_str = if trimmed.starts_with('[') && trimmed.contains(']') {
        let end = trimmed.find(']').unwrap();
        &trimmed[1..end]
    } else if trimmed.matches(':').count() == 1 {
        // likely ipv4:port
        trimmed.split(':').next().unwrap()
    } else {
        trimmed
    };

    match IpAddr::from_str(ip_str) {
        Ok(ip) => {
            let version = match ip {
                IpAddr::V4(_) => IpVersion::V4,
                IpAddr::V6(_) => IpVersion::V6,
            };
            let canonical = ip.to_string();
            Ok(NormalizedIp {
                raw: input.to_string(),
                ip,
                canonical,
                version,
            })
        }
        Err(err) => Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "ip".to_string(),
            value: input.to_string(),
            reason: format!("Failed to parse IP address: {err}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ipv4() {
        let norm = parse_and_normalize_ip(" 192.168.1.1 ").unwrap();
        assert_eq!(norm.canonical, "192.168.1.1");
        assert_eq!(norm.version, IpVersion::V4);
    }

    #[test]
    fn test_valid_ipv6() {
        let norm = parse_and_normalize_ip("2001:0db8:0000:0000:0000:ff00:0042:8329").unwrap();
        assert_eq!(norm.canonical, "2001:db8::ff00:42:8329");
        assert_eq!(norm.version, IpVersion::V6);
    }

    #[test]
    fn test_invalid_ip() {
        assert!(parse_and_normalize_ip("").is_err());
        assert!(parse_and_normalize_ip("256.256.256.256").is_err());
        assert!(parse_and_normalize_ip("1.2.3").is_err());
        assert!(parse_and_normalize_ip("google.com").is_err());
    }
}
