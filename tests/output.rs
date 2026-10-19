use tempfile::tempdir;

use dorkforge::generator::{DorkGenerator, GeneratorOptions};
use dorkforge::identifiers::{parse_identifier, IdentifierType};
use dorkforge::output::{
    format_graph_dot, format_html, format_json, format_txt, OutputFormat, OutputWriter, TxtMode,
};
use dorkforge::providers::Provider;
use dorkforge::templates::TemplateRegistry;

#[test]
fn test_txt_and_json_unified_output() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id =
        parse_identifier(Some(IdentifierType::Email), "analyst@osint.org", None, None).unwrap();

    let options = GeneratorOptions {
        providers: vec![Provider::Google, Provider::DuckDuckGo],
        ..Default::default()
    };

    let generator = DorkGenerator::new(&registry, options);
    let result = generator.generate_for_identifiers(&[id]).unwrap();

    let clean_txt = format_txt(&result, TxtMode::Clean);
    let readable_txt = format_txt(&result, TxtMode::Readable);
    let json_output = format_json(&result, true).unwrap();

    assert!(!clean_txt.is_empty());
    assert!(readable_txt.contains("## Provider: GOOGLE"));
    assert!(json_output.contains("\"format_version\": \"1.0\""));

    // Verify both formats reflect the same queries count
    let json_parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    let queries_json = json_parsed["queries"].as_array().unwrap();
    let lines_clean: Vec<&str> = clean_txt.lines().filter(|l| !l.trim().is_empty()).collect();

    assert_eq!(queries_json.len(), lines_clean.len());
    assert_eq!(queries_json.len(), result.stats.total_queries);
}

#[test]
fn test_output_writer_to_dir_and_conflict() {
    let temp_dir = tempdir().unwrap();
    let dir_path = temp_dir.path();

    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Domain), "example.com", None, None).unwrap();
    let generator = DorkGenerator::new(&registry, GeneratorOptions::default());
    let result = generator.generate_for_identifiers(&[id]).unwrap();

    let writer = OutputWriter::new(OutputFormat::Both, TxtMode::Readable, true, false);
    let written = writer
        .write_to_destination(&result, Some(dir_path))
        .unwrap();

    assert_eq!(written.len(), 2);
    assert!(dir_path.join("dorks.txt").exists());
    assert!(dir_path.join("dorks.json").exists());

    // Conflict test: writing again with overwrite=false should fail
    let conflict_err = writer.write_to_destination(&result, Some(dir_path));
    assert!(conflict_err.is_err());

    // Overwrite=true should succeed
    let overwrite_writer = OutputWriter::new(OutputFormat::Both, TxtMode::Readable, true, true);
    assert!(overwrite_writer
        .write_to_destination(&result, Some(dir_path))
        .is_ok());
}

#[test]
fn test_html_and_dot_exports_include_clickable_search_urls() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Domain), "example.org", None, None).unwrap();
    let result = DorkGenerator::new(&registry, GeneratorOptions::default())
        .generate_for_identifiers(&[id])
        .unwrap();

    let html = format_html(&result);
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("href=\"https://"));
    assert!(html.contains("rel=\"noopener noreferrer\""));

    let dot = format_graph_dot(&result);
    assert!(dot.starts_with("digraph dorkforge"));
    assert!(dot.contains("URL=\"https://"));
}

#[test]
fn test_html_shows_validation_and_escapes_limitations_and_warnings() {
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let identifier = parse_identifier(Some(IdentifierType::Ip), "192.0.2.9", None, None).unwrap();
    let mut result = DorkGenerator::new(&registry, GeneratorOptions::default())
        .generate_for_identifiers(&[identifier])
        .unwrap();
    result.queries[0].limitations = Some("<script>alert(1)</script>".to_string());
    result.stats.warnings = vec!["<img src=x>".to_string()];

    let html = format_html(&result);
    assert!(html.contains("<th>Validation</th>"));
    assert!(html.contains("untested"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;img src=x&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
}

#[test]
fn test_multi_file_export_preflights_conflicts() {
    let temp_dir = tempdir().unwrap();
    let registry = TemplateRegistry::load_embedded_defaults().unwrap();
    let id = parse_identifier(Some(IdentifierType::Domain), "example.org", None, None).unwrap();
    let result = DorkGenerator::new(&registry, GeneratorOptions::default())
        .generate_for_identifiers(&[id])
        .unwrap();

    std::fs::write(temp_dir.path().join("dorks.json"), "existing").unwrap();
    let writer = OutputWriter::new(OutputFormat::Both, TxtMode::Readable, true, false);
    assert!(writer
        .write_to_destination(&result, Some(temp_dir.path()))
        .is_err());
    assert!(!temp_dir.path().join("dorks.txt").exists());
}
