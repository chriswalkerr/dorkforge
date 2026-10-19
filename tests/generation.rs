use dorkforge::generator::{DorkGenerator, GeneratorOptions};
use dorkforge::identifiers::{parse_identifier, IdentifierType};
use dorkforge::providers::Provider;
use dorkforge::templates::TemplateRegistry;

#[test]
fn test_generation_for_all_providers() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let email_id =
        parse_identifier(Some(IdentifierType::Email), "alex@testcorp.com", None, None).unwrap();

    let options = GeneratorOptions {
        providers: Provider::ALL.to_vec(),
        categories: None,
        max_variants: 5,
        max_queries_per_identifier: 50,
        max_queries_per_provider: 100,
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[email_id]).unwrap();

    assert!(result.stats.total_queries > 0);
    // Ensure all 5 providers have queries
    for p in Provider::ALL {
        let count = result
            .stats
            .queries_by_provider
            .get(p.id())
            .copied()
            .unwrap_or(0);
        assert!(count > 0, "Provider {} must have queries", p.id());
    }
}

#[test]
fn test_generation_category_filter() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let dom_id = parse_identifier(
        Some(IdentifierType::Domain),
        "security.example.org",
        None,
        None,
    )
    .unwrap();

    let options = GeneratorOptions {
        providers: vec![Provider::Google],
        categories: Some(vec!["domain_pages".to_string()]),
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[dom_id]).unwrap();

    assert!(!result.queries.is_empty());
    for q in &result.queries {
        assert_eq!(q.category, "domain_pages");
    }
}

#[test]
fn test_multiple_identifiers_batch() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id1 = parse_identifier(Some(IdentifierType::Email), "target@bank.com", None, None).unwrap();
    let id2 = parse_identifier(Some(IdentifierType::Ip), "8.8.8.8", None, None).unwrap();
    let id3 = parse_identifier(Some(IdentifierType::Domain), "bank.com", None, None).unwrap();

    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::Bing],
        categories: None,
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator
        .generate_for_identifiers(&[id1, id2, id3])
        .unwrap();

    assert_eq!(result.inputs.len(), 3);
    assert!(result.stats.total_queries >= 6);
}

#[test]
fn test_limits_warning() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Domain), "targetcorp.com", None, None).unwrap();

    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::Bing, Provider::Brave],
        max_queries_per_identifier: 2,
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[id]).unwrap();

    assert_eq!(result.queries.len(), 2);
    assert!(!result.stats.warnings.is_empty());
    assert!(result.stats.warnings[0].contains("reached maximum query limit"));
}

#[test]
fn test_legacy_category_alias_public_documents() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Email), "audit@corp.com", None, None).unwrap();

    let options = GeneratorOptions {
        providers: vec![Provider::Google],
        categories: Some(vec!["public_documents".to_string()]),
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[id]).unwrap();

    assert!(
        result.stats.total_queries > 0,
        "Legacy category 'public_documents' must match canonical documents templates"
    );
    assert!(result
        .queries
        .iter()
        .all(|query| query.category == "documents"));
}

#[test]
fn test_unique_query_ids() {
    // Every query_id must be unique across the full result set
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Vin), "1HGCR2F83HA000000", None, None).unwrap();

    let options = GeneratorOptions {
        providers: Provider::ALL.to_vec(),
        max_variants: 5,
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[id]).unwrap();

    let mut seen_ids = std::collections::HashSet::new();
    for q in &result.queries {
        assert!(
            seen_ids.insert(q.query_id.clone()),
            "Duplicate query_id found: {}",
            q.query_id
        );
        // query_id must contain provider
        assert!(
            q.query_id.contains(&q.provider),
            "query_id '{}' should contain provider name '{}'",
            q.query_id,
            q.provider
        );
        // fingerprint must be 8 chars hex
        assert_eq!(q.fingerprint.len(), 8);
        assert!(q.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn test_stats_distinguish_query_text_from_provider_query_records() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let identifier =
        parse_identifier(Some(IdentifierType::Email), "audit@example.org", None, None).unwrap();
    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::Bing],
        categories: Some(vec!["exact_match".to_string()]),
        max_variants: 1,
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[identifier]).unwrap();

    assert_eq!(result.stats.total_queries, 2);
    assert_eq!(result.stats.unique_query_texts, 1);
    assert_eq!(result.stats.unique_provider_queries, 2);
    assert_eq!(result.stats.templates_used, 2);
    assert_ne!(result.queries[0].record_id, result.queries[1].record_id);
    assert_eq!(result.queries[0].record_id.len(), 69);
    assert!(result
        .queries
        .iter()
        .all(|query| query.validation_status == dorkforge::templates::ValidationStatus::Untested));
}

#[test]
fn test_format_urls_output() {
    use dorkforge::output::format_urls;
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Email), "target@osint.io", None, None).unwrap();

    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::DuckDuckGo],
        categories: Some(vec!["exact_match".to_string()]),
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[id]).unwrap();

    let urls = format_urls(&result);
    assert!(!urls.is_empty());
    for line in urls.lines() {
        assert!(
            line.starts_with("https://"),
            "URL output must be HTTPS: {line}"
        );
        // Query must be percent-encoded — spaces become + or %20, " becomes %22
        assert!(
            !line.contains('"'),
            "URL output must not contain raw quotes: {line}"
        );
    }
}

#[test]
fn test_search_urls_encode_reserved_characters_and_unicode() {
    use dorkforge::output::format_urls;

    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let identifier =
        parse_identifier(Some(IdentifierType::TextPhrase), "A+B & Москва", None, None).unwrap();
    let options = GeneratorOptions {
        providers: vec![Provider::Google],
        categories: Some(vec!["exact_match".to_string()]),
        max_variants: 1,
        ..Default::default()
    };
    let result = DorkGenerator::new(&registry, options)
        .generate_for_identifiers(&[identifier])
        .unwrap();

    let urls = format_urls(&result);
    assert!(urls.contains("%2B"));
    assert!(urls.contains("%26"));
    assert!(urls.contains("%D0%9C"));
    assert!(!urls.contains('"'));
}
