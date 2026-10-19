use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{DorkForgeError, Result};
use crate::identifiers::IdentifierType;
use crate::providers::Provider;
use crate::quality::{ProviderValidationRecord, ValidationStatus};
use crate::templates::validation::{
    check_duplicate_template_ids, validate_category_for_type, validate_placeholders,
    validate_template_for_providers, ALLOWED_PLACEHOLDERS,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateDefinition {
    pub id: String,
    pub enabled: bool,
    #[serde(default)]
    pub identifier_type: Option<IdentifierType>,
    #[serde(default)]
    pub identifier_types: Vec<IdentifierType>,
    pub category: String,
    pub providers: Vec<Provider>,
    pub template: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default, deserialize_with = "deserialize_limitations")]
    pub limitations: Option<String>,
    #[serde(default)]
    pub source_domains: Vec<String>,
    #[serde(default)]
    pub operator_requirements: Vec<String>,
    #[serde(default)]
    pub official_references: Vec<String>,
    #[serde(default)]
    pub template_version: Option<u32>,
    #[serde(default)]
    pub validation_status: Option<ValidationStatus>,
    #[serde(default, deserialize_with = "deserialize_validation")]
    pub validation: TemplateValidation,
    #[serde(default)]
    pub provider_validation: Vec<ProviderValidationRecord>,
}

impl TemplateDefinition {
    pub fn is_combination(&self) -> bool {
        !self.identifier_types.is_empty()
    }

    pub fn primary_identifier_type(&self) -> Option<IdentifierType> {
        self.identifier_type
            .or_else(|| self.identifier_types.first().copied())
    }

    pub fn effective_purpose(&self) -> Option<&str> {
        self.purpose.as_deref().or(self.description.as_deref())
    }

    pub fn effective_validation_status(&self) -> ValidationStatus {
        if let Some(status) = self.validation_status {
            status
        } else {
            self.validation.status
        }
    }

    pub fn limitations_list(&self) -> Vec<String> {
        match &self.limitations {
            Some(lim) => lim
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            None => Vec::new(),
        }
    }
}

