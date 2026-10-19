# Changelog

All notable changes to DorkForge are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
versioning follows [Semantic Versioning](https://semver.org/).

## [1.3.0] – 2026-10-19

### Added

- **Combined Identifiers**: Multi-identifier search scenarios with bounded Cartesian variant expansion:
  - CLI `--combine <type>:<value>` flag (e.g., `--combine username:admin --combine domain:example.com`).
  - Batch JSON input format supporting `"combinations": [...]` with typed identifier lists.
  - Combination safety limits: `--max-combination-variants`, `--max-variant-combinations`, and `--max-combination-queries` to protect against combinatorial explosion.
  - Combination templates in TOML supporting `identifier_types = [...]` and type-specific `{type}` placeholders.
  - Backwards-compatible query output schema tracking query-level `identifiers` array for combinations.
- **Source Coverage & Matrix Analysis**:
  - `config/sources.toml` defining canonical OSINT source profiles with domains, categories, platforms, and query archetypes.
  - `dorkforge coverage` subcommand computing multidimensional coverage matrices:
    - Category x Provider
    - Type x Category
    - Combination x Category
    - Source x Provider
  - Output formats for coverage reports: formatted plain text table or machine-readable JSON (`--format json|txt`).
  - Filtering by `--provider`, `--category`, `--type` in coverage analysis.
- **Template Quality & Static Linting**:
  - Rich template metadata: `purpose`, `rationale`, `limitations`, `operator_requirements`, `official_references`, and per-provider validation records.
  - `dorkforge lint-templates` subcommand performing offline static analysis:
    - Syntax validation and template parsing.
    - Placeholder validation ensuring placeholders match declared identifier type(s).
    - Provider operator compatibility checks against provider operator registries.
    - URL syntax validation for official documentation references.
  - Strict mode (`--strict`) converting warnings to exit errors for CI pipelines.
- **Combination Shared Templates**: Built-in combination templates covering `username+domain`, `name+company`, `email+domain`, `company+domain`, and `vin+license_plate`.

### Changed

- `TemplateDefinition` now supports `identifier_types` alongside legacy `identifier_type`.
- `deserialize_limitations` dynamically accepts both legacy string format and string array format.
- `GeneratorOptions` extended with combination limits.

---

## [1.2.0] – 2026-10-16

### Added

- Provider operator documentation/test statuses with official references and limitations.
- Template purpose, rationale, source-domain, validation, and limitation metadata.
- Stable provider-query record IDs and JSON statistics separating query texts from provider-query pairs.
- An offline manual operator test matrix for all five search providers.

### Fixed

- Built-in categories now use the canonical vocabulary; shared TOML templates load from their own catalog.
- Runtime operator validation now checks symbolic operators as well as prefixes and Boolean words.
- Strict query deduplication preserves identifier case while normalizing repeated whitespace.
- Catalog linting now scans the actual `config/` layout and validates operator metadata.

### Changed

- HTML reports display validation status, template identity, limitations, and escaped warnings.
- Legacy `query_id`, `fingerprint`, and JSON format version remain unchanged.

---

## [1.1.0] – 2026-10-16

### Added

- **`query_id` field** on every `DorkQuery` — unique deterministic identifier:
  `{template_id}_{provider}_{sha256[:8]}` — fixes duplicate `id` values when
  multiple variants of the same template are rendered.
- **`fingerprint` field** on every `DorkQuery` — first 8 hex chars of SHA-256 of
  the rendered query string; useful for coverage tracking between runs.
- **`--quiet` / `-q`** flag on `generate` — suppresses all `[DorkForge]` and
  `[Warning]` lines on stderr; ideal for scripting and CI pipelines.
- **stdin support** — pass `-v -` to read a single identifier from stdin, e.g.
  `echo 'user@example.com' | dorkforge generate -v -`.
- **`--extra-templates <dir>`** — merge custom TOML templates on top of built-in
  defaults without losing the embedded template set (contrast: `--config-dir`
  replaces built-ins entirely).
- **`--urls`** flag — render results as clickable search engine URLs
  (`https://www.google.com/search?q=…`) instead of raw dork strings.
- **`dorkforge completions <shell>`** subcommand — emit shell completion scripts
  for bash, zsh, fish, powershell, and elvish via `clap_complete`.
- **Category aliases** — `-c documents` now matches `public_documents` templates;
  likewise for `profiles` → `public_profiles`, `code` → `code_repositories`,
  `archives` → `historical_pages`, `company` → `company_records`,
  `security` → `security_reports`, `tech` → `technology_fingerprint`.
- **Category/type mismatch warning** — at generation time, if the requested
  category has no templates for the given identifier type, a warning is printed.
- **`dorkforge categories` is now dynamic** — categories are read from the loaded
  TOML templates at runtime instead of being a hard-coded static list.
- **Readable TXT variant sub-headers** — `#### Variant: {value}` lines group
  queries by variant within each Provider/Category section.
- **`--max-variants 0` validation** — now returns an error; use `1` to disable
  multi-variant expansion.
- **`--format both` without `-o` fix** — instead of mixing TXT and JSON on
  stdout in an invalid format, DorkForge now falls back to `--format txt` with
  a warning message suggesting `-o <dir>`.
- `examples/input.json` now contains all 16 identifier types.

### Fixed

- Duplicate `id` values in JSON output when the same template is rendered for
  multiple variants — replaced by unique `query_id`.
- `-c documents` silently returning 0 results — category alias normalization now
  resolves user-facing short names to canonical template category names.
- `AmbiguousIdentifier` and `UnknownIdentifierType` error messages now list all
  16 supported type names so users know what to pass with `--type`.
- `--format both` stdout producing an output stream that was neither valid JSON
  nor valid plain TXT — fixed with fallback and stderr warning.

### Changed

- `--format` default changed from `both` to `txt` (more predictable for stdout).
- `dorkforge providers` output now also shows the provider's search base URL.

---

## [1.0.0] – 2026-10-12

### Added

- Initial release of DorkForge.
- 16 identifier types: email, wallet, username, name, domain, ip, phone,
  license_plate, vin, company, file_hash, person_id, text_phrase, asn,
  certificate, url.
- 5 search engine providers: Google, Bing, Yandex, Brave, DuckDuckGo.
- TOML-based template system with embedded defaults and external override support.
- Batch JSON and TXT input; TXT supports `type:value` syntax.
- Readable and Clean TXT output modes; pretty and compact JSON.
- Deterministic generation — no timestamps, no randomness, bitwise reproducible.
- Strict and lenient batch-processing modes.
- `--config-dir` for fully custom template sets.
- `dorkforge types`, `providers`, `categories` informational subcommands.
- 65 unit and integration tests; 0 clippy warnings; release binary ≈ 1.8 MB.
