use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{DorkForgeError, Result};

/// Verification status of a template or operator.
///
/// In DorkForge 1.3.0, statuses formalize both documentation availability
/// and empirical manual checking (without network access).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// Not yet verified manually.
    #[default]
    Untested,
    /// Documented in official search engine documentation.
    Documented,
    /// Verified manually against the search engine interface.
    ManuallyVerified,
    /// Works with documented limitations.
    Limited,
    /// Operator or pattern is not supported by the provider.
    Unsupported,
    /// Deprecated pattern; kept for compatibility.
    Deprecated,
    /// Automated test status: passed syntax and operator checks.
    Passed,
    /// Automated test status: partially supported.
    Partial,
    /// Automated test status: failed checks.
    Failed,
    /// Status check is outdated.
    Stale,
}

impl ValidationStatus {
    pub const ALL: &'static [ValidationStatus] = &[
        ValidationStatus::Untested,
        ValidationStatus::Documented,
        ValidationStatus::ManuallyVerified,
        ValidationStatus::Limited,
        ValidationStatus::Unsupported,
        ValidationStatus::Deprecated,
        ValidationStatus::Passed,
        ValidationStatus::Partial,
        ValidationStatus::Failed,
        ValidationStatus::Stale,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Untested => "untested",
            Self::Documented => "documented",
            Self::ManuallyVerified => "manually_verified",
            Self::Limited => "limited",
            Self::Unsupported => "unsupported",
            Self::Deprecated => "deprecated",
            Self::Passed => "passed",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Stale => "stale",
        }
    }

    /// Returns true if this status allows generating queries in non-forced mode.
    pub const fn is_runnable(self) -> bool {
        !matches!(self, Self::Unsupported)
    }

    /// Returns true if the template has formal documentation or verification.
    pub const fn is_verified_or_documented(self) -> bool {
        matches!(
            self,
            Self::Documented | Self::ManuallyVerified | Self::Passed
        )
    }
}

impl fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ValidationStatus {
    type Err = DorkForgeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "untested" => Ok(Self::Untested),
            "documented" => Ok(Self::Documented),
            "manually_verified" | "verified" => Ok(Self::ManuallyVerified),
            "limited" => Ok(Self::Limited),
            "unsupported" => Ok(Self::Unsupported),
            "deprecated" => Ok(Self::Deprecated),
            "passed" => Ok(Self::Passed),
            "partial" => Ok(Self::Partial),
            "failed" => Ok(Self::Failed),
            "stale" => Ok(Self::Stale),
            other => Err(DorkForgeError::ValidationError(format!(
                "Invalid validation status: '{other}'. Choose from: untested, documented, manually_verified, limited, unsupported, deprecated"
            ))),
        }
    }
}

/// Metadata record documenting manual verification of a template for a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderValidationRecord {
    pub provider: String,
    pub status: ValidationStatus,
    #[serde(default)]
    pub checked_on: Option<String>,
    #[serde(default)]
    pub checked_by: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_status_str_roundtrip() {
        for status in ValidationStatus::ALL {
            let s = status.as_str();
            let parsed = ValidationStatus::from_str(s).unwrap();
            assert_eq!(*status, parsed);
        }
    }

    #[test]
    fn test_invalid_validation_status() {
        assert!(ValidationStatus::from_str("completely_unknown").is_err());
    }
}
