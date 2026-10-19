use dorkforge::combination::{CombinationIdentifierInput, CombinationSpec};
use dorkforge::generator::{DorkGenerator, GeneratorOptions};
use dorkforge::output::{format_json, format_txt, TxtMode};
use dorkforge::providers::Provider;
use dorkforge::templates::TemplateRegistry;

#[test]
fn test_combination_generation_username_and_domain() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::Bing],
        ..GeneratorOptions::default()
    };

    let generator = DorkGenerator::new(&registry, options);

    let combo = CombinationSpec {
        id: "user_domain_test".to_string(),
        identifiers: vec![
            CombinationIdentifierInput {
                r#type: "username".to_string(),
                value: "alice".to_string(),
            },
            CombinationIdentifierInput {
                r#type: "domain".to_string(),
                value: "example.org".to_string(),
            },
        ],
        categories: Some(vec!["public_profiles".to_string()]),
        providers: Some(vec!["google".to_string(), "bing".to_string()]),
    };

    let result = generator
        .generate_for_combinations(&[combo])
        .expect("Generation should succeed");

    assert!(!result.queries.is_empty());

    for query in &result.queries {
        assert_eq!(query.category, "public_profiles");
        assert!(query.query.to_lowercase().contains("alice"));
        assert!(query.query.contains("example.org"));
        assert!(query.identifiers.is_some());

        let trace = query.identifiers.as_ref().unwrap();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].r#type, "username");
        assert_eq!(trace[0].value, "alice");
        assert_eq!(trace[1].r#type, "domain");
        assert_eq!(trace[1].value, "example.org");
    }

    // JSON export includes identifiers field
    let json_str = format_json(&result, true).unwrap();
    assert!(json_str.contains("\"identifiers\": ["));
    assert!(json_str.contains("\"type\": \"username\""));
    assert!(json_str.contains("\"type\": \"domain\""));

    // TXT export renders queries cleanly
    let txt_str = format_txt(&result, TxtMode::Clean);
    assert!(!txt_str.trim().is_empty());
}

#[test]
fn test_combination_generation_name_and_company() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::DuckDuckGo],
        ..GeneratorOptions::default()
    };

    let generator = DorkGenerator::new(&registry, options);

    let combo = CombinationSpec {
        id: "person_company_test".to_string(),
        identifiers: vec![
            CombinationIdentifierInput {
                r#type: "name".to_string(),
                value: "Jane Doe".to_string(),
            },
            CombinationIdentifierInput {
                r#type: "company".to_string(),
                value: "Acme Corp".to_string(),
            },
        ],
        categories: Some(vec!["documents".to_string()]),
        providers: None,
    };

    let result = generator
        .generate_for_combinations(&[combo])
        .expect("Generation should succeed");

    assert!(!result.queries.is_empty());
    for query in &result.queries {
        let q_lower = query.query.to_lowercase();
        assert!(q_lower.contains("jane") && q_lower.contains("doe"));
        assert!(q_lower.contains("acme"));
        assert!(query.query.contains("filetype:pdf"));
    }
}

#[test]
fn test_combination_vin_and_plate() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let options = GeneratorOptions {
        providers: vec![Provider::Google],
        ..GeneratorOptions::default()
    };

    let generator = DorkGenerator::new(&registry, options);

    let combo = CombinationSpec {
        id: "auto_market_test".to_string(),
        identifiers: vec![
            CombinationIdentifierInput {
                r#type: "vin".to_string(),
                value: "1HGCR2F83HA000000".to_string(),
            },
            CombinationIdentifierInput {
                r#type: "license_plate".to_string(),
                value: "BG-123-AA".to_string(),
            },
        ],
        categories: Some(vec!["marketplaces".to_string()]),
        providers: None,
    };

    let result = generator.generate_for_combinations(&[combo]).unwrap();
    assert!(!result.queries.is_empty());
    for q in &result.queries {
        assert_eq!(q.category, "marketplaces");
        let q_upper = q.query.to_uppercase();
        assert!(q_upper.contains("1HG") && q_upper.contains("CR2F83"));
        assert!(q_upper.contains("BG") && q_upper.contains("123"));
    }
}

#[test]
fn test_cartesian_limits_strict_and_non_strict() {
    let spec = CombinationSpec {
        id: "limit_test".to_string(),
        identifiers: vec![
            CombinationIdentifierInput {
                r#type: "name".to_string(),
                value: "Алексей Иванов".to_string(),
            },
            CombinationIdentifierInput {
                r#type: "company".to_string(),
                value: "ООО Вектор".to_string(),
            },
        ],
        categories: None,
        providers: None,
    };

    let registry = TemplateRegistry::load_embedded_defaults().unwrap();

    // Strict mode with low limit triggers error
    let options_strict = GeneratorOptions {
        max_combination_variants: 10,
        max_variant_combinations: 1, // Will be exceeded
        strict: true,
        ..GeneratorOptions::default()
    };
    let generator_strict = DorkGenerator::new(&registry, options_strict);
    assert!(generator_strict
        .generate_for_combinations(std::slice::from_ref(&spec))
        .is_err());

    // Non-strict mode truncates and emits warning
    let options_non_strict = GeneratorOptions {
        max_combination_variants: 10,
        max_variant_combinations: 2,
        strict: false,
        ..GeneratorOptions::default()
    };
    let generator_non_strict = DorkGenerator::new(&registry, options_non_strict);
    let result = generator_non_strict
        .generate_for_combinations(&[spec])
        .unwrap();
    assert!(!result.stats.warnings.is_empty());
}
