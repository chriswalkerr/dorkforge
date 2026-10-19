use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "dorkforge",
    author = "DorkForge Contributors",
    version,
    about = "Deterministic local OSINT search query generator",
    long_about = "DorkForge generates structured OSINT search engine dorks locally without network access,\nsupporting single & combined identifiers, search coverage matrix, template quality linting, providers, and templates.\n\nTips:\n  - Use '-v -' to read a single identifier from stdin.\n  - Use '--combine username:alice --combine domain:example.org' for multi-identifier combinations.\n  - Use '-i batch.json' for batch input (JSON array or combinations JSON file).\n  - Use 'dorkforge coverage' to inspect template & source coverage.\n  - Use 'dorkforge lint-templates --strict' to validate template catalog quality.\n  - Use '--format all -o output/' to save TXT, JSON, and clickable HTML."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate search queries from identifiers or combinations
    Generate(Box<GenerateArgs>),

    /// List supported identifier types
    Types,

    /// List supported search engine providers
    Providers,

    /// List supported query categories (dynamic, loaded from templates)
    Categories,

    /// Report search catalog coverage across categories, providers, types, and source profiles
    Coverage(CoverageArgs),

    /// Validate template catalog quality, metadata, syntax, and operator compatibility
    LintTemplates(LintTemplatesArgs),

    /// Generate shell completion scripts
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Render a generation JSON file as a Graphviz DOT OSINT pivot graph
    Graph(GraphArgs),

    /// Compare two JSON generation results
    Diff(DiffArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GenerateArgs {
    /// Explicit identifier type (e.g. email, wallet, username, domain, ip, phone,
    /// license_plate, vin, company, file_hash, person_id, text_phrase, asn, certificate, url)
    #[arg(short = 't', long)]
    pub r#type: Option<String>,

    /// Identifier value. Use '-' to read from stdin (e.g. echo 'user@example.com' | dorkforge generate -v -)
    #[arg(short = 'v', long)]
    pub value: Option<String>,

    /// Combine multiple identifiers: e.g. --combine username:sample_user --combine domain:example.org
    #[arg(long = "combine", value_name = "TYPE:VALUE")]
    pub combine: Vec<String>,

    /// Path to file with identifiers (JSON array, TXT file, or combinations JSON file)
    #[arg(short = 'i', long)]
    pub input: Option<PathBuf>,

    /// Target search providers (comma-separated: google,bing,yandex,brave,duckduckgo)
    #[arg(short = 'p', long, value_delimiter = ',')]
    pub providers: Option<Vec<String>>,

    /// Target categories (comma-separated, aliases supported: documents, profiles, code, archives, …)
    #[arg(short = 'c', long, value_delimiter = ',')]
    pub categories: Option<Vec<String>>,

    /// Output format: txt, json, html, both (txt+json), or all (txt+json+html).
    #[arg(short = 'f', long, default_value = "txt")]
    pub format: CliOutputFormat,

    /// Output destination path (directory or file). When omitted, results go to stdout.
    /// With --format both and a file path (e.g. -o out.txt) two files are created:
    /// out.txt and out.json using the same base name.
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// TXT layout mode: 'readable' (with headers) or 'clean' (queries only, for piping)
    #[arg(short = 'm', long, default_value = "readable")]
    pub mode: CliTxtMode,

    /// Suppress all informational messages on stderr (for scripting)
    #[arg(short = 'q', long, default_value_t = false)]
    pub quiet: bool,

    /// Use only templates from this directory instead of built-in defaults
    #[arg(long, conflicts_with = "extra_templates")]
    pub config_dir: Option<PathBuf>,

    /// Merge extra templates from this directory ON TOP of built-in defaults
    #[arg(long, conflicts_with = "config_dir")]
    pub extra_templates: Option<PathBuf>,

    /// Optional country profile hint (e.g. RS, US, RU) for license plates, phone formatting, etc.
    #[arg(long)]
    pub country: Option<String>,

    /// Optional network/asset hint for crypto addresses (e.g. btc, eth, sol)
    #[arg(long)]
    pub network: Option<String>,

    /// Maximum variants per identifier (0 is rejected)
    #[arg(long, default_value_t = 10)]
    pub max_variants: usize,

    /// Maximum queries per identifier
    #[arg(long, default_value_t = 50)]
    pub max_queries_per_identifier: usize,

    /// Maximum queries per provider
    #[arg(long, default_value_t = 100)]
    pub max_queries_per_provider: usize,

    /// Maximum variants per identifier in combinations
    #[arg(long, default_value_t = 5)]
    pub max_combination_variants: usize,

    /// Maximum variant combinations (cartesian product)
    #[arg(long, default_value_t = 25)]
    pub max_variant_combinations: usize,

    /// Maximum total queries per combination
    #[arg(long, default_value_t = 100)]
    pub max_combination_queries: usize,

    /// Strict mode: abort immediately on any batch input or limit error
    #[arg(long, default_value_t = false)]
    pub strict: bool,

    /// Overwrite existing output files
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,

    /// Generate clickable search URLs instead of raw dork strings
    /// (e.g. https://google.com/search?q=... )
    #[arg(long, default_value_t = false)]
    pub urls: bool,

    /// Keep running and regenerate when external template files change.
    /// Requires --config-dir or --extra-templates.
    #[arg(long, default_value_t = false)]
    pub watch: bool,

    /// Polling interval in seconds used by --watch (minimum: 1)
    #[arg(long, default_value_t = 1, requires = "watch")]
    pub watch_interval: u64,
}

#[derive(Args, Debug, Clone)]
pub struct CoverageArgs {
    /// Report format: txt or json
    #[arg(short = 'f', long, default_value = "txt")]
    pub format: CliDiffFormat,

    /// Filter by specific provider (e.g. google, bing, yandex, brave, duckduckgo)
    #[arg(short = 'p', long)]
    pub provider: Option<String>,

    /// Filter by category (e.g. code_repositories, public_documents, forums)
    #[arg(short = 'c', long)]
    pub category: Option<String>,

    /// Filter by identifier type (e.g. username, domain, email)
    #[arg(short = 't', long)]
    pub r#type: Option<String>,

    /// Path to custom templates directory
    #[arg(long)]
    pub config_dir: Option<PathBuf>,

    /// Path to custom source profiles file (defaults to built-in)
    #[arg(long)]
    pub sources_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct LintTemplatesArgs {
    /// Strict mode: fail on untested validation statuses or missing optional metadata
    #[arg(long, default_value_t = false)]
    pub strict: bool,

    /// Output format: txt or json
    #[arg(short = 'f', long, default_value = "txt")]
    pub format: CliDiffFormat,

    /// Path to custom templates directory
    #[arg(long)]
    pub config_dir: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliOutputFormat {
    Txt,
    Json,
    Html,
    Both,
    All,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliTxtMode {
    Clean,
    Readable,
}

#[derive(Args, Debug)]
pub struct GraphArgs {
    /// Generation result JSON produced by `dorkforge generate --format json`
    pub input: PathBuf,

    /// Destination DOT file. Defaults to stdout.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overwrite an existing DOT file
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// Earlier generation result JSON
    pub before: PathBuf,

    /// Later generation result JSON
    pub after: PathBuf,

    /// Report format: txt or json
    #[arg(short, long, default_value = "txt")]
    pub format: CliDiffFormat,

    /// Destination report file. Defaults to stdout.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overwrite an existing report file
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliDiffFormat {
    Txt,
    Json,
}
