use std::collections::HashSet;

use crate::identifiers::name::{cyrillic_to_latin, has_cyrillic};
use crate::identifiers::wallet::WalletFamily;
use crate::identifiers::ParsedIdentifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantConfig {
    pub max_variants_per_identifier: usize,
}

impl Default for VariantConfig {
    fn default() -> Self {
        Self {
            max_variants_per_identifier: 10,
        }
    }
}

pub struct VariantGenerator {
    config: VariantConfig,
}

impl VariantGenerator {
    pub fn new(config: VariantConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, identifier: &ParsedIdentifier) -> Vec<String> {
        let mut candidates = Vec::new();

        // 1. Always prioritize the canonical or cleaned value, and raw if different
        let raw = identifier.raw_value().trim();
        let canonical = identifier.canonical_value().trim();

        if !raw.is_empty() {
            candidates.push(raw.to_string());
        }
        if !canonical.is_empty() && canonical != raw {
            candidates.push(canonical.to_string());
        }

        match identifier {
            ParsedIdentifier::Email(e) => {
                // Lowercase full email
                candidates.push(e.canonical.to_ascii_lowercase());

                // Check for Gmail-style +tag: user+tag@domain -> user@domain
                let base_local = if let Some(plus_idx) = e.local_part.find('+') {
                    let stripped = &e.local_part[..plus_idx];
                    if !stripped.is_empty() {
                        candidates.push(format!("{}@{}", stripped, e.domain));
                        candidates.push(format!("{}@{}", stripped.to_ascii_lowercase(), e.domain));
                    }
                    stripped
                } else {
                    e.local_part.as_str()
                };

                // Check for dot variations in local-part: user.name@domain -> username@domain
                if e.local_part.contains('.') {
                    let no_dots = e.local_part.replace('.', "");
                    if !no_dots.is_empty() {
                        candidates.push(format!("{}@{}", no_dots, e.domain));
                        candidates.push(format!("{}@{}", no_dots.to_ascii_lowercase(), e.domain));
                    }
                }

                if base_local.contains('.') {
                    let no_dots = base_local.replace('.', "");
                    if !no_dots.is_empty() {
                        candidates.push(format!("{}@{}", no_dots, e.domain));
                        candidates.push(format!("{}@{}", no_dots.to_ascii_lowercase(), e.domain));
                    }
                }
            }

            ParsedIdentifier::Wallet(w) => {
                // Maintain case-sensitivity for Base58
                // For EVM addresses, 0x hex lowercase is a common variant
                if w.family == WalletFamily::Ethereum {
                    candidates.push(w.address.to_ascii_lowercase());
                }
            }

            ParsedIdentifier::Username(u) => {
                let cleaned = &u.cleaned;
                candidates.push(cleaned.to_ascii_lowercase());

                // Capitalized first letter
                let mut chars = cleaned.chars();
                if let Some(first) = chars.next() {
                    let cap = first.to_uppercase().collect::<String>() + chars.as_str();
                    candidates.push(cap);
                }

                // Separator variations: replace _ with - or .
                if cleaned.contains('_') {
                    candidates.push(cleaned.replace('_', "-"));
                    candidates.push(cleaned.replace('_', "."));
                    candidates.push(cleaned.replace('_', ""));
                } else if cleaned.contains('-') {
                    candidates.push(cleaned.replace('-', "_"));
                    candidates.push(cleaned.replace('-', "."));
                    candidates.push(cleaned.replace('-', ""));
                } else if cleaned.contains('.') {
                    candidates.push(cleaned.replace('.', "_"));
                    candidates.push(cleaned.replace('.', "-"));
                    candidates.push(cleaned.replace('.', ""));
                }
            }

            ParsedIdentifier::Name(n) => {
                let tokens = &n.tokens;
                if tokens.len() == 2 {
                    // "First Last" -> "Last First"
                    candidates.push(format!("{} {}", tokens[1], tokens[0]));
                }

                // Transliteration if Cyrillic
                if has_cyrillic(&n.canonical) {
                    let translit = cyrillic_to_latin(&n.canonical);
                    candidates.push(translit.clone());
                    let lat_tokens: Vec<&str> = translit.split_whitespace().collect();
                    if lat_tokens.len() == 2 {
                        candidates.push(format!("{} {}", lat_tokens[1], lat_tokens[0]));
                    }
                }

                // Lowercase
                candidates.push(n.canonical.to_ascii_lowercase());
            }

            ParsedIdentifier::Domain(d) => {
                let dom = &d.domain;
                if let Some(without_www) = dom.strip_prefix("www.") {
                    candidates.push(without_www.to_string());
                } else {
                    candidates.push(format!("www.{dom}"));
                }

                if let Some(ref root) = d.root_domain {
                    if root != dom {
                        candidates.push(root.clone());
                    }
                }
            }

            ParsedIdentifier::Ip(i) => {
                match i.ip {
                    std::net::IpAddr::V4(_) => {
                        candidates.push(i.canonical.clone());
                    }
                    std::net::IpAddr::V6(v6) => {
                        candidates.push(i.canonical.clone());
                        // Expanded 8-segment form
                        let segments = v6.segments();
                        let expanded = format!(
                            "{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}",
                            segments[0],
                            segments[1],
                            segments[2],
                            segments[3],
                            segments[4],
                            segments[5],
                            segments[6],
                            segments[7]
                        );
                        candidates.push(expanded);
                    }
                }
            }

            ParsedIdentifier::Phone(p) => {
                if let Some(ref e164) = p.e164 {
                    candidates.push(e164.clone());
                }
                candidates.push(p.digits_only.clone());

                // Format with spaces or hyphens if we have standard lengths
                let digits = &p.digits_only;
                if digits.len() == 11 && digits.starts_with('1') {
                    // +1 US format
                    let formatted =
                        format!("+1-{}-{}-{}", &digits[1..4], &digits[4..7], &digits[7..]);
                    let spaced = format!("+1 {} {} {}", &digits[1..4], &digits[4..7], &digits[7..]);
                    candidates.push(formatted);
                    candidates.push(spaced);
                } else if digits.len() >= 9 {
                    // General spaced chunks
                    let chunk_size = 3;
                    let chunks: Vec<String> = digits
                        .as_bytes()
                        .chunks(chunk_size)
                        .map(|c| std::str::from_utf8(c).unwrap_or("").to_string())
                        .collect();
                    candidates.push(chunks.join(" "));
                    candidates.push(chunks.join("-"));
                }
            }

            ParsedIdentifier::LicensePlate(l) => {
                candidates.push(l.cleaned.clone());

                if let (Some(ref reg), Some(ref dig), Some(ref suf)) =
                    (&l.region_code, &l.digits, &l.suffix)
                {
                    // Formats: BG 123 AA, BG-123-AA, BG 123-AA
                    candidates.push(format!("{reg} {dig} {suf}"));
                    candidates.push(format!("{reg}-{dig}-{suf}"));
                    candidates.push(format!("{reg} {dig}-{suf}"));
                } else if l.cleaned.len() >= 6 {
                    // Generic chunking
                    let mid = l.cleaned.len() / 2;
                    candidates.push(format!("{} {}", &l.cleaned[..mid], &l.cleaned[mid..]));
                    candidates.push(format!("{}-{}", &l.cleaned[..mid], &l.cleaned[mid..]));
                }
            }

            ParsedIdentifier::Company(c) => {
                candidates.push(c.clean_name.clone());
                if let Some(ref legal) = c.legal_form {
                    candidates.push(format!("{} {}", c.clean_name, legal));
                }
                candidates.push(c.clean_name.to_ascii_lowercase());
            }

            ParsedIdentifier::PersonId(pid) => {
                candidates.push(pid.canonical.clone());
                candidates.push(pid.digits_or_compact.clone());
                if pid.digits_or_compact.len() == 9 {
                    // SSN 3-2-4
                    let d = &pid.digits_or_compact;
                    candidates.push(format!("{}-{}-{}", &d[..3], &d[3..5], &d[5..]));
                    candidates.push(format!("{} {} {}", &d[..3], &d[3..5], &d[5..]));
                } else if pid.digits_or_compact.len() == 10 {
                    // INN 4-3-3
                    let d = &pid.digits_or_compact;
                    candidates.push(format!("{} {} {}", &d[..4], &d[4..7], &d[7..]));
                }
            }

            ParsedIdentifier::Url(u) => {
                candidates.push(u.canonical.clone());
                if let Some(ref fname) = u.filename {
                    candidates.push(fname.clone());
                }
                candidates.push(u.path.clone());
                if let Some(ref h) = u.host {
                    candidates.push(h.clone());
                }
            }

            ParsedIdentifier::Vin(v) => {
                candidates.push(v.vin.clone());
                candidates.push(format!("{} {} {}", v.wmi, v.vds, v.vis));
                candidates.push(format!("{}-{}-{}", v.wmi, v.vds, v.vis));
            }

            ParsedIdentifier::FileHash(h) => {
                candidates.push(h.hash.clone());
                candidates.push(h.hash.to_ascii_uppercase());
            }

            ParsedIdentifier::TextPhrase(tp) => {
                candidates.push(tp.phrase.clone());
                candidates.push(tp.phrase.to_ascii_lowercase());
            }

            ParsedIdentifier::Asn(a) => {
                candidates.push(a.canonical.clone());
                candidates.push(a.number.to_string());
                candidates.push(format!("AS {}", a.number));
            }

            ParsedIdentifier::Certificate(cert) => {
                candidates.push(cert.compact_hex.clone());
                candidates.push(cert.colon_separated.clone());
                candidates.push(cert.compact_hex.to_ascii_lowercase());
                candidates.push(cert.colon_separated.to_ascii_lowercase());
            }
        }

        // Deduplicate while maintaining deterministic order
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for candidate in candidates {
            let trimmed = candidate.trim().to_string();
            if !trimmed.is_empty() && seen.insert(trimmed.clone()) {
                result.push(trimmed);
                if result.len() >= self.config.max_variants_per_identifier {
                    break;
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::parse_identifier;
    use crate::identifiers::IdentifierType;

    #[test]
    fn test_email_variants() {
        let id = parse_identifier(
            Some(IdentifierType::Email),
            "John.Doe+security@Example.COM",
            None,
            None,
        )
        .unwrap();
        let gen = VariantGenerator::new(VariantConfig::default());
        let vars = gen.generate(&id);

        assert!(!vars.is_empty());
        assert!(vars.contains(&"john.doe@example.com".to_string()));
        assert!(vars.contains(&"johndoe@example.com".to_string()));
    }

    #[test]
    fn test_variant_limits() {
        let id = parse_identifier(
            Some(IdentifierType::Username),
            "super_cool_agent",
            None,
            None,
        )
        .unwrap();
        let gen = VariantGenerator::new(VariantConfig {
            max_variants_per_identifier: 3,
        });
        let vars = gen.generate(&id);
        assert!(vars.len() <= 3);
    }
}
