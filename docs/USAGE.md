# DorkForge: Usage Examples

All commands run locally: DorkForge generates queries and links to be opened manually, but does not perform searches, network connections, or scanning.

The examples assume the binary is available as `dorkforge`. For a local build, use `cargo run --` before the command, or `target/release/dorkforge` after `cargo build --release`.

## Table of Contents

- [Reference lists and shell completion](#reference-lists-and-shell-completion)
- [Quick start](#quick-start)
- [All identifier types](#all-identifier-types)
- [Input sources](#input-sources)
- [Filtering, variants, and limits](#filtering-variants-and-limits)
- [Formats and output locations](#formats-and-output-locations)
- [External templates](#external-templates)
- [Template hot reload](#template-hot-reload)
- [OSINT pivot graph](#osint-pivot-graph)
- [Diff of runs](#diff-of-runs)
- [Combined identifiers (Multi-Identifier OSINT)](#combined-identifiers-multi-identifier-osint)
- [Source coverage matrix (Coverage)](#source-coverage-matrix-coverage)
- [Template linting and validation (Lint Templates)](#template-linting-and-validation-lint-templates)

---

## Reference lists and shell completion

```bash
# Supported types, categories, and providers
dorkforge types
dorkforge categories
dorkforge providers

# Generate completion for the current shell
dorkforge completions zsh > ~/.zfunc/_dorkforge
dorkforge completions bash > ~/.local/share/bash-completion/completions/dorkforge
dorkforge completions fish > ~/.config/fish/completions/dorkforge.fish
```

## Quick start

```bash
# A single identifier, readable TXT to stdout
dorkforge generate -t domain -v example.org

# All providers and all artifacts into the results/ directory
dorkforge generate -t domain -v example.org -f all -o results

# Google and Brave only, specific categories
dorkforge generate -t email -v analyst@example.org \
  -p google,brave -c exact_match,documents,code -f html -o report.html
```

`-p` accepts `google`, `bing`, `yandex`, `brave`, `duckduckgo` (also `ddg`). The canonical categories are listed by the `dorkforge categories` command; the legacy aliases `profiles`, `code`, `archives`, `company`, `security`, and `tech` remain valid for filtering.

Provider templates are supplemented by the shared layer [shared_templates.toml](../config/templates/shared_templates.toml). Shared queries use only syntax that is suitable for all the listed search engines; exact duplicates of the provider catalogs are checked by tests. `high_recall` is not a completeness metric, and the generator does not promise full coverage of search results or subdomains.

## All identifier types

Always specifying `--type` explicitly is recommended: it eliminates the ambiguity of automatic detection.

```bash
dorkforge generate -t email         -v user@example.org
dorkforge generate -t wallet        -v 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --network eth
dorkforge generate -t username      -v @security_lead
dorkforge generate -t name          -v "Иван Иванов"
dorkforge generate -t domain        -v example.org
dorkforge generate -t ip            -v 2001:db8::1
dorkforge generate -t phone         -v "060 123 4567" --country RS
dorkforge generate -t license_plate -v "BG 123-AA" --country RS
dorkforge generate -t company       -v "Acme Corporation LLC"
dorkforge generate -t person_id     -v 7707083893 --country RU
dorkforge generate -t url           -v https://site.example/secret/passwords.txt
dorkforge generate -t vin           -v 1HGCR2F83HA000000
dorkforge generate -t file_hash     -v e4d909c290d0fb1ca068ffaddf22cbd0
dorkforge generate -t text_phrase   -v "CONFIDENTIAL PROJECT OMEGA"
dorkforge generate -t asn           -v AS15169
dorkforge generate -t certificate   -v 3B:A0:6B:7C:F4:A8:12:44:89:10:00:11:22:33:44:55
```

The `--country` and `--network` hints are not network enrichment: they only control local normalization and templates.

## Input sources

### Value from an argument

```bash
dorkforge generate --type email --value analyst@example.org
```

### A single value from stdin

```bash
printf '%s\n' 'analyst@example.org' | dorkforge generate -t email -v -
```

`--watch` is not available with stdin, because stdin is a one-shot source.

### Text batch file

One line is one identifier. For ambiguous lines, use JSON with explicit types.

```bash
dorkforge generate --input identifiers.txt -f both -o results
```

### JSON batch file

```json
[
  { "type": "email", "value": "analyst@example.org" },
  { "type": "domain", "value": "targetcorp.com" },
  { "type": "wallet", "value": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "network": "eth" },
  { "type": "license_plate", "value": "BG-123-AA", "country": "RS" }
]
```

```bash
dorkforge generate -i identifiers.json -f all -o results

# Stop the entire batch at the first invalid record
dorkforge generate -i identifiers.json --strict -f json -o results
```

Without `--strict`, invalid lines are skipped with a warning. For a machine-driven scenario without diagnostic messages, use `--quiet`.

## Filtering, variants, and limits

```bash
# Categories with short aliases
dorkforge generate -t domain -v example.org -c documents,profiles,code

# Restrict search engines
dorkforge generate -t username -v alice_dev -p google,duckduckgo

# Disable additional spelling variants
dorkforge generate -t phone -v "+381 60 1234567" --max-variants 1

# Limit the output volume for a batch run
dorkforge generate -i identifiers.json \
  --max-variants 5 \
  --max-queries-per-identifier 30 \
  --max-queries-per-provider 100
```

Generation remains deterministic: identical input, templates, and parameters produce the same query order. `fingerprint` is the first 8 hexadecimal characters of the SHA-256 of the query text and serves as a short label. `record_id` (`dfq1_` + SHA-256) stably identifies a "provider + query" pair; the former `query_id` keeps its semantics.

Strict deduplication collapses repeated whitespace but preserves the case of the value. Queries of different providers are not removed because of each other. In JSON, `unique_query_texts` counts distinct texts, `unique_provider_queries` counts "provider + text" pairs, and `templates_used` is the number of templates used.

## Formats and output locations

| Flag | Content | Without `-o` | In the `-o results` directory |
| --- | --- | --- | --- |
| `-f txt` | TXT | stdout | `results/dorks.txt` |
| `-f json` | JSON | stdout | `results/dorks.json` |
| `-f html` | HTML with links | stdout | `results/dorks.html` |
| `-f both` | TXT + JSON | TXT to stdout with a warning | `dorks.txt`, `dorks.json` |
| `-f all` | TXT + JSON + HTML | TXT to stdout with a warning | `dorks.txt`, `dorks.json`, `dorks.html` |

```bash
# Clean TXT: one query per line, convenient for pipes
dorkforge generate -t email -v analyst@example.org -f txt -m clean > dorks.txt

# Readable TXT: grouped by search engine, category, and variant
dorkforge generate -t email -v analyst@example.org -f txt -m readable

# Ready-made search URLs only, for TXT
dorkforge generate -t domain -v example.org --urls -m clean > urls.txt

# HTML report: ready-to-click links, no external scripts or resources
dorkforge generate -t domain -v example.org -f html -o report.html

# When a direct file name is given for several formats, the extension is replaced
dorkforge generate -t domain -v example.org -f all -o artifacts/report.txt
# Creates artifacts/report.txt, artifacts/report.json, artifacts/report.html
```

By default, an existing file is not overwritten. To overwrite deliberately, add `--overwrite`.

```bash
dorkforge generate -t domain -v example.org -f all -o results --overwrite
```

`--urls` is intended for TXT; HTML always contains links, and JSON keeps the original structured queries.

## External templates

```bash
# Fully replace the built-in template set
dorkforge generate -t email -v analyst@example.org --config-dir ./templates -f all -o results

# Add templates to the built-in set
dorkforge generate -t email -v analyst@example.org --extra-templates ./templates -f all -o results
```

A minimal TOML template:

```toml
[[template]]
id = "custom_email_mentions"
enabled = true
identifier_type = "email"
category = "mentions"
providers = ["google", "bing"]
template = '"{value}"'
tags = ["email", "custom"]
```

Available placeholders: `{value}`, `{domain}`, `{country}`. The provider and category are validated when the template is loaded. The optional fields `description`, `rationale`, `limitations`, `source_domains`, and `template_version` are saved in JSON if the corresponding field is part of the report format.

Template validation is specified as a table:

```toml
[template.validation]
status = "untested"
checked_at = "2026-10-16"
comment = "Manual search test has not been performed yet"
```

Allowed statuses: `untested`, `passed`, `partial`, `failed`, `stale`. `passed` may be specified only after manual verification; the project's regular tests do not contact search engines. The legacy string form `validation = "untested"` remains compatible.

Operator statuses and official links are located in `config/providers.toml`. `documentation_status` accepts `documented`, `experimental`, `unknown`, or `unsupported`; `test_status` stores the separate result of manual verification. Documentation status by itself does not mean the behavior has been verified.

## Template hot reload

`--watch` monitors TOML files in the external directory and regenerates the artifacts when the contents are added, removed, or modified. For the first run, existing files are still protected; add `--overwrite` if they already exist.

```bash
# Check once per second
dorkforge generate -t email -v analyst@example.org \
  --extra-templates ./templates --watch -f all -o results

# Check once every 3 seconds
dorkforge generate -t domain -v example.org \
  --config-dir ./templates --watch --watch-interval 3 -f html -o results
```

If an editor momentarily saves invalid TOML, DorkForge reports the error and keeps the previous artifacts. The next valid change regenerates the result. Stop the mode with `Ctrl-C`.

## OSINT pivot graph

The `graph` command accepts JSON created via `generate -f json` and outputs Graphviz DOT. The graph contains input identifiers, queries, categories, and providers. Query nodes have a URL attribute, so they become clickable after rendering to SVG.

```bash
dorkforge generate -t domain -v example.org -f json -o results
dorkforge graph results/dorks.json -o results/pivots.dot

# Optional rendering, if Graphviz is installed
dot -Tsvg results/pivots.dot -o results/pivots.svg

# Print DOT to stdout
dorkforge graph results/dorks.json

# Overwrite an existing file
dorkforge graph results/dorks.json -o results/pivots.dot --overwrite
```

## Diff of runs

Diff compares the full "provider + query" pair, not the short fingerprint. This stays reliable even with a theoretical collision of the 8-character fingerprint.

```bash
# Readable text report to stdout
dorkforge diff before/dorks.json after/dorks.json

# JSON report for automation
dorkforge diff before/dorks.json after/dorks.json -f json -o changes.json
```

The JSON diff contains the `added` and `removed` arrays, as well as the `unchanged` count.

## Combined identifiers (Multi-Identifier OSINT)

DorkForge supports simultaneous searching for combinations of several identifiers, when templates require several types of input data (for example, `username + domain`, `name + company`, `vin + license_plate`).

### A single combination from the CLI

```bash
# Search for a combination of a nickname and a corporate domain
dorkforge generate \
  --combine username:security_lead \
  --combine domain:corp.example.org \
  --format txt --mode readable

# Specification with explicit limits on the Cartesian product of variants
dorkforge generate \
  --combine name:"Иван Иванов" \
  --combine company:"ООО Вектор" \
  --max-combination-variants 4 \
  --max-variant-combinations 16 \
  --max-combination-queries 50 \
  --format json -o results
```

### Batch combinations file (JSON)

The file `examples/combinations.json` contains an array of combination scenarios:

```json
{
  "combinations": [
    {
      "id": "scenario_analyst_domain",
      "identifiers": [
        { "type": "username", "value": "analyst" },
        { "type": "domain", "value": "threatintel.org" }
      ]
    },
    {
      "id": "scenario_executive_company",
      "identifiers": [
        { "type": "name", "value": "John Doe" },
        { "type": "company", "value": "Acme Corp" }
      ]
    }
  ]
}
```

Running batch generation of combinations:

```bash
dorkforge generate -i examples/combinations.json -f all -o results
```

In the resulting JSON file, each combination query contains an `identifiers` field with a full trace of the types and variants used.

## Source coverage matrix (Coverage)

The `coverage` command analyzes the coverage density of categories, providers, and OSINT data sources (`config/sources.toml`).

```bash
# Print the coverage table to stdout
dorkforge coverage

# Export the matrix as JSON
dorkforge coverage --format json -o coverage.json

# Filter by a specific provider, category, or identifier type
dorkforge coverage --provider google --category public_documents
dorkforge coverage --type domain
```

## Template linting and validation (Lint Templates)

The `lint-templates` command performs an offline audit of TOML template directories for syntax errors, placeholders, compatibility of operators with search engine databases, and validity of documentation URL syntax:

```bash
# Check the built-in templates
dorkforge lint-templates

# Strict mode: any findings (warning/error) cause exit code 1
dorkforge lint-templates --strict

# Check an external directory of custom templates
dorkforge lint-templates --path ./my-custom-templates --strict
```