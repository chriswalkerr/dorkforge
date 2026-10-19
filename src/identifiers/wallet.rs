use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WalletFamily {
    Ethereum,
    BitcoinP2PKH,
    BitcoinP2SH,
    BitcoinBech32,
    Solana,
    Tron,
    Unknown,
}

impl WalletFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletFamily::Ethereum => "ethereum",
            WalletFamily::BitcoinP2PKH => "bitcoin_p2pkh",
            WalletFamily::BitcoinP2SH => "bitcoin_p2sh",
            WalletFamily::BitcoinBech32 => "bitcoin_bech32",
            WalletFamily::Solana => "solana",
            WalletFamily::Tron => "tron",
            WalletFamily::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedWallet {
    pub raw: String,
    pub address: String,
    pub family: WalletFamily,
    pub network_hint: Option<String>,
}

fn is_base58_char(c: char) -> bool {
    matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
}

fn is_bech32_char(c: char) -> bool {
    matches!(c, '0' | '2'..='9' | 'a'..='c' | 'e'..='h' | 'j'..='n' | 'p'..='z')
}

pub fn detect_wallet_family(addr: &str) -> WalletFamily {
    let len = addr.len();

    // Ethereum / EVM: 0x followed by 40 hex digits
    if (addr.starts_with("0x") || addr.starts_with("0X"))
        && len == 42
        && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return WalletFamily::Ethereum;
    }

    // Bitcoin Bech32: starts with bc1
    if addr.to_ascii_lowercase().starts_with("bc1") {
        let payload = &addr[3..];
        if (39..=59).contains(&payload.len())
            && payload
                .chars()
                .all(|c| is_bech32_char(c.to_ascii_lowercase()))
        {
            return WalletFamily::BitcoinBech32;
        }
    }

    // Bitcoin P2PKH: starts with 1, 26-35 base58
    if addr.starts_with('1') && (26..=35).contains(&len) && addr.chars().all(is_base58_char) {
        return WalletFamily::BitcoinP2PKH;
    }

    // Bitcoin P2SH: starts with 3, 34 base58
    if addr.starts_with('3') && len == 34 && addr.chars().all(is_base58_char) {
        return WalletFamily::BitcoinP2SH;
    }

    // Tron: starts with T, 34 base58
    if addr.starts_with('T') && len == 34 && addr.chars().all(is_base58_char) {
        return WalletFamily::Tron;
    }

    // Solana: 32-44 base58
    if (32..=44).contains(&len) && addr.chars().all(is_base58_char) {
        return WalletFamily::Solana;
    }

    WalletFamily::Unknown
}

pub fn parse_and_normalize_wallet(
    input: &str,
    network_hint: Option<&str>,
) -> Result<NormalizedWallet> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "wallet".to_string(),
            value: input.to_string(),
            reason: "Wallet address cannot be empty".to_string(),
        });
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "wallet".to_string(),
            value: input.to_string(),
            reason: "Wallet address cannot contain whitespace".to_string(),
        });
    }

    let family = detect_wallet_family(trimmed);

    Ok(NormalizedWallet {
        raw: input.to_string(),
        address: trimmed.to_string(),
        family,
        network_hint: network_hint.map(|s| s.trim().to_ascii_lowercase()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_wallet() {
        let addr = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let norm = parse_and_normalize_wallet(addr, None).unwrap();
        assert_eq!(norm.address, addr);
        assert_eq!(norm.family, WalletFamily::Ethereum);
    }

    #[test]
    fn test_btc_wallet() {
        let addr = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let norm = parse_and_normalize_wallet(addr, Some("btc")).unwrap();
        assert_eq!(norm.address, addr);
        assert_eq!(norm.family, WalletFamily::BitcoinP2PKH);
        assert_eq!(norm.network_hint.as_deref(), Some("btc"));
    }

    #[test]
    fn test_solana_wallet() {
        let addr = "HN7cABqLq46Es1jh92dQQisAq662SmxELLLsHHe4YWrH";
        let norm = parse_and_normalize_wallet(addr, None).unwrap();
        assert_eq!(norm.address, addr);
        assert_eq!(norm.family, WalletFamily::Solana);
    }
}
