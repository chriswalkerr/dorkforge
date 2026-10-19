use crate::error::{DorkForgeError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedName {
    pub raw: String,
    pub tokens: Vec<String>,
    pub canonical: String,
}

pub fn parse_and_normalize_name(input: &str) -> Result<NormalizedName> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "name".to_string(),
            value: input.to_string(),
            reason: "Name cannot be empty".to_string(),
        });
    }

    let tokens: Vec<String> = trimmed
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if tokens.is_empty() {
        return Err(DorkForgeError::InvalidInputFormat {
            identifier_type: "name".to_string(),
            value: input.to_string(),
            reason: "Name must contain at least one word".to_string(),
        });
    }

    // Check that tokens contain alphabetic characters, hyphens or apostrophes
    for token in &tokens {
        let valid = token
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == '\'');
        if !valid {
            return Err(DorkForgeError::InvalidInputFormat {
                identifier_type: "name".to_string(),
                value: input.to_string(),
                reason: format!("Token '{token}' contains invalid characters"),
            });
        }
    }

    let canonical = tokens.join(" ");

    Ok(NormalizedName {
        raw: input.to_string(),
        tokens,
        canonical,
    })
}

/// Transliterates Cyrillic text to Latin using standard ICAO / BGN mapping.
pub fn cyrillic_to_latin(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        let mapped = match c {
            'а' => "a",
            'А' => "A",
            'б' => "b",
            'Б' => "B",
            'в' => "v",
            'В' => "V",
            'г' => "g",
            'Г' => "G",
            'д' => "d",
            'Д' => "D",
            'е' => "e",
            'Е' => "E",
            'ё' => "yo",
            'Ё' => "Yo",
            'ж' => "zh",
            'Ж' => "Zh",
            'з' => "z",
            'З' => "Z",
            'и' => "i",
            'И' => "I",
            'й' => "y",
            'Й' => "Y",
            'к' => "k",
            'К' => "K",
            'л' => "l",
            'Л' => "L",
            'м' => "m",
            'М' => "M",
            'н' => "n",
            'Н' => "N",
            'о' => "o",
            'О' => "O",
            'п' => "p",
            'П' => "P",
            'р' => "r",
            'Р' => "R",
            'с' => "s",
            'С' => "S",
            'т' => "t",
            'Т' => "T",
            'у' => "u",
            'У' => "U",
            'ф' => "f",
            'Ф' => "F",
            'х' => "kh",
            'Х' => "Kh",
            'ц' => "ts",
            'Ц' => "Ts",
            'ч' => "ch",
            'Ч' => "Ch",
            'ш' => "sh",
            'Ш' => "Sh",
            'щ' => "shch",
            'Щ' => "Shch",
            'ъ' => "",
            'Ъ' => "",
            'ы' => "y",
            'Ы' => "Y",
            'ь' => "",
            'Ь' => "",
            'э' => "e",
            'Э' => "E",
            'ю' => "yu",
            'Ю' => "Yu",
            'я' => "ya",
            'Я' => "Ya",
            // Serbian / Balkan Cyrillic specific letters:
            'ђ' => "dj",
            'Ђ' => "Dj",
            'ј' => "j",
            'Ј' => "J",
            'љ' => "lj",
            'Љ' => "Lj",
            'њ' => "nj",
            'Њ' => "Nj",
            'ћ' => "c",
            'Ћ' => "C",
            'џ' => "dz",
            'Џ' => "Dz",
            // Ukrainian specific letters:
            'і' => "i",
            'І' => "I",
            'ї' => "yi",
            'Ї' => "Yi",
            'є' => "ye",
            'Є' => "Ye",
            'ґ' => "g",
            'Ґ' => "G",
            _ => {
                result.push(c);
                continue;
            }
        };
        result.push_str(mapped);
    }
    result
}

pub fn has_cyrillic(input: &str) -> bool {
    input
        .chars()
        .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        let norm = parse_and_normalize_name("  John   Doe  ").unwrap();
        assert_eq!(norm.canonical, "John Doe");
        assert_eq!(norm.tokens, vec!["John", "Doe"]);
    }

    #[test]
    fn test_cyrillic_translit() {
        let norm = parse_and_normalize_name("Иван Иванов").unwrap();
        assert!(has_cyrillic(&norm.canonical));
        let lat = cyrillic_to_latin(&norm.canonical);
        assert_eq!(lat, "Ivan Ivanov");
    }
}
