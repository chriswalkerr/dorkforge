<div align="center">

# DorkForge v1.3

**A deterministic, local OSINT dork and search query generator written in Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

</div>

DorkForge is a high-performance CLI tool for security researchers and OSINT analysts. It takes digital identifiers, normalizes them, builds a deterministic list of spelling variants, applies a library of search templates, and exports ready-to-use search engine queries as **TXT**, **JSON**, and **HTML**.

---

## Table of Contents

- [Key Principles](#key-principles)
- [Supported Identifiers](#supported-identifiers)
- [Search Operators](#search-operators)
- [Query Categories](#query-categories)
- [Build and Installation](#build-and-installation)
- [CLI Usage](#cli-usage)
  - [1. Browsing reference lists](#1-browsing-reference-lists)
  - [2. Generating queries for a single value](#2-generating-queries-for-a-single-value)
  - [3. Combined identifiers (Multi-Identifier OSINT)](#3-combined-identifiers-multi-identifier-osint)
  - [4. Source coverage matrix](#4-source-coverage-matrix-dorkforge-coverage)
  - [5. Template quality checks and linting](#5-template-quality-checks-and-linting-dorkforge-lint-templates)
  - [6. Batch processing of single identifiers](#6-batch-processing-of-single-identifiers)
  - [7. HTML report with ready-to-click links](#7-html-report-with-ready-to-click-links)
  - [8. OSINT pivot graph](#8-osint-pivot-graph)
  - [9. Diff of two runs](#9-diff-of-two-runs)
  - [10. Template hot reload](#10-template-hot-reload)
- [Export Formats](#export-formats)
- [Adding Custom Templates](#adding-custom-templates)
- [Testing and Verification](#testing-and-verification)
- [License](#license)

---

## Key Principles

1. **Fully local**: The application works 100% offline. No HTTP requests, DNS resolution, sockets, or background daemons.
2. **Zero telemetry**: No data submission, analytics, or third-party API keys.
3. **Strict determinism**: The same input and configuration are guaranteed to produce an identical result, down to the byte.
4. **Independence of types and categories**: Any identifier can be linked to the applicable search categories (OSINT pivoting).
5. **Provider profiles**: The generator takes into account the real syntax operators of each search engine and never produces queries that are known not to work (for example, the `filetype:` operator is replaced with `mime:` for Yandex, and Boolean operators are excluded for DuckDuckGo).

---

## Supported Identifiers

DorkForge supports 16 identifier types:

| Type | Description | Examples |
| --- | --- | --- |
| `email` | Email address | `user@example.org`, `john.doe+tag@company.com` |
| `wallet` | Cryptocurrency wallet (BTC, ETH, SOL, TRX) | `0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045` |
| `username` | Nickname, alias, or handle | `alex_dev`, `@security_lead` |
| `name` | First and last name (with transliteration and reordering) | `John Doe`, `Иван Иванов` |
| `domain` | Domain name (pages, subdomains, documents) | `example.org`, `sub.corp.com` |
| `ip` | IPv4 or IPv6 address | `192.168.1.1`, `2001:db8::1` |
| `phone` | International phone number | `+1 (555) 019-2834`, `+381 60 1234567` |
| `license_plate` | Vehicle license plate with country profiles | `BG-123-AA` (Serbia), `B-MW-1234` |
| `company` | Company or brand name | `Acme Corporation LLC`, `ООО Яндекс` |
| `person_id` | Registry identifiers (INN, OGRN, SSN, JMBG) | `7707083893`, `123-45-6789` |
| `url` | URL, endpoint, or file name | `https://site.org/secret/passwords.txt` |
| `vin` | Vehicle identifier (17 characters, ISO 3779) | `1HGCR2F83HA000000` |
| `file_hash` | Cryptographic file hash (MD5, SHA1, SHA256) | `e4d909c290d0fb1ca068ffaddf22cbd0` |
| `text_phrase` | Unique phrase, quote, or leak signature | `CONFIDENTIAL PROJECT OMEGA` |
| `asn` | Autonomous system number | `AS15169`, `13335` |
| `certificate` | TLS/SSL certificate fingerprint | `3B:A0:6B:7C:F4:A8:12:44:89:10:00:11...` |

---

## Search Operators

The operator registry, official links, documentation status, and limitations of each provider are located in [config/providers.toml](config/providers.toml). Brave operators are marked `experimental` according to its help page. An operator that is absent from the official documentation is marked `unknown`, not as confirmed support. The built-in manual verification labels are set to `untested`: DorkForge does not perform searches.

The documentation status (`documented`, `experimental`, `unknown`, `unsupported`) is not a test result. The separate `test_status` takes the values `untested`, `passed`, `partial`, `failed`, and `stale`.

---

## Query Categories

| Category | Description |
| --- | --- |
| `exact_match` | Exact match of a string in quotes. |
| `mentions` | Mentions in page bodies (`intext:`, `inbody:`). |
| `documents` | Indexable documents (PDF, XLSX, CSV, DOCX). |
| `public_profiles` | Public profiles on platforms. |
| `forums` | Discussions, forums, and topical communities. |
| `code_repositories` | Public repositories and code. |
| `domain_pages` | Pages within a domain (`site:domain`). |
| `subdomains` | Search heuristics for related pages; they do not enumerate all DNS subdomains. |
| `archives` | Archived pages and historical materials. |
| `public_records` | Public registries related to a person's identifier. |
| `company_records` | Company records. |
| `marketplaces` | Listings, classifieds, car auctions. |
| `security_reports` | Incident reports, vulnerability databases, VirusTotal, Shodan. |
| `technology_fingerprint` | Mentions of technology stacks and network infrastructure. |
| `custom` | User-defined templates. |

---

## Build and Installation

Requires an installed Rust compiler (version 1.85+):

```bash
cargo build --release
```

The executable will be available at `target/release/dorkforge`.

---

## CLI Usage

For the full set of recipes, including all identifier types, inputs, formats, URL mode, custom templates, hot reload, graph, and diff, see [docs/USAGE.md](docs/USAGE.md).

### 1. Browsing reference lists

```bash
# List identifier types and applicable categories
dorkforge types

# List supported search engines and operators
dorkforge providers

# List search query categories
dorkforge categories
```

### 2. Generating queries for a single value

```bash
# Search for a domain on Google, Brave, and DuckDuckGo, exporting to both formats
dorkforge generate \
  --type domain \
  --value "example.org" \
  --providers google,brave,duckduckgo \
  --format both \
  --output ./results

# Search for an email across selected categories
dorkforge generate \
  --type email \
  --value "user@example.org" \
  --categories exact_match,documents,code \
  --format txt \
  --mode readable

# License plate with a country profile (Serbia)
dorkforge generate \
  --type license_plate \
  --value "BG 123-AA" \
  --country RS \
  --format json \
  --output ./results
```

### 3. Combined identifiers (Multi-Identifier OSINT)

DorkForge supports searching for combinations of identifiers (for example, `username + domain`, `name + company`, `vin + license_plate`):

```bash
# Combination from the CLI using --combine <type>:<value> flags
dorkforge generate \
  --combine username:johndoe \
  --combine domain:example.com \
  --providers google,bing \
  --format txt --mode readable

# With safe Cartesian product limits
dorkforge generate \
  --combine name:"John Doe" \
  --combine company:"Acme Corp" \
  --max-combination-variants 4 \
  --max-variant-combinations 16 \
  --max-combination-queries 50
```

Batch processing of combinations from JSON (`examples/combinations.json`):

```bash
dorkforge generate \
  --input examples/combinations.json \
  --format json \
  --output ./results
```

Example structure of `combinations.json`:

```json
{
  "combinations": [
    {
      "id": "target_corp_lead",
      "identifiers": [
        { "type": "username", "value": "security_lead" },
        { "type": "domain", "value": "corp.example.org" }
      ]
    }
  ]
}
```

### 4. Source coverage matrix (`dorkforge coverage`)

Analysis of search template coverage density and the OSINT source matrix (`config/sources.toml`):

```bash
# Summary coverage table by category, provider, and source
dorkforge coverage

# Export the matrix as JSON
dorkforge coverage --format json

# Filter by a specific provider or type
dorkforge coverage --provider google --type email
```

### 5. Template quality checks and linting (`dorkforge lint-templates`)

Static analysis of templates: checks syntax, placeholders, conformance of operators to search engine capabilities, and validity of documentation links:

```bash
# Standard check of built-in and external templates
dorkforge lint-templates

# Strict mode for CI (fails on any findings)
dorkforge lint-templates --strict

# Check a custom template directory
dorkforge lint-templates --path ./my-templates --strict
```

### 6. Batch processing of single identifiers

Input files in **JSON** and **TXT** formats are supported:

```bash
dorkforge generate \
  --input ./examples/input.json \
  --format both \
  --output ./results
```

Example structure of the `input.json` input file:

```json
[
  { "type": "email", "value": "analyst@example.org" },
  { "type": "domain", "value": "targetcorp.com" },
  { "type": "wallet", "value": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "network": "eth" },
  { "type": "license_plate", "value": "BG-123-AA", "country": "RS" }
]
```

### 7. HTML report with ready-to-click links

```bash
dorkforge generate \
  --type domain --value example.org \
  --format html --output ./results
```

This creates `results/dorks.html`: a standalone table of queries in which each row contains a safe, clickable link to the corresponding search engine. For the full set of artifacts, use `--format all` (TXT, JSON, and HTML). In TXT, you can output URLs only with `--urls`.

### 8. OSINT pivot graph

The graph is built from a saved JSON file and emitted in a deterministic Graphviz DOT format. Query nodes contain URL attributes that become clickable after rendering to SVG.

```bash
dorkforge generate -t domain -v example.org -f json -o results/
dorkforge graph results/dorks.json -o results/pivots.dot
dot -Tsvg results/pivots.dot -o results/pivots.svg
```

### 9. Diff of two runs

```bash
dorkforge diff before/dorks.json after/dorks.json
dorkforge diff before/dorks.json after/dorks.json --format json -o changes.json
```

The comparison is performed on the full combination of provider and query, not on a short fingerprint, so rare eight-character SHA-256 collisions do not distort the result.

### 10. Template hot reload

For external templates, the generator can stay running and regenerate the output when a TOML file is added, removed, or modified:

```bash
dorkforge generate -t email -v analyst@example.org \
  --extra-templates ./my-templates --format all -o ./results --watch
```

By default, the check runs once per second; change the interval with `--watch-interval 3`. An invalid intermediate save of a template does not terminate the process: the previous artifacts are kept until the next valid change.

---

## Export Formats

### Clean TXT (`--format txt --mode clean`)

Outputs one query per line without service headers:

```text
"user@example.org" filetype:pdf
inbody:"user@example.org"
```

### Readable TXT (`--format txt --mode readable`)

Groups queries by search engine and category with Markdown headings:

```text
# DorkForge Generation Results
# Total queries: 17, Inputs: user@example.org

## Provider: GOOGLE
### Category: documents
"user@example.org" filetype:pdf
```

### Structured JSON (`--format json`)

A machine-readable result with a single schema, parameters, and detailed statistics:

```json
{
  "format_version": "1.0",
  "generator_version": "1.0.0",
  "parameters": { ... },
  "inputs": [ "user@example.org" ],
  "queries": [
    {
      "id": "google_email_pdf",
      "template_id": "google_email_pdf",
      "record_id": "dfq1_<sha256>",
      "template_version": null,
      "validation_status": "untested",
      "limitations": null,
      "identifier_type": "email",
      "identifier": "user@example.org",
      "variant": "user@example.org",
      "provider": "google",
      "category": "documents",
      "query": "\"user@example.org\" filetype:pdf",
      "tags": ["email", "documents"]
    }
  ],
  "stats": {
    "total_queries": 17,
    "unique_query_texts": 12,
    "unique_provider_queries": 17,
    "templates_used": 15,
    "queries_by_validation_status": { "untested": 17 },
    "queries_by_provider": { "google": 6, ... },
    "queries_by_category": { "documents": 6, ... },
    "warnings": []
  }
}
```

### HTML (`--format html`)

A standalone HTML document showing the template, validation status, limitations, query, and search link. Warnings and user-provided values are HTML-escaped; the report contains no external resources or scripts.

---

## Adding Custom Templates

The catalogs of individual search engines are supplemented by a small shared layer, [shared_templates.toml](config/templates/shared_templates.toml). It includes only queries whose syntax is common to the listed providers, with no exact duplicates of the provider catalogs. The `high_recall` label does not mean measured completeness; full coverage of search results, profiles, mentions, or subdomains is not claimed.

Templates are stored in TOML format and do not require recompiling the program. You can specify the template directory with the `--config-dir <path>` flag:

```toml
[[template]]
id = "custom_email_leaks"
enabled = true
identifier_type = "email"
category = "security_reports"
providers = ["google", "bing"]
template = '"{value}" (site:pastebin.com OR site:ghostbin.co)'
tags = ["email", "leaks"]
description = "Search indexed public pages for an email address."
rationale = "Find public pages that mention the identifier."
limitations = "Search results are incomplete and depend on provider indexing."
source_domains = ["pastebin.com", "ghostbin.co"]
template_version = 1

[template.validation]
status = "untested"
```

The legacy `validation = "untested"` and the `note` field continue to be read. For manual verification, use `[template.validation]` with `status`, `checked_at`, and `comment`.

---

## Testing and Verification

All tests run locally without internet access:

```bash
# Run all unit and integration tests
cargo test

# Check formatting
cargo fmt --check

# Lint with no warnings allowed
cargo clippy --all-targets -- -D warnings
```

---

## License

This project is distributed under the [MIT](LICENSE) license.