use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{DorkForgeError, Result};
use crate::identifiers::IdentifierType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProfile {
    pub id: String,
    pub name: String,
    #[serde(default = "default_source_kind")]
    pub kind: String,
    pub domains: Vec<String>,
    pub categories: Vec<String>,
    pub supported_identifier_types: Vec<IdentifierType>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

fn default_source_kind() -> String {
    "source_group".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourcesConfigFile {
    #[serde(default)]
    pub source: Vec<SourceProfile>,
}

#[derive(Debug, Clone, Default)]
pub struct SourceRegistry {
    profiles: Vec<SourceProfile>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    pub fn profiles(&self) -> &[SourceProfile] {
        &self.profiles
    }

    pub fn register(&mut self, profile: SourceProfile) -> Result<()> {
        if profile.id.trim().is_empty() {
            return Err(DorkForgeError::ValidationError(
                "Source profile id cannot be empty".to_string(),
            ));
        }
        if self.profiles.iter().any(|p| p.id == profile.id) {
            return Err(DorkForgeError::ValidationError(format!(
                "Duplicate source profile ID: '{}'",
                profile.id
            )));
        }
        self.profiles.push(profile);
        Ok(())
    }

    pub fn parse_and_add_toml(
        &mut self,
        toml_str: &str,
        source_path: Option<&Path>,
    ) -> Result<usize> {
        let parsed: SourcesConfigFile =
            toml::from_str(toml_str).map_err(|e| DorkForgeError::ConfigError {
                path: source_path.map(|p| p.to_path_buf()).unwrap_or_default(),
                message: format!("Failed to parse TOML sources file: {e}"),
            })?;

        let mut count = 0;
        for p in parsed.source {
            self.register(p)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn load_embedded_defaults() -> Result<Self> {
        let mut reg = Self::new();
        let sources_toml = include_str!("../../config/sources.toml");
        reg.parse_and_add_toml(sources_toml, None)?;
        Ok(reg)
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<usize> {
        if !path.exists() {
            return Err(DorkForgeError::ConfigError {
                path: path.to_path_buf(),
                message: "Sources configuration file does not exist".to_string(),
            });
        }
        let content = fs::read_to_string(path).map_err(|e| DorkForgeError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.parse_and_add_toml(&content, Some(path))
    }
}
