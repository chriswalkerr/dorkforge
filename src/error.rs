use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DorkForgeError {
    #[error("Unknown identifier type: '{0}'. Supported types: email, wallet, username, name, domain, ip, phone, license_plate, vin, company, file_hash, person_id, text_phrase, asn, certificate, url")]
    UnknownIdentifierType(String),

    #[error("Invalid input format for {identifier_type}: '{value}' ({reason})")]
    InvalidInputFormat {
        identifier_type: String,
        value: String,
        reason: String,
    },

    #[error("Ambiguous identifier '{value}'. Please specify the type explicitly with --type. Supported types: email, wallet, username, name, domain, ip, phone, license_plate, vin, company, file_hash, person_id, text_phrase, asn, certificate, url")]
    AmbiguousIdentifier { value: String },

    #[error(
        "Unknown provider: '{0}'. Supported providers: google, bing, yandex, brave, duckduckgo"
    )]
    UnknownProvider(String),

    #[error("Unknown category: '{0}'. Supported categories: exact_match, mentions, documents, profiles, forums, code, domain_pages, subdomains, archives, custom")]
    UnknownCategory(String),

    #[error("Configuration error in '{path}': {message}")]
    ConfigError { path: PathBuf, message: String },

    #[error("Invalid template '{id}': {message}")]
    InvalidTemplate { id: String, message: String },

    #[error("Unknown placeholder '{placeholder}' in template '{template_id}'")]
    UnknownPlaceholder {
        template_id: String,
        placeholder: String,
    },

    #[error("Limit exceeded: {message}")]
    LimitExceeded { message: String },

    #[error("I/O error at '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Output conflict at '{path}': destination already exists and overwrite is disabled")]
    OutputConflict { path: PathBuf },

    #[error("Empty input provided")]
    EmptyInput,

    #[error("Validation failed: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, DorkForgeError>;
