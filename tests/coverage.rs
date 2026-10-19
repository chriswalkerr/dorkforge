use dorkforge::coverage::{
    format_coverage_json, format_coverage_txt, CoverageFilter, CoverageMatrix, SourceRegistry,
};
use dorkforge::templates::TemplateRegistry;

#[test]
fn test_source_profiles_loading() {
    let registry = SourceRegistry::load_embedded_defaults().expect("Embedded sources must load");
    assert!(!registry.profiles().is_empty());
    assert!(registry.profiles().len() >= 5);

    let code_hosts = registry
        .profiles()
        .iter()
        .find(|p| p.id == "public_code_hosts")
        .expect("public_code_hosts profile must exist");
    assert!(code_hosts.domains.contains(&"github.com".to_string()));
    assert!(code_hosts
        .categories
        .contains(&"code_repositories".to_string()));
}

#[test]
fn test_coverage_matrix_calculation_and_filtering() {
    let templates = TemplateRegistry::load_embedded_defaults().unwrap();
    let sources = SourceRegistry::load_embedded_defaults().unwrap();

    // 1. Unfiltered matrix
    let matrix_all = CoverageMatrix::build(&templates, &sources, CoverageFilter::default());
    assert!(matrix_all.total_templates > 0);
    assert!(!matrix_all.category_by_provider.is_empty());
    assert!(!matrix_all.type_by_category.is_empty());
    assert!(!matrix_all.combination_by_category.is_empty());
    assert!(!matrix_all.source_by_provider.is_empty());
    assert!(!matrix_all.operator_status_summary.is_empty());

    // 2. Filter by provider
    let matrix_google = CoverageMatrix::build(
        &templates,
        &sources,
        CoverageFilter {
            provider: Some("google".to_string()),
            category: None,
            r#type: None,
        },
    );
    for p_map in matrix_google.category_by_provider.values() {
        assert_eq!(p_map.len(), 1);
        assert!(p_map.contains_key("google"));
    }

    // 3. Format as TXT
    let txt = format_coverage_txt(&matrix_all);
    assert!(txt.contains("DORKFORGE SOURCE COVERAGE MATRIX"));
    assert!(txt.contains("CATEGORY x PROVIDER MATRIX"));
    assert!(txt.contains("SOURCE PROFILES x PROVIDER COVERAGE"));

    // 4. Format as JSON
    let json = format_coverage_json(&matrix_all, true).unwrap();
    assert!(json.contains("\"total_templates\":"));
    assert!(json.contains("\"category_by_provider\":"));
}