fn deserialize_limitations<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LimitationsInput {
        Single(String),
        Multiple(Vec<String>),
    }

    Ok(
        match Option::<LimitationsInput>::deserialize(deserializer)? {
            Some(LimitationsInput::Single(s)) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            Some(LimitationsInput::Multiple(v)) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v.join("; "))
                }
            }
            None => None,
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TemplateValidation {
    #[serde(default)]
    pub status: ValidationStatus,
    #[serde(default)]
    pub checked_at: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

fn deserialize_validation<'de, D>(
    deserializer: D,
) -> std::result::Result<TemplateValidation, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ValidationInput {
        Status(ValidationStatus),
        Details(TemplateValidation),
    }

    Ok(match ValidationInput::deserialize(deserializer)? {
        ValidationInput::Status(status) => TemplateValidation {
            status,
            ..TemplateValidation::default()
        },
        ValidationInput::Details(details) => details,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfigFile {
    #[serde(default)]
    pub template: Vec<TemplateDefinition>,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateRegistry {
    templates: Vec<TemplateDefinition>,
}

/// Normalise a user-supplied category alias to the built-in canonical vocabulary.
pub fn normalize_category(cat: &str) -> String {
    match cat.to_ascii_lowercase().as_str() {
        "documents" | "public_documents" => "documents".to_string(),
        "profiles" | "public_profiles" => "public_profiles".to_string(),
        "code" | "code_repositories" => "code_repositories".to_string(),
        "archives" | "historical_pages" => "archives".to_string(),
        "company" | "company_records" => "company_records".to_string(),
        "security" | "security_reports" => "security_reports".to_string(),
        "tech" | "technology" | "technology_fingerprint" => "technology_fingerprint".to_string(),
        other => other.to_string(),
    }
}

/// Returns true if a user-requested category matches the stored template category,
/// including alias expansion on both sides.
pub fn category_matches(requested: &str, template_category: &str) -> bool {
    if requested.eq_ignore_ascii_case(template_category) {
        return true;
    }
    normalize_category(requested) == normalize_category(template_category)
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    pub fn templates(&self) -> &[TemplateDefinition] {
        &self.templates
    }

    pub fn register(&mut self, template: TemplateDefinition) -> Result<()> {
        if !template.enabled {
            return Ok(());
        }

        // 1. Validate placeholders
        let mut allowed: Vec<&str> = ALLOWED_PLACEHOLDERS.to_vec();
        let custom_names: Vec<String> = template
            .identifier_types
            .iter()
            .map(|t| t.as_str().to_string())
            .collect();
        for s in &custom_names {
            allowed.push(s.as_str());
        }
        validate_placeholders(&template.id, &template.template, &allowed)?;

        // 2. Validate category for identifier type
        if let Some(id_type) = template.identifier_type {
            validate_category_for_type(&template.id, id_type, &template.category)?;
        }

        // 3. Validate providers support the operators in this template
        validate_template_for_providers(&template.id, &template.template, &template.providers)?;

        // 4. Duplicate ID check
        if self.templates.iter().any(|t| t.id == template.id) {
            return Err(DorkForgeError::InvalidTemplate {
                id: template.id.clone(),
                message: "Duplicate template ID in registry".to_string(),
            });
        }

        self.templates.push(template);
        Ok(())
    }

    pub fn parse_and_add_toml(
        &mut self,
        toml_str: &str,
        source_path: Option<&Path>,
    ) -> Result<usize> {
        let parsed: TemplateConfigFile =
            toml::from_str(toml_str).map_err(|e| DorkForgeError::ConfigError {
                path: source_path.map(|p| p.to_path_buf()).unwrap_or_default(),
                message: format!("Failed to parse TOML template file: {e}"),
            })?;

        // Check duplicates within the file itself
        check_duplicate_template_ids(parsed.template.iter().map(|t| t.id.as_str()))?;

        let mut count = 0;
        for t in parsed.template {
            if t.enabled {
                self.register(t)?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn load_from_dir(&mut self, dir_path: &Path) -> Result<usize> {
        if !dir_path.exists() {
            return Err(DorkForgeError::ConfigError {
                path: dir_path.to_path_buf(),
                message: "Configuration directory does not exist".to_string(),
            });
        }

        let entries = fs::read_dir(dir_path).map_err(|e| DorkForgeError::IoError {
            path: dir_path.to_path_buf(),
            source: e,
        })?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| DorkForgeError::IoError {
                path: dir_path.to_path_buf(),
                source: e,
            })?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                paths.push(path);
            }
        }

        paths.sort();
        let mut count = 0;
        for path in paths {
            let content = fs::read_to_string(&path).map_err(|e| DorkForgeError::IoError {
                path: path.clone(),
                source: e,
            })?;
            count += self.parse_and_add_toml(&content, Some(&path))?;
        }

        Ok(count)
    }

    /// Loads built-in embedded templates covering all 16 identifier types and 5 search engines.
    pub fn load_embedded_defaults() -> Result<Self> {
        let mut registry = Self::new();

        let google_toml = include_str!("../../config/templates/google.toml");
        let bing_toml = include_str!("../../config/templates/bing.toml");
        let yandex_toml = include_str!("../../config/templates/yandex.toml");
        let brave_toml = include_str!("../../config/templates/brave.toml");
        let duckduckgo_toml = include_str!("../../config/templates/duckduckgo.toml");
        let shared_templates_toml = include_str!("../../config/templates/shared_templates.toml");

        registry.parse_and_add_toml(google_toml, None)?;
        registry.parse_and_add_toml(bing_toml, None)?;
        registry.parse_and_add_toml(yandex_toml, None)?;
        registry.parse_and_add_toml(brave_toml, None)?;
        registry.parse_and_add_toml(duckduckgo_toml, None)?;
        registry.parse_and_add_toml(shared_templates_toml, None)?;

        Ok(registry)
    }

    /// Load embedded defaults first, then merge templates from an extra directory.
    pub fn load_embedded_with_extra(extra_dir: &Path) -> Result<Self> {
        let mut registry = Self::load_embedded_defaults()?;
        let count = registry.load_from_dir(extra_dir)?;
        if count == 0 {
            eprintln!(
                "[DorkForge] Warning: --extra-templates dir '{}' contained no valid templates",
                extra_dir.display()
            );
        }
        Ok(registry)
    }

    pub fn find_templates<'a>(
        &'a self,
        identifier_type: IdentifierType,
        provider: Provider,
        categories: Option<&[String]>,
    ) -> Vec<&'a TemplateDefinition> {
        self.templates
            .iter()
            .filter(|t| {
                if t.is_combination() {
                    return false;
                }
                if t.identifier_type != Some(identifier_type) {
                    return false;
                }
                if !t.providers.contains(&provider) {
                    return false;
                }
                if let Some(cats) = categories {
                    if !cats.iter().any(|c| category_matches(c, &t.category)) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn find_combination_templates<'a>(
        &'a self,
        types: &[IdentifierType],
        provider: Provider,
        categories: Option<&[String]>,
    ) -> Vec<&'a TemplateDefinition> {
        self.templates
            .iter()
            .filter(|t| {
                if !t.is_combination() {
                    return false;
                }
                if t.identifier_types.as_slice() != types {
                    return false;
                }
                if !t.providers.contains(&provider) {
                    return false;
                }
                if let Some(cats) = categories {
                    if !cats.iter().any(|c| category_matches(c, &t.category)) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Return all unique categories from loaded templates (dynamic, from TOML).
    pub fn all_categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .templates
            .iter()
            .map(|t| t.category.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        cats.sort();
        cats
    }
}
