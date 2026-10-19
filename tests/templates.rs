use dorkforge::error::DorkForgeError;
use dorkforge::identifiers::IdentifierType;
use dorkforge::providers::{OperatorTestStatus, Provider, ProvidersFile};
use dorkforge::templates::{TemplateConfigFile, TemplateRegistry};

#[test]
fn test_embedded_templates_loading() {
    let registry =
        TemplateRegistry::load_embedded_defaults().expect("Embedded templates must be valid");
    assert!(!registry.templates().is_empty());

    // Verify all supported identifier types have templates.
    for id_type in IdentifierType::ALL {
        let matching: Vec<_> = registry
            .templates()
            .iter()
            .filter(|t| t.identifier_type == Some(*id_type))
            .collect();
        assert!(
            !matching.is_empty(),
            "Identifier type {:?} must have at least one template",
            id_type
        );
    }
}

#[test]
fn test_extended_template_metadata_is_preserved() {
    let toml_str = r#"
    [[template]]
    id = "metadata_test"
    enabled = true
    identifier_type = "email"
    category = "exact_match"
    providers = ["google"]
    template = '"{value}"'
    note = "Legacy note"
    description = "Search public pages"
    rationale = "Find indexed mentions"
    limitations = "Not exhaustive"
    source_domains = ["example.org"]
    template_version = 2

    [template.validation]
    status = "partial"
    checked_at = "2026-10-16"
    comment = "Some results were not indexed"
    "#;

    let mut registry = TemplateRegistry::new();
    registry.parse_and_add_toml(toml_str, None).unwrap();
    let template = &registry.templates()[0];

    assert_eq!(template.note.as_deref(), Some("Legacy note"));
    assert_eq!(template.description.as_deref(), Some("Search public pages"));
    assert_eq!(template.rationale.as_deref(), Some("Find indexed mentions"));
    assert_eq!(template.limitations.as_deref(), Some("Not exhaustive"));
    assert_eq!(template.source_domains, ["example.org"]);
    assert_eq!(template.template_version, Some(2));
    assert_eq!(
        template.validation.status,
        dorkforge::templates::ValidationStatus::Partial
    );
    assert_eq!(
        template.validation.checked_at.as_deref(),
        Some("2026-10-16")
    );
}

#[test]
fn test_provider_operator_metadata_covers_supported_operators() {
    let providers: ProvidersFile =
        toml::from_str(include_str!("../config/providers.toml")).unwrap();

    for provider in providers.providers {
        for operator in &provider.supported_operators {
            let metadata = provider
                .operator_metadata
                .get(operator)
                .unwrap_or_else(|| panic!("{} operator {operator} has no metadata", provider.id));
            assert_eq!(metadata.test_status, OperatorTestStatus::Untested);
            assert!(!metadata.description.is_empty());
        }
        assert!(!provider.documentation_urls.is_empty(), "{}", provider.id);
        assert_eq!(provider.last_checked.as_deref(), Some("2026-10-16"));
    }
}

#[test]
fn test_runtime_provider_operators_match_catalog() {
    let providers: ProvidersFile =
        toml::from_str(include_str!("../config/providers.toml")).unwrap();

    for provider in providers.providers {
        let runtime_provider: Provider = provider.id.parse().unwrap();
        let config_operators: std::collections::HashSet<_> = provider
            .supported_operators
            .iter()
            .map(String::as_str)
            .collect();
        let runtime_operators: std::collections::HashSet<_> = runtime_provider
            .supported_operators()
            .iter()
            .copied()
            .collect();
        assert_eq!(config_operators, runtime_operators, "{}", provider.id);
        let runtime_categories: Vec<_> = runtime_provider
            .supported_categories()
            .iter()
            .map(|category| category.to_string())
            .collect();
        assert_eq!(
            provider.supported_categories, runtime_categories,
            "{}",
            provider.id
        );
        for operator in &provider.supported_operators {
            assert!(
                runtime_provider.supports_operator(operator),
                "{} config operator {operator} is missing from the runtime allowlist",
                provider.id
            );
        }
    }
}

#[test]
fn test_builtin_categories_use_controlled_vocabulary() {
    const BUILTIN_CATEGORIES: &[&str] = &[
        "exact_match",
        "mentions",
        "documents",
        "public_profiles",
        "forums",
        "code_repositories",
        "domain_pages",
        "subdomains",
        "archives",
        "public_records",
        "company_records",
        "security_reports",
        "technology_fingerprint",
        "marketplaces",
        "custom",
    ];

    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    for template in registry.templates() {
        assert!(
            BUILTIN_CATEGORIES.contains(&template.category.as_str()),
            "{} uses non-canonical category {}",
            template.id,
            template.category
        );
    }
}

