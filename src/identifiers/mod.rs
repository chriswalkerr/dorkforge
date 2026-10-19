pub mod asn;
pub mod certificate;
pub mod company;
pub mod domain;
pub mod email;
pub mod file_hash;
pub mod ip;
pub mod license_plate;
pub mod name;
pub mod person_id;
pub mod phone;
pub mod text_phrase;
pub mod url;
pub mod username;
pub mod vin;
pub mod wallet;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{DorkForgeError, Result};
pub use asn::NormalizedAsn;
pub use certificate::NormalizedCertificate;
pub use company::NormalizedCompany;
pub use domain::NormalizedDomain;
pub use email::NormalizedEmail;
pub use file_hash::NormalizedFileHash;
pub use ip::NormalizedIp;
pub use license_plate::NormalizedLicensePlate;
pub use name::NormalizedName;
pub use person_id::NormalizedPersonId;
pub use phone::NormalizedPhone;
pub use text_phrase::NormalizedTextPhrase;
pub use url::NormalizedUrl;
pub use username::NormalizedUsername;
pub use vin::NormalizedVin;
pub use wallet::NormalizedWallet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifierType {
    Email,
    Wallet,
    Username,
    Name,
    Domain,
    Ip,
    Phone,
    LicensePlate,
    // Priority 1 additions:
    Company,
    PersonId,
    Url,
    Vin,
    FileHash,
    TextPhrase,
    Asn,
    Certificate,
}

