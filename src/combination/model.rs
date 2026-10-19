use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::identifiers::{IdentifierType, ParsedIdentifier};

/// Input identifier within a combination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombinationIdentifierInput {
    pub r#type: String,
    pub value: String,
}

/// Specification of a multi-identifier combination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombinationSpec {
    pub id: String,
    pub identifiers: Vec<CombinationIdentifierInput>,
    #[serde(default)]
    pub categories: Option<Vec<String>>,
    #[serde(default)]
    pub providers: Option<Vec<String>>,
}

/// Wrapper for batch JSON inputs containing combinations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CombinationsFile {
    #[serde(default)]
    pub combinations: Vec<CombinationSpec>,
}

/// A validated, normalized identifier with its generated variants.
#[derive(Debug, Clone)]
pub struct ResolvedCombinationIdentifier {
    pub id_type: IdentifierType,
    pub original_value: String,
    pub parsed: ParsedIdentifier,
    pub variants: Vec<String>,
}

/// A specific bound set of variants for one rendering instance of a combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombinationVariantBinding {
    pub values_by_type: BTreeMap<String, String>,
    pub variants_by_type: BTreeMap<String, String>,
}
