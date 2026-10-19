use dorkforge::identifiers::wallet::WalletFamily;
use dorkforge::identifiers::{
    auto_detect_type, parse_identifier, IdentifierType, ParsedIdentifier,
};

#[test]
fn test_email_validation() {
    let valid = parse_identifier(
        Some(IdentifierType::Email),
        "test.user@example.com",
        None,
        None,
    )
    .unwrap();
    assert_eq!(valid.canonical_value(), "test.user@example.com");

    let invalid = parse_identifier(Some(IdentifierType::Email), "invalid-email@", None, None);
    assert!(invalid.is_err());
}

#[test]
fn test_ip_validation() {
    let v4 = parse_identifier(Some(IdentifierType::Ip), "127.0.0.1", None, None).unwrap();
    assert_eq!(v4.canonical_value(), "127.0.0.1");

    let v6 = parse_identifier(Some(IdentifierType::Ip), "2001:0db8::1", None, None).unwrap();
    assert_eq!(v6.canonical_value(), "2001:db8::1");

    let invalid = parse_identifier(Some(IdentifierType::Ip), "999.999.999.999", None, None);
    assert!(invalid.is_err());
}

#[test]
fn test_domain_validation() {
    let d = parse_identifier(
        Some(IdentifierType::Domain),
        "https://sub.Example.COM/path",
        None,
        None,
    )
    .unwrap();
    assert_eq!(d.canonical_value(), "sub.example.com");

    let invalid = parse_identifier(Some(IdentifierType::Domain), "-invalid-.com", None, None);
    assert!(invalid.is_err());
}

#[test]
fn test_empty_string_rejection() {
    assert!(parse_identifier(Some(IdentifierType::Username), "", None, None).is_err());
    assert!(parse_identifier(Some(IdentifierType::Email), "   ", None, None).is_err());
}

#[test]
fn test_unicode_and_cyrillic() {
    let name = parse_identifier(Some(IdentifierType::Name), "Иван Иванов", None, None).unwrap();
    assert_eq!(name.canonical_value(), "Иван Иванов");
}

#[test]
fn test_case_sensitive_crypto_wallet() {
    let btc = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    let wallet = parse_identifier(Some(IdentifierType::Wallet), btc, None, Some("btc")).unwrap();
    assert_eq!(wallet.canonical_value(), btc);

    if let ParsedIdentifier::Wallet(w) = wallet {
        assert_eq!(w.family, WalletFamily::BitcoinP2PKH);
        assert_eq!(w.network_hint.as_deref(), Some("btc"));
    } else {
        panic!("Expected wallet");
    }
}

#[test]
fn test_phone_formatting() {
    let phone1 =
        parse_identifier(Some(IdentifierType::Phone), "+1 (555) 019-2834", None, None).unwrap();
    assert_eq!(phone1.canonical_value(), "+15550192834");

    let phone2 = parse_identifier(
        Some(IdentifierType::Phone),
        "555-019-2834",
        Some("+1"),
        None,
    )
    .unwrap();
    assert_eq!(phone2.canonical_value(), "+15550192834");
}

#[test]
fn test_license_plates() {
    let plate_rs = parse_identifier(
        Some(IdentifierType::LicensePlate),
        "bg-123-aa",
        Some("RS"),
        None,
    )
    .unwrap();
    assert_eq!(plate_rs.canonical_value(), "BG123AA");

    let plate_generic =
        parse_identifier(Some(IdentifierType::LicensePlate), "AB-123-CD", None, None).unwrap();
    assert_eq!(plate_generic.canonical_value(), "AB123CD");
}

#[test]
fn test_auto_detection_and_ambiguity() {
    // Unambiguous email
    assert_eq!(
        auto_detect_type("alice@domain.org").unwrap(),
        IdentifierType::Email
    );

    // Unambiguous IP
    assert_eq!(auto_detect_type("192.168.1.1").unwrap(), IdentifierType::Ip);

    // Unambiguous EVM wallet
    assert_eq!(
        auto_detect_type("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap(),
        IdentifierType::Wallet
    );

    // Ambiguous word must NOT guess silently
    assert!(auto_detect_type("alexander").is_err());
    assert!(auto_detect_type("target_value").is_err());
}

#[test]
fn test_new_identifier_types() {
    // Company
    let comp = parse_identifier(
        Some(IdentifierType::Company),
        "Acme Corporation LLC",
        None,
        None,
    )
    .unwrap();
    assert_eq!(comp.canonical_value(), "Acme");

    // Person ID
    let pid = parse_identifier(
        Some(IdentifierType::PersonId),
        "7707083893",
        Some("RU"),
        None,
    )
    .unwrap();
    assert_eq!(pid.canonical_value(), "7707083893");

    // URL
    let url = parse_identifier(
        Some(IdentifierType::Url),
        "https://site.org/docs/report.pdf",
        None,
        None,
    )
    .unwrap();
    assert_eq!(url.canonical_value(), "site.org/docs/report.pdf");

    // VIN
    let vin = parse_identifier(Some(IdentifierType::Vin), "1HGCR2F83HA000000", None, None).unwrap();
    assert_eq!(vin.canonical_value(), "1HGCR2F83HA000000");

    // File Hash
    let hash = parse_identifier(
        Some(IdentifierType::FileHash),
        "e4d909c290d0fb1ca068ffaddf22cbd0",
        None,
        None,
    )
    .unwrap();
    assert_eq!(hash.canonical_value(), "e4d909c290d0fb1ca068ffaddf22cbd0");

    // Text Phrase
    let phrase = parse_identifier(
        Some(IdentifierType::TextPhrase),
        "   confidential leaked   credentials ",
        None,
        None,
    )
    .unwrap();
    assert_eq!(phrase.canonical_value(), "confidential leaked credentials");

    // ASN
    let asn = parse_identifier(Some(IdentifierType::Asn), "AS15169", None, None).unwrap();
    assert_eq!(asn.canonical_value(), "AS15169");

    // Certificate
    let cert = parse_identifier(
        Some(IdentifierType::Certificate),
        "3B:A0:6B:7C:F4:A8:12:44:89:10:00:11:22:33:44:55:66:77:88:99",
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        cert.canonical_value(),
        "3BA06B7CF4A81244891000112233445566778899"
    );
}
