use dorkforge::identifiers::{parse_identifier, IdentifierType};
use dorkforge::variants::{VariantConfig, VariantGenerator};

#[test]
fn test_variants_repeatability_and_determinism() {
    let id = parse_identifier(Some(IdentifierType::Name), "Михаил Булгаков", None, None).unwrap();

    let gen = VariantGenerator::new(VariantConfig::default());
    let run1 = gen.generate(&id);
    let run2 = gen.generate(&id);

    assert_eq!(run1, run2);
}

#[test]
fn test_no_duplicates_in_variants() {
    let id = parse_identifier(
        Some(IdentifierType::Domain),
        "https://example.com",
        None,
        None,
    )
    .unwrap();

    let gen = VariantGenerator::new(VariantConfig::default());
    let vars = gen.generate(&id);

    let mut set = std::collections::HashSet::new();
    for v in &vars {
        assert!(set.insert(v), "Duplicate variant found: {v}");
    }
}

#[test]
fn test_max_variants_limit() {
    let id = parse_identifier(
        Some(IdentifierType::Username),
        "very_long_user_name_test",
        None,
        None,
    )
    .unwrap();

    let gen = VariantGenerator::new(VariantConfig {
        max_variants_per_identifier: 2,
    });
    let vars = gen.generate(&id);
    assert_eq!(vars.len(), 2);
}

#[test]
fn test_preservation_of_original_value() {
    let raw_addr = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let id = parse_identifier(Some(IdentifierType::Wallet), raw_addr, None, Some("btc")).unwrap();

    let gen = VariantGenerator::new(VariantConfig::default());
    let vars = gen.generate(&id);
    assert_eq!(vars[0], raw_addr);
}

#[test]
fn test_license_plate_variants() {
    let id = parse_identifier(
        Some(IdentifierType::LicensePlate),
        "BG 123-AA",
        Some("RS"),
        None,
    )
    .unwrap();

    let gen = VariantGenerator::new(VariantConfig::default());
    let vars = gen.generate(&id);

    assert!(vars.contains(&"BG123AA".to_string()));
    assert!(vars.contains(&"BG 123 AA".to_string()));
    assert!(vars.contains(&"BG-123-AA".to_string()));
}

#[test]
fn test_new_types_variants() {
    let gen = VariantGenerator::new(VariantConfig::default());

    // Company
    let comp = parse_identifier(
        Some(IdentifierType::Company),
        "Acme Corporation LLC",
        None,
        None,
    )
    .unwrap();
    let comp_vars = gen.generate(&comp);
    assert!(comp_vars.contains(&"Acme".to_string()));
    assert!(comp_vars.contains(&"Acme LLC".to_string()));

    // VIN
    let vin = parse_identifier(Some(IdentifierType::Vin), "1HGCR2F83HA000000", None, None).unwrap();
    let vin_vars = gen.generate(&vin);
    assert!(vin_vars.contains(&"1HG CR2F83 HA000000".to_string()));

    // ASN
    let asn = parse_identifier(Some(IdentifierType::Asn), "AS15169", None, None).unwrap();
    let asn_vars = gen.generate(&asn);
    assert!(asn_vars.contains(&"AS15169".to_string()));
    assert!(asn_vars.contains(&"15169".to_string()));
    assert!(asn_vars.contains(&"AS 15169".to_string()));

    // Certificate
    let cert = parse_identifier(
        Some(IdentifierType::Certificate),
        "3BA06B7CF4A81244891000112233445566778899",
        None,
        None,
    )
    .unwrap();
    let cert_vars = gen.generate(&cert);
    assert!(cert_vars.contains(&"3BA06B7CF4A81244891000112233445566778899".to_string()));
    assert!(cert_vars
        .contains(&"3B:A0:6B:7C:F4:A8:12:44:89:10:00:11:22:33:44:55:66:77:88:99".to_string()));
}