impl IdentifierType {
    pub const ALL: &'static [IdentifierType] = &[
        IdentifierType::Email,
        IdentifierType::Wallet,
        IdentifierType::Username,
        IdentifierType::Name,
        IdentifierType::Domain,
        IdentifierType::Ip,
        IdentifierType::Phone,
        IdentifierType::LicensePlate,
        IdentifierType::Company,
        IdentifierType::PersonId,
        IdentifierType::Url,
        IdentifierType::Vin,
        IdentifierType::FileHash,
        IdentifierType::TextPhrase,
        IdentifierType::Asn,
        IdentifierType::Certificate,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            IdentifierType::Email => "email",
            IdentifierType::Wallet => "wallet",
            IdentifierType::Username => "username",
            IdentifierType::Name => "name",
            IdentifierType::Domain => "domain",
            IdentifierType::Ip => "ip",
            IdentifierType::Phone => "phone",
            IdentifierType::LicensePlate => "license_plate",
            IdentifierType::Company => "company",
            IdentifierType::PersonId => "person_id",
            IdentifierType::Url => "url",
            IdentifierType::Vin => "vin",
            IdentifierType::FileHash => "file_hash",
            IdentifierType::TextPhrase => "text_phrase",
            IdentifierType::Asn => "asn",
            IdentifierType::Certificate => "certificate",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            IdentifierType::Email => "Email address (e.g., user@example.com)",
            IdentifierType::Wallet => "Cryptocurrency wallet address (e.g., BTC, ETH, SOL)",
            IdentifierType::Username => "Online username / handle (e.g., alex_dev)",
            IdentifierType::Name => "Full name (e.g., John Doe, Иван Иванов)",
            IdentifierType::Domain => "Internet domain name (e.g., example.org)",
            IdentifierType::Ip => "IPv4 or IPv6 address (e.g., 192.168.1.1, 2001:db8::1)",
            IdentifierType::Phone => "Phone number with optional country code",
            IdentifierType::LicensePlate => "Vehicle license plate (e.g., BG-123-AA)",
            IdentifierType::Company => "Company, organization or brand name",
            IdentifierType::PersonId => {
                "National, tax, or public registry ID (INN, OGRN, SSN, JMBG)"
            }
            IdentifierType::Url => "URL, endpoint, path, or filename",
            IdentifierType::Vin => "Vehicle Identification Number (17-character VIN)",
            IdentifierType::FileHash => "Cryptographic file hash (MD5, SHA1, SHA256)",
            IdentifierType::TextPhrase => "Exact text phrase, quote, or incident signature",
            IdentifierType::Asn => "Autonomous System Number (e.g., AS15169)",
            IdentifierType::Certificate => "TLS/SSL Certificate fingerprint or serial number",
        }
    }

    pub fn applicable_categories(&self) -> &'static [&'static str] {
        match self {
            IdentifierType::Email => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "profiles",
                "public_profiles",
                "forums",
                "code",
                "code_repositories",
                "security_reports",
                "marketplaces",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Wallet => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "forums",
                "security_reports",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Username => &[
                "exact_match",
                "mentions",
                "profiles",
                "public_profiles",
                "forums",
                "code",
                "code_repositories",
                "marketplaces",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Name => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "profiles",
                "public_profiles",
                "forums",
                "company_records",
                "marketplaces",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Domain => &[
                "exact_match",
                "domain_pages",
                "subdomains",
                "documents",
                "public_documents",
                "profiles",
                "public_profiles",
                "forums",
                "code",
                "code_repositories",
                "archives",
                "historical_pages",
                "mentions",
                "company_records",
                "security_reports",
                "technology_fingerprint",
                "custom",
            ],
            IdentifierType::Ip => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "forums",
                "security_reports",
                "technology_fingerprint",
                "custom",
            ],
            IdentifierType::Phone => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "forums",
                "marketplaces",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::LicensePlate => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "forums",
                "marketplaces",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Company => &[
                "exact_match",
                "mentions",
                "company_records",
                "documents",
                "public_documents",
                "profiles",
                "public_profiles",
                "forums",
                "marketplaces",
                "security_reports",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::PersonId => &[
                "exact_match",
                "mentions",
                "public_records",
                "company_records",
                "documents",
                "public_documents",
                "forums",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Url => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "code",
                "code_repositories",
                "security_reports",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Vin => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "marketplaces",
                "forums",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::FileHash => &[
                "exact_match",
                "mentions",
                "security_reports",
                "code",
                "code_repositories",
                "documents",
                "public_documents",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::TextPhrase => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "forums",
                "security_reports",
                "marketplaces",
                "archives",
                "historical_pages",
                "custom",
            ],
            IdentifierType::Asn => &[
                "exact_match",
                "mentions",
                "documents",
                "public_documents",
                "technology_fingerprint",
                "security_reports",
                "forums",
                "custom",
            ],
            IdentifierType::Certificate => &[
                "exact_match",
                "mentions",
                "security_reports",
                "technology_fingerprint",
                "documents",
                "public_documents",
                "custom",
            ],
        }
    }
}

impl fmt::Display for IdentifierType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for IdentifierType {
    type Err = DorkForgeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "email" => Ok(IdentifierType::Email),
            "wallet" => Ok(IdentifierType::Wallet),
            "username" => Ok(IdentifierType::Username),
            "name" => Ok(IdentifierType::Name),
            "domain" => Ok(IdentifierType::Domain),
            "ip" => Ok(IdentifierType::Ip),
            "phone" => Ok(IdentifierType::Phone),
            "license_plate" | "licenseplate" | "plate" => Ok(IdentifierType::LicensePlate),
            "company" | "org" | "organization" => Ok(IdentifierType::Company),
            "person_id" | "personid" | "inn" | "ssn" | "jmbg" => Ok(IdentifierType::PersonId),
            "url" | "path" | "endpoint" => Ok(IdentifierType::Url),
            "vin" => Ok(IdentifierType::Vin),
            "file_hash" | "filehash" | "hash" => Ok(IdentifierType::FileHash),
            "text_phrase" | "textphrase" | "phrase" | "quote" => Ok(IdentifierType::TextPhrase),
            "asn" | "as" => Ok(IdentifierType::Asn),
            "certificate" | "cert" | "fingerprint" => Ok(IdentifierType::Certificate),
            other => Err(DorkForgeError::UnknownIdentifierType(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedIdentifier {
    Email(NormalizedEmail),
    Wallet(NormalizedWallet),
    Username(NormalizedUsername),
    Name(NormalizedName),
    Domain(NormalizedDomain),
    Ip(NormalizedIp),
    Phone(NormalizedPhone),
    LicensePlate(NormalizedLicensePlate),
    Company(NormalizedCompany),
    PersonId(NormalizedPersonId),
    Url(NormalizedUrl),
    Vin(NormalizedVin),
    FileHash(NormalizedFileHash),
    TextPhrase(NormalizedTextPhrase),
    Asn(NormalizedAsn),
    Certificate(NormalizedCertificate),
}

impl ParsedIdentifier {
    pub fn identifier_type(&self) -> IdentifierType {
        match self {
            ParsedIdentifier::Email(_) => IdentifierType::Email,
            ParsedIdentifier::Wallet(_) => IdentifierType::Wallet,
            ParsedIdentifier::Username(_) => IdentifierType::Username,
            ParsedIdentifier::Name(_) => IdentifierType::Name,
            ParsedIdentifier::Domain(_) => IdentifierType::Domain,
            ParsedIdentifier::Ip(_) => IdentifierType::Ip,
            ParsedIdentifier::Phone(_) => IdentifierType::Phone,
            ParsedIdentifier::LicensePlate(_) => IdentifierType::LicensePlate,
            ParsedIdentifier::Company(_) => IdentifierType::Company,
            ParsedIdentifier::PersonId(_) => IdentifierType::PersonId,
            ParsedIdentifier::Url(_) => IdentifierType::Url,
            ParsedIdentifier::Vin(_) => IdentifierType::Vin,
            ParsedIdentifier::FileHash(_) => IdentifierType::FileHash,
            ParsedIdentifier::TextPhrase(_) => IdentifierType::TextPhrase,
            ParsedIdentifier::Asn(_) => IdentifierType::Asn,
            ParsedIdentifier::Certificate(_) => IdentifierType::Certificate,
        }
    }

    pub fn raw_value(&self) -> &str {
        match self {
            ParsedIdentifier::Email(e) => &e.raw,
            ParsedIdentifier::Wallet(w) => &w.raw,
            ParsedIdentifier::Username(u) => &u.raw,
            ParsedIdentifier::Name(n) => &n.raw,
            ParsedIdentifier::Domain(d) => &d.raw,
            ParsedIdentifier::Ip(i) => &i.raw,
            ParsedIdentifier::Phone(p) => &p.raw,
            ParsedIdentifier::LicensePlate(l) => &l.raw,
            ParsedIdentifier::Company(c) => &c.raw,
            ParsedIdentifier::PersonId(pid) => &pid.raw,
            ParsedIdentifier::Url(u) => &u.raw,
            ParsedIdentifier::Vin(v) => &v.raw,
            ParsedIdentifier::FileHash(h) => &h.raw,
            ParsedIdentifier::TextPhrase(tp) => &tp.raw,
            ParsedIdentifier::Asn(a) => &a.raw,
            ParsedIdentifier::Certificate(c) => &c.raw,
        }
    }

    pub fn canonical_value(&self) -> &str {
        match self {
            ParsedIdentifier::Email(e) => &e.canonical,
            ParsedIdentifier::Wallet(w) => &w.address,
            ParsedIdentifier::Username(u) => &u.cleaned,
            ParsedIdentifier::Name(n) => &n.canonical,
            ParsedIdentifier::Domain(d) => &d.domain,
            ParsedIdentifier::Ip(i) => &i.canonical,
            ParsedIdentifier::Phone(p) => p.e164.as_deref().unwrap_or(&p.digits_only),
            ParsedIdentifier::LicensePlate(l) => &l.cleaned,
            ParsedIdentifier::Company(c) => &c.canonical,
            ParsedIdentifier::PersonId(pid) => &pid.canonical,
            ParsedIdentifier::Url(u) => &u.canonical,
            ParsedIdentifier::Vin(v) => &v.vin,
            ParsedIdentifier::FileHash(h) => &h.hash,
            ParsedIdentifier::TextPhrase(tp) => &tp.phrase,
            ParsedIdentifier::Asn(a) => &a.canonical,
            ParsedIdentifier::Certificate(c) => &c.compact_hex,
        }
    }
}

/// Helper function to automatically detect the identifier type.
/// If ambiguous or invalid, returns an error instead of guessing silently.
pub fn auto_detect_type(value: &str) -> Result<IdentifierType> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::EmptyInput);
    }

    // Unambiguous IP?
    if trimmed.parse::<std::net::IpAddr>().is_ok() {
        return Ok(IdentifierType::Ip);
    }

    // Unambiguous Email? (contains @, dot in domain, no spaces)
    if trimmed.contains('@') && !trimmed.contains(' ') {
        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() == 2 && parts[1].contains('.') && !parts[0].is_empty() {
            return Ok(IdentifierType::Email);
        }
    }

    // Phone starting with '+' and followed by mostly digits
    if trimmed.starts_with('+') {
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 5 && digits.len() <= 15 {
            return Ok(IdentifierType::Phone);
        }
    }

    // EVM Wallet starting with 0x and length 42 hex
    if (trimmed.starts_with("0x") || trimmed.starts_with("0X"))
        && trimmed.len() == 42
        && trimmed[2..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Ok(IdentifierType::Wallet);
    }

    // Bitcoin Bech32 starting with bc1
    if trimmed.to_ascii_lowercase().starts_with("bc1") && (42..=62).contains(&trimmed.len()) {
        return Ok(IdentifierType::Wallet);
    }

    // URL starting with http:// or https://
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(IdentifierType::Url);
    }

    // ASN starting with AS and digits
    if trimmed.to_ascii_uppercase().starts_with("AS")
        && trimmed.len() > 2
        && trimmed[2..].trim().chars().all(|c| c.is_ascii_digit())
    {
        return Ok(IdentifierType::Asn);
    }

    // 17-character VIN (alphanumeric, no I, O, Q)
    if trimmed.len() == 17
        && trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric()
                && c != 'I'
                && c != 'O'
                && c != 'Q'
                && c != 'i'
                && c != 'o'
                && c != 'q'
        })
    {
        return Ok(IdentifierType::Vin);
    }

    // 64-character SHA256 hex
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(IdentifierType::FileHash);
    }

    // Otherwise, ambiguous: could be domain, username, wallet, name, plate, company, etc.
    Err(DorkForgeError::AmbiguousIdentifier {
        value: value.to_string(),
    })
}

