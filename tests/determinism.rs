use dorkforge::generator::{DorkGenerator, GeneratorOptions};
use dorkforge::identifiers::{parse_identifier, IdentifierType};
use dorkforge::output::{format_json, format_txt, TxtMode};
use dorkforge::providers::Provider;
use dorkforge::templates::TemplateRegistry;

#[test]
fn test_strict_determinism() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let email = parse_identifier(
        Some(IdentifierType::Email),
        "target.investigation@bank.com",
        None,
        None,
    )
    .unwrap();
    let domain = parse_identifier(Some(IdentifierType::Domain), "bank.com", None, None).unwrap();
    let name = parse_identifier(Some(IdentifierType::Name), "Johnathan Doe", None, None).unwrap();

    let options = GeneratorOptions {
        providers: Provider::ALL.to_vec(),
        categories: None,
        max_variants: 5,
        max_queries_per_identifier: 30,
        max_queries_per_provider: 50,
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);

    let run1 = generator
        .generate_for_identifiers(&[email.clone(), domain.clone(), name.clone()])
        .unwrap();
    let run2 = generator
        .generate_for_identifiers(&[email.clone(), domain.clone(), name.clone()])
        .unwrap();

    // 1. Structure equality
    assert_eq!(run1.queries, run2.queries);
    assert_eq!(run1.stats, run2.stats);

    // 2. Exact string output equality
    let txt1 = format_txt(&run1, TxtMode::Readable);
    let txt2 = format_txt(&run2, TxtMode::Readable);
    assert_eq!(txt1, txt2);

    let json1 = format_json(&run1, true).unwrap();
    let json2 = format_json(&run2, true).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn test_no_timestamps_in_query_ids() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Domain), "target.com", None, None).unwrap();
    let generator = DorkGenerator::new(&registry, GeneratorOptions::default());
    let res = generator.generate_for_identifiers(&[id]).unwrap();

    for q in &res.queries {
        // Query IDs should be template IDs, without timestamp markers
        assert!(!q.id.contains("2026"));
        assert!(!q.id.contains("timestamp"));
        assert!(!q.id.contains("date"));
    }
}
