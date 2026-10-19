use dorkforge::quality::{lint_template, lint_templates, validate_url_format, LintSeverity};
use dorkforge::templates::TemplateRegistry;

#[test]
fn test_url_syntax_validation() {
    assert!(validate_url_format("https://support.google.com/websearch/answer/2466433").is_ok());
    assert!(validate_url_format("http://help.bing.microsoft.com/").is_ok());
    assert!(validate_url_format("not_a_url").is_err());
    assert!(validate_url_format("https://invalid host/").is_err());
    assert!(validate_url_format("").is_err());
}

#[test]
fn test_lint_templates_report_embedded_catalog() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let report = lint_templates(registry.templates(), false);
    for issue in &report.issues {
        if issue.severity == LintSeverity::Error {
            println!(
                "LINT ERROR: {} [field: {:?}]: {}",
                issue.template_id, issue.field, issue.message
            );
        }
    }
    assert_eq!(report.error_count(), 0);
}

#[test]
fn test_lint_catches_invalid_operator_requirement() {
    let toml = r#"
    [[template]]
    id = "bad_operator_template"
    enabled = true
    identifier_type = "email"
    category = "mentions"
    providers = ["duckduckgo"]
    template = '"{value}"'
    purpose = "Test purpose"
    rationale = "Test rationale"
    limitations = "Test limitations"
    operator_requirements = ["completely_fake_operator"]
    validation_status = "untested"
    "#;

    let mut registry = TemplateRegistry::new();
    registry.parse_and_add_toml(toml, None).unwrap();

    let issues = lint_template(&registry.templates()[0], false);
    assert!(issues.iter().any(
        |i| i.severity == LintSeverity::Error && i.message.contains("completely_fake_operator")
    ));
}
