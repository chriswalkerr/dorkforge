# Operator Manual Verification Matrix

This matrix is intended for manual runs in search engine interfaces. DorkForge and the project's regular tests do not perform network search requests. All rows are currently `untested`; the actual observation and date must be filled in after manual verification. An operator being documented does not mean its behavior has passed an empirical test.

`probe` and `example.com` are used as synthetic test values. For comparison, check the result against the same query without the operator under test. For unknown syntax, the goal of the check is to establish whether the filter is applied, whether the operator is ignored, or whether the parsing of the query changes.

## Table of Contents

- [Google](#google)
- [Bing](#bing)
- [Yandex](#yandex)
- [Brave](#brave)
- [DuckDuckGo](#duckduckgo)
- [Sources](#sources)

---

## Google

| Operator | Test query | Expected behavior | Observation | Date | Status |
| --- | --- | --- | --- | --- | --- |
| `site:` | `probe site:example.com` | Results are limited to the specified site/domain | Not checked | - | `untested` |
| `filetype:` | `probe filetype:pdf` | Results are limited to PDF documents | Not checked | - | `untested` |
| `ext:` | `probe ext:pdf` | Check whether results are limited by file extension | Not checked | - | `untested` |
| `inurl:` | `probe inurl:example` | Check filtering of a term by URL | Not checked | - | `untested` |
| `allinurl:` | `probe allinurl:example test` | Check whether all following terms are restricted to the URL | Not checked | - | `untested` |
| `intitle:` | `probe intitle:example` | Check filtering of a term by title | Not checked | - | `untested` |
| `allintitle:` | `probe allintitle:example test` | Check whether all following terms are restricted to the title | Not checked | - | `untested` |
| `intext:` | `probe intext:example` | Check filtering of a term by page text | Not checked | - | `untested` |
| `allintext:` | `probe allintext:example test` | Check whether the page text is restricted by all following terms | Not checked | - | `untested` |
| `OR` | `example OR sample` | Results match one of the alternatives | Not checked | - | `untested` |
| `AND` | `example AND sample` | Check explicit logical intersection | Not checked | - | `untested` |
| `-` | `example -sample` | Results exclude the term `sample` | Not checked | - | `untested` |
| `"` | `"exact phrase probe"` | Exact phrase search | Not checked | - | `untested` |

## Bing

| Operator | Test query | Expected behavior | Observation | Date | Status |
| --- | --- | --- | --- | --- | --- |
| `site:` | `probe site:example.com` | Results are limited to the site/domain | Not checked | - | `untested` |
| `filetype:` | `probe filetype:pdf` | Results are limited to PDF documents | Not checked | - | `untested` |
| `ext:` | `probe ext:pdf` | Results are limited by file extension | Not checked | - | `untested` |
| `inurl:` | `probe inurl:example` | Official behavior not confirmed; check filtering by URL | Not checked | - | `untested` |
| `intitle:` | `probe intitle:example` | Check filtering of a term by title metadata | Not checked | - | `untested` |
| `inbody:` | `probe inbody:example` | The term is in the page text; assess reliability separately | Not checked | - | `untested` |
| `inanchor:` | `probe inanchor:example` | The term is in the link text | Not checked | - | `untested` |
| `feed:` | `feed:probe` | Finds RSS/Atom feeds for the term | Not checked | - | `untested` |
| `hasfeed:` | `probe site:example.com hasfeed:probe` | Finds pages of the site that contain an RSS/Atom feed | Not checked | - | `untested` |
| `ip:` | `probe ip:8.8.8.8` | Results relate to sites on the specified IPv4 | Not checked | - | `untested` |
| `loc:` | `probe loc:US` | Results are limited to the specified region | Not checked | - | `untested` |
| `language:` | `probe language:en` | Results are limited to the specified language | Not checked | - | `untested` |
| `contains:` | `probe contains:pdf` | Finds pages with links to the specified file type | Not checked | - | `untested` |
| `url:` | `url:example.com` | Checks whether the domain/URL is present in the index | Not checked | - | `untested` |
| `OR` | `example OR sample` | Results match one of the alternatives | Not checked | - | `untested` |
| `AND` | `example AND sample` | Results contain both terms/conditions | Not checked | - | `untested` |
| `NOT` | `example NOT sample` | Results exclude the condition `sample` | Not checked | - | `untested` |
| `-` | `example -sample` | Results exclude the term `sample` | Not checked | - | `untested` |
| `"` | `"exact phrase probe"` | Exact phrase search | Not checked | - | `untested` |

## Yandex

| Operator | Test query | Expected behavior | Observation | Date | Status |
| --- | --- | --- | --- | --- | --- |
| `site:` | `probe site:example.com` | Search on the site, its subdomains, and pages | Not checked | - | `untested` |
| `host:` | `probe host:example.com` | Search on a specific main host | Not checked | - | `untested` |
| `rhost:` | `probe rhost:com.example` | Search on a host whose name is in reverse order | Not checked | - | `untested` |
| `url:` | `probe url:example.com/path` | Search for pages by address or URL prefix | Not checked | - | `untested` |
| `domain:` | `probe domain:com` | Restriction by domain zone | Not checked | - | `untested` |
| `inurl:` | `probe inurl:example` | Official behavior is not confirmed by the current help; check URL parsing | Not checked | - | `untested` |
| `title:` | `probe title:example` | Official behavior is not confirmed by the current help; check the title filter | Not checked | - | `untested` |
| `mime:` | `probe mime:pdf` | Results are limited to documents of a supported type | Not checked | - | `untested` |
| `lang:` | `probe lang:en` | Results are limited by language | Not checked | - | `untested` |
| `date:` | `probe date:20200101` | Results are limited by publication date | Not checked | - | `untested` |
| `\|` | `example \| sample` | Results contain one or more of the alternative terms | Not checked | - | `untested` |
| `&&` | `example && sample` | Official behavior not confirmed; check explicit logical intersection | Not checked | - | `untested` |
| `~~` | `example ~~ sample` | Official behavior not confirmed; check the word operator separately from `site:` | Not checked | - | `untested` |
| `-` | `example -sample` | Word exclusion; check the positional restriction at the end of the query | Not checked | - | `untested` |
| `"` | `"exact phrase probe"` | Search for a phrase in the given word order | Not checked | - | `untested` |

## Brave

| Operator | Test query | Expected behavior | Observation | Date | Status |
| --- | --- | --- | --- | --- | --- |
| `site:` | `probe site:example.com` | Results are limited to the site | Not checked | - | `untested` |
| `inurl:` | `probe inurl:example` | Not in the confirmed allowlist; check whether it is ignored or applied | Not checked | - | `untested` |
| `filetype:` | `probe filetype:pdf` | Results are limited to the specified file type | Not checked | - | `untested` |
| `ext:` | `probe ext:pdf` | Results are limited by file extension | Not checked | - | `untested` |
| `intitle:` | `probe intitle:example` | The term is present in the page title | Not checked | - | `untested` |
| `inbody:` | `probe inbody:example` | The term is present in the page body | Not checked | - | `untested` |
| `inpage:` | `probe inpage:example` | The term is present in the page title or body | Not checked | - | `untested` |
| `loc:` | `probe loc:ca` | Results are limited to the country/region | Not checked | - | `untested` |
| `lang:` | `probe lang:en` | Results are limited by language | Not checked | - | `untested` |
| `OR` | `example OR sample` | Results match one of the alternatives | Not checked | - | `untested` |
| `AND` | `example AND sample` | Results match both conditions | Not checked | - | `untested` |
| `NOT` | `example NOT sample` | Results exclude the condition `sample` | Not checked | - | `untested` |
| `-` | `example -sample` | Excludes results with the term `sample` | Not checked | - | `untested` |
| `+` | `example +sample` | Raises the priority of pages with the term `sample` | Not checked | - | `untested` |
| `"` | `"exact phrase probe"` | Exact phrase search | Not checked | - | `untested` |

## DuckDuckGo

| Operator | Test query | Expected behavior | Observation | Date | Status |
| --- | --- | --- | --- | --- | --- |
| `site:` | `probe site:example.com` | Results are limited to the specified site | Not checked | - | `untested` |
| `filetype:` | `probe filetype:pdf` | Results are limited to documented file types | Not checked | - | `untested` |
| `inurl:` | `probe inurl:example` | The URL contains the specified term; check the single-word restriction | Not checked | - | `untested` |
| `intitle:` | `probe intitle:example` | The page title contains the specified term | Not checked | - | `untested` |
| `OR` | `example OR sample` | Not listed in the official help; check whether it is treated as an ordinary term | Not checked | - | `untested` |
| `AND` | `example AND sample` | Not listed in the official help; check whether it is treated as an ordinary term | Not checked | - | `untested` |
| `-` | `example -sample` | The number of results with the term `sample` decreases | Not checked | - | `untested` |
| `+` | `example +sample` | The priority of pages with the term `sample` is raised | Not checked | - | `untested` |
| `"` | `"exact phrase probe"` | Exact phrase search; check the fallback to similar results | Not checked | - | `untested` |

---

## Sources

- Google: <https://support.google.com/websearch/answer/2466433>
- Bing: <https://support.microsoft.com/ru-ru/bing/advanced-search-options> and <https://support.microsoft.com/ru-ru/bing/advanced-search-keywords>
- Yandex: <https://yandex.com/support/search/en/query-language/qlanguage.md>, <https://yandex.com/support/search/en/query-language/search-operators.md>, and <https://yandex.com/support/search/en/query-language/search-context.md>
- Brave: <https://search.brave.com/help/operators>
- DuckDuckGo: <https://duckduckgo.com/duckduckgo-help-pages/results/syntax/>