/// Parses an identifier given an optional explicit type or through auto-detection.
pub fn parse_identifier(
    type_opt: Option<IdentifierType>,
    value: &str,
    country_hint: Option<&str>,
    network_hint: Option<&str>,
) -> Result<ParsedIdentifier> {
    let resolved_type = match type_opt {
        Some(t) => t,
        None => auto_detect_type(value)?,
    };

    match resolved_type {
        IdentifierType::Email => {
            email::parse_and_normalize_email(value).map(ParsedIdentifier::Email)
        }
        IdentifierType::Wallet => {
            wallet::parse_and_normalize_wallet(value, network_hint).map(ParsedIdentifier::Wallet)
        }
        IdentifierType::Username => {
            username::parse_and_normalize_username(value).map(ParsedIdentifier::Username)
        }
        IdentifierType::Name => name::parse_and_normalize_name(value).map(ParsedIdentifier::Name),
        IdentifierType::Domain => {
            domain::parse_and_normalize_domain(value).map(ParsedIdentifier::Domain)
        }
        IdentifierType::Ip => ip::parse_and_normalize_ip(value).map(ParsedIdentifier::Ip),
        IdentifierType::Phone => {
            phone::parse_and_normalize_phone(value, country_hint).map(ParsedIdentifier::Phone)
        }
        IdentifierType::LicensePlate => {
            license_plate::parse_and_normalize_license_plate(value, country_hint)
                .map(ParsedIdentifier::LicensePlate)
        }
        IdentifierType::Company => {
            company::parse_and_normalize_company(value).map(ParsedIdentifier::Company)
        }
        IdentifierType::PersonId => person_id::parse_and_normalize_person_id(value, country_hint)
            .map(ParsedIdentifier::PersonId),
        IdentifierType::Url => url::parse_and_normalize_url(value).map(ParsedIdentifier::Url),
        IdentifierType::Vin => vin::parse_and_normalize_vin(value).map(ParsedIdentifier::Vin),
        IdentifierType::FileHash => {
            file_hash::parse_and_normalize_file_hash(value).map(ParsedIdentifier::FileHash)
        }
        IdentifierType::TextPhrase => {
            text_phrase::parse_and_normalize_text_phrase(value).map(ParsedIdentifier::TextPhrase)
        }
        IdentifierType::Asn => asn::parse_and_normalize_asn(value).map(ParsedIdentifier::Asn),
        IdentifierType::Certificate => {
            certificate::parse_and_normalize_certificate(value).map(ParsedIdentifier::Certificate)
        }
    }
}
