use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::quality::metadata::ValidationStatus;
use crate::quality::validation::{
    extract_placeholders, validate_url_format, DEFAULT_ALLOWED_PLACEHOLDERS,
};
use crate::templates::TemplateDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    Warning,
    Error,
}

impl LintSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintIssue {
    pub template_id: String,
    pub severity: LintSeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LintReport {
    pub total_templates: usize,
    pub passed_templates: usize,
    pub issues: Vec<LintIssue>,
    pub strict: bool,
}

impl LintReport {
    pub fn is_success(&self) -> bool {
        if self.strict {
            self.issues.is_empty()
        } else {
            !self
                .issues
                .iter()
                .any(|i| i.severity == LintSeverity::Error)
        }
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == LintSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == LintSeverity::Warning)
            .count()
    }
}

/// Lint a single template definition.
pub fn lint_template(template: &TemplateDefinition, strict: bool) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let id = &template.id;

    // 1. Template ID
    if id.trim().is_empty() {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: LintSeverity::Error,
            field: Some("id".to_string()),
            message: "Template id cannot be empty".to_string(),
        });
    }

    // 2. Providers
    if template.providers.is_empty() {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: LintSeverity::Error,
            field: Some("providers".to_string()),
            message: "Template must specify at least one provider".to_string(),
        });
    }

    // 3. Category
    if template.category.trim().is_empty() {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: LintSeverity::Error,
            field: Some("category".to_string()),
            message: "Template category cannot be empty".to_string(),
        });
    }

    // 4. Identifier types check
    if template.identifier_type.is_none() && template.identifier_types.is_empty() {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: LintSeverity::Error,
            field: Some("identifier_type".to_string()),
            message: "Template must specify either identifier_type or identifier_types".to_string(),
        });
    }

    // 5. Placeholders validation
    match extract_placeholders(&template.template) {
        Ok(placeholders) => {
            if placeholders.is_empty() {
                issues.push(LintIssue {
                    template_id: id.clone(),
                    severity: LintSeverity::Warning,
                    field: Some("template".to_string()),
                    message: "Template string contains no placeholders".to_string(),
                });
            } else {
                for ph in &placeholders {
                    let is_allowed = if template.is_combination() {
                        template.identifier_types.iter().any(|t| t.as_str() == ph)
                            || DEFAULT_ALLOWED_PLACEHOLDERS.contains(&ph.as_str())
                    } else {
                        DEFAULT_ALLOWED_PLACEHOLDERS.contains(&ph.as_str())
                    };

                    if !is_allowed {
                        issues.push(LintIssue {
                            template_id: id.clone(),
                            severity: LintSeverity::Error,
                            field: Some("template".to_string()),
                            message: format!("Unknown or unmapped placeholder '{ph}' in template"),
                        });
                    }
                }
            }
        }
        Err(e) => {
            issues.push(LintIssue {
                template_id: id.clone(),
                severity: LintSeverity::Error,
                field: Some("template".to_string()),
                message: format!("Invalid placeholder syntax: {e}"),
            });
        }
    }

    // 6. Quality Metadata: purpose, rationale, limitations
    if template
        .purpose
        .as_ref()
        .is_none_or(|p| p.trim().is_empty())
        && template
            .description
            .as_ref()
            .is_none_or(|d| d.trim().is_empty())
    {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: if strict {
                LintSeverity::Error
            } else {
                LintSeverity::Warning
            },
            field: Some("purpose".to_string()),
            message: "Missing 'purpose' (or 'description') explaining template goal".to_string(),
        });
    }

    if template
        .rationale
        .as_ref()
        .is_none_or(|r| r.trim().is_empty())
    {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: if strict {
                LintSeverity::Error
            } else {
                LintSeverity::Warning
            },
            field: Some("rationale".to_string()),
            message: "Missing 'rationale' explaining OSINT reasoning".to_string(),
        });
    }

    if template
        .limitations
        .as_ref()
        .is_none_or(|l| l.trim().is_empty())
    {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: if strict {
                LintSeverity::Error
            } else {
                LintSeverity::Warning
            },
            field: Some("limitations".to_string()),
            message: "Missing 'limitations' stating known search constraints".to_string(),
        });
    }

    // 7. Validation status
    let status = template.effective_validation_status();
    if status == ValidationStatus::Untested {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: if strict {
                LintSeverity::Error
            } else {
                LintSeverity::Warning
            },
            field: Some("validation_status".to_string()),
            message: "Template has validation status 'untested'".to_string(),
        });
    } else if status == ValidationStatus::Unsupported {
        issues.push(LintIssue {
            template_id: id.clone(),
            severity: LintSeverity::Error,
            field: Some("validation_status".to_string()),
            message: "Template has validation status 'unsupported' and should not be enabled"
                .to_string(),
        });
    }

    // 8. Official references URL check (offline)
    for ref_url in &template.official_references {
        if let Err(e) = validate_url_format(ref_url) {
            issues.push(LintIssue {
                template_id: id.clone(),
                severity: LintSeverity::Error,
                field: Some("official_references".to_string()),
                message: format!("Invalid official reference URL '{ref_url}': {e}"),
            });
        }
    }

    // 9. Operator requirements
    for req in &template.operator_requirements {
        let with_colon = format!("{req}:");
        for provider in &template.providers {
            let supported = provider.supports_operator(req)
                || provider.supports_operator(&with_colon)
                || provider.supports_operator(req.trim_end_matches(':'));

            if !supported {
                issues.push(LintIssue {
                    template_id: id.clone(),
                    severity: LintSeverity::Error,
                    field: Some("operator_requirements".to_string()),
                    message: format!(
                        "Operator requirement '{req}' is not supported by provider '{}'",
                        provider.id()
                    ),
                });
            }
        }
    }

    issues
}

/// Lint an entire set of templates.
pub fn lint_templates(templates: &[TemplateDefinition], strict: bool) -> LintReport {
    let mut report = LintReport {
        total_templates: templates.len(),
        passed_templates: 0,
        issues: Vec::new(),
        strict,
    };

    let mut seen_ids = HashSet::new();

    for t in templates {
        // Check duplicate IDs
        if !seen_ids.insert(t.id.clone()) {
            report.issues.push(LintIssue {
                template_id: t.id.clone(),
                severity: LintSeverity::Error,
                field: Some("id".to_string()),
                message: format!("Duplicate template ID: '{}'", t.id),
            });
        }

        let item_issues = lint_template(t, strict);
        if item_issues.is_empty() {
            report.passed_templates += 1;
        } else {
            report.issues.extend(item_issues);
        }
    }

    report
}