#[test]
fn test_enabled_subdomain_templates_do_not_use_site_wildcards() {
    let catalogs = [
        include_str!("../config/templates/google.toml"),
        include_str!("../config/templates/bing.toml"),
        include_str!("../config/templates/yandex.toml"),
        include_str!("../config/templates/brave.toml"),
        include_str!("../config/templates/duckduckgo.toml"),
    ];

    for catalog in catalogs {
        let parsed: TemplateConfigFile = toml::from_str(catalog).unwrap();
        for template in parsed.template {
            if template.enabled && template.category == "subdomains" {
                assert!(
                    !template.template.contains("site:*."),
                    "{} relies on an unverified site wildcard",
                    template.id
                );
            }
        }
    }
}

#[test]
fn test_shared_templates_are_loaded_for_all_providers() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let shared_templates: Vec<_> = registry
        .templates()
        .iter()
        .filter(|template| template.id.starts_with("shared_"))
        .collect();

    assert_eq!(shared_templates.len(), 3);
    for template in shared_templates {
        assert_eq!(template.providers.len(), 5, "{}", template.id);
        for provider in &template.providers {
            assert!(
                !registry.templates().iter().any(|provider_template| {
                    !provider_template.id.starts_with("shared_")
                        && provider_template.identifier_type == template.identifier_type
                        && provider_template.providers.contains(provider)
                        && provider_template.template == template.template
                }),
                "{} duplicates a provider catalog template for {}",
                template.id,
                provider
            );
        }
    }
}

#[test]
fn test_unknown_placeholder_error() {
    let toml_str = r#"
    [[template]]
    id = "invalid_placeholder_test"
    enabled = true
    identifier_type = "email"
    category = "exact_match"
    providers = ["google"]
    template = '"{value}" {unknown_variable}'
    tags = ["test"]
    "#;

    let mut registry = TemplateRegistry::new();
    let res = registry.parse_and_add_toml(toml_str, None);
    assert!(res.is_err());
    match res.unwrap_err() {
        DorkForgeError::UnknownPlaceholder { placeholder, .. } => {
            assert_eq!(placeholder, "unknown_variable");
        }
        other => panic!("Expected UnknownPlaceholder, got {other:?}"),
    }
}

#[test]
fn test_duplicate_template_ids() {
    let toml_str = r#"
    [[template]]
    id = "dup_id"
    enabled = true
    identifier_type = "email"
    category = "exact_match"
    providers = ["google"]
    template = '"{value}"'
    tags = ["test"]

    [[template]]
    id = "dup_id"
    enabled = true
    identifier_type = "email"
    category = "exact_match"
    providers = ["bing"]
    template = '"{value}"'
    tags = ["test"]
    "#;

    let mut registry = TemplateRegistry::new();
    let res = registry.parse_and_add_toml(toml_str, None);
    assert!(res.is_err());
}

#[test]
fn test_disabled_template_is_ignored() {
    let toml_str = r#"
    [[template]]
    id = "disabled_test"
    enabled = false
    identifier_type = "email"
    category = "exact_match"
    providers = ["google"]
    template = '"{value}"'
    tags = ["test"]
    "#;

    let mut registry = TemplateRegistry::new();
    let count = registry.parse_and_add_toml(toml_str, None).unwrap();
    assert_eq!(count, 0);
    assert!(registry.templates().is_empty());
}

#[test]
fn test_unsupported_operator_for_provider() {
    // Yandex does not support Google-specific `filetype:` operator
    let toml_str = r#"
    [[template]]
    id = "unsupported_op_test"
    enabled = true
    identifier_type = "email"
    category = "documents"
    providers = ["yandex"]
    template = '"{value}" filetype:pdf'
    tags = ["test"]
    "#;

    let mut registry = TemplateRegistry::new();
    let res = registry.parse_and_add_toml(toml_str, None);
    assert!(res.is_err());
}

#[test]
fn test_unsupported_symbol_operator_for_provider() {
    let toml_str = r#"
    [[template]]
    id = "unsupported_symbol_op_test"
    enabled = true
    identifier_type = "email"
    category = "exact_match"
    providers = ["google"]
    template = '"{value}" +required'
    tags = ["test"]
    "#;

    let mut registry = TemplateRegistry::new();
    assert!(registry.parse_and_add_toml(toml_str, None).is_err());
}

#[test]
fn test_invalid_category_for_identifier_type() {
    // `subdomains` is only for domains, not for email
    let toml_str = r#"
    [[template]]
    id = "wrong_cat_test"
    enabled = true
    identifier_type = "email"
    category = "subdomains"
    providers = ["google"]
    template = 'site:{value}'
    tags = ["test"]
    "#;

    let mut registry = TemplateRegistry::new();
    let res = registry.parse_and_add_toml(toml_str, None);
    assert!(res.is_err());
}
