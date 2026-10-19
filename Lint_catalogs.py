# -*- coding: utf-8 -*-
#!/usr/bin/env python3
"""Validate built-in template catalogs. Usage: python3 Lint_catalogs.py [config_dir]."""
import glob
import os
import re
import sys
import tomllib


BUILTIN_CATEGORIES = {
    "exact_match", "mentions", "documents", "public_profiles", "forums",
    "code_repositories", "domain_pages", "subdomains", "archives",
    "public_records", "company_records", "security_reports",
    "technology_fingerprint", "marketplaces", "custom",
}
IDENTIFIER_TYPES = {
    "email", "wallet", "username", "name", "domain", "ip", "phone",
    "license_plate", "company", "person_id", "url", "vin", "file_hash",
    "text_phrase", "asn", "certificate",
}
DOCUMENTATION_STATUSES = {"documented", "experimental", "unknown", "unsupported"}
TEST_STATUSES = {"untested", "passed", "partial", "failed", "stale"}
PLACEHOLDERS = {"value", "domain", "country"}


def operators(template):
    text = re.sub(r'"[^"\\]*(?:\\.[^"\\]*)*"', '""', template)
    found = {f"{name}:" for name in re.findall(r"(?<![\w/.])([A-Za-z]+):", text)}
    found.update(re.findall(r"\b(?:OR|AND|NOT)\b", text))
    found.update(operator for operator in ("&&", "~~", "|") if operator in text)
    if re.search(r"(?:^|[\s(])-(?=[\w])", text):
        found.add("-")
    if '"' in template:
        found.add('"')
    if "+" in text:
        found.add("+")
    return found

base = os.path.dirname(os.path.abspath(__file__))
input_dir = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.join(base, "config")
if os.path.basename(input_dir) == "templates":
    template_dir = input_dir
    config_dir = os.path.dirname(input_dir)
else:
    config_dir = input_dir
    template_dir = os.path.join(config_dir, "templates")
providers_path = os.path.join(config_dir, "providers.toml")

with open(providers_path, "rb") as source:
    providers = {provider["id"]: provider for provider in tomllib.load(source)["providers"]}

errors, warnings, seen_ids, seen_queries = [], [], {}, {}
template_count = 0

for path in sorted(glob.glob(os.path.join(template_dir, "*.toml"))):
    with open(path, "rb") as source:
        catalog = tomllib.load(source)
    for template in catalog.get("template", []):
        template_count += 1
        location = f"{os.path.basename(path)}:{template.get('id', '<missing id>')}"
        template_id = template.get("id")
        if not template_id:
            errors.append(f"{location}: missing id")
        elif template_id in seen_ids:
            errors.append(f"{location}: duplicate id (also {seen_ids[template_id]})")
        else:
            seen_ids[template_id] = location

        identifier_type = template.get("identifier_type")
        if identifier_type not in IDENTIFIER_TYPES:
            errors.append(f"{location}: unknown identifier_type {identifier_type}")
        category = template.get("category")
        if category not in BUILTIN_CATEGORIES:
            errors.append(f"{location}: non-canonical category {category}")

        query_template = template.get("template", "")
        placeholders = set(re.findall(r"\{([^{}]*)\}", query_template))
        unknown_placeholders = placeholders - PLACEHOLDERS
        if unknown_placeholders:
            errors.append(f"{location}: unknown placeholders {sorted(unknown_placeholders)}")
        if "value" not in placeholders:
            errors.append(f"{location}: no {{value}} placeholder")
        if query_template.count("{") != query_template.count("}"):
            errors.append(f"{location}: malformed placeholder braces")

        validation = template.get("validation", "untested")
        if isinstance(validation, str):
            validation_status = validation
        elif isinstance(validation, dict):
            validation_status = validation.get("status", "untested")
        else:
            validation_status = None
        if validation_status not in TEST_STATUSES:
            errors.append(f"{location}: invalid validation status {validation_status}")

        for provider_id in template.get("providers", []):
            provider = providers.get(provider_id)
            if provider is None:
                errors.append(f"{location}: unknown provider {provider_id}")
                continue
            if category not in provider.get("supported_categories", []):
                errors.append(f"{location}: category {category} is not listed for {provider_id}")

            used_operators = operators(query_template)
            unsupported = used_operators - set(provider.get("supported_operators", []))
            if unsupported:
                errors.append(f"{location}: unsupported operators for {provider_id}: {sorted(unsupported)}")

            metadata = provider.get("operator_metadata", {})
            for operator in used_operators:
                details = metadata.get(operator)
                if details is None:
                    errors.append(f"{location}: {provider_id} operator {operator} has no metadata")
                elif details.get("documentation_status") not in DOCUMENTATION_STATUSES:
                    errors.append(f"{location}: invalid documentation status for {provider_id} {operator}")
                elif details.get("test_status") not in TEST_STATUSES:
                    errors.append(f"{location}: invalid test status for {provider_id} {operator}")

            if provider.get("file_types"):
                for file_type in re.findall(r"(?:filetype|ext|mime):(\w+)", query_template):
                    if file_type not in provider["file_types"]:
                        errors.append(f"{location}: file type {file_type} is not listed for {provider_id}")

            if template.get("enabled", True):
                key = (provider_id, identifier_type, query_template)
                if key in seen_queries:
                    warnings.append(f"{location}: duplicate query for {provider_id} (also {seen_queries[key]})")
                else:
                    seen_queries[key] = location

for warning in warnings:
    print("WARN ", warning)
for error in errors:
    print("ERROR", error)
print(f"{template_count} templates, {len(errors)} errors, {len(warnings)} warnings")
sys.exit(1 if errors else 0)
#!/usr/bin/env python3
"""Consistency lint for DorkForge catalogs. Usage: python lint_catalogs.py [dir]
Checks every *_templates.toml against providers.toml. Exit code 1 on errors."""
import re, sys, glob, os, tomllib
D = sys.argv[1] if len(sys.argv) > 1 else "."
TYPES = {"email","wallet","username","name","domain","ip","phone","license_plate","company",
         "person_id","url","vin","file_hash","text_phrase","asn","certificate"}
prov = {p["id"]: p for p in tomllib.load(open(os.path.join(D, "providers.toml"), "rb"))["providers"]}
errors, warns, seen_ids, seen_q = [], [], {}, {}

def operators(tpl):
    t = re.sub(r'"[^"]*"', '""', tpl)                       # drop quoted phrases
    ops = {m + ":" for m in re.findall(r'(?<![\w/.])([A-Za-z]+):', t)}
    if re.search(r'\sOR\s', t): ops.add("OR")
    if re.search(r'\sAND\s', t): ops.add("AND")
    if re.search(r'\sNOT\s', t): ops.add("NOT")
    if "|" in t: ops.add("|")
    if "~~" in t: ops.add("~~")
    if re.search(r'(?:^|[\s(])-(?=[\w])', t): ops.add("-")
    return ops

for f in sorted(glob.glob(os.path.join(D, "*_templates.toml"))):
    for t in tomllib.load(open(f, "rb")).get("template", []):
        where = f'{os.path.basename(f)}:{t.get("id")}'
        if t["id"] in seen_ids: errors.append(f"{where}: duplicate id (also {seen_ids[t['id']]})")
        seen_ids[t["id"]] = where
        if t["identifier_type"] not in TYPES: errors.append(f"{where}: unknown identifier_type {t['identifier_type']}")
        tpl = t["template"]
        if "{value}" not in tpl: errors.append(f"{where}: no {{value}} placeholder")
        if re.search(r'\{(?!value\})', tpl): errors.append(f"{where}: unknown placeholder")
        for p in t["providers"]:
            if p not in prov: errors.append(f"{where}: unknown provider {p}"); continue
            P = prov[p]
            if t["category"] not in P["supported_categories"]:
                errors.append(f"{where}: category {t['category']} not supported by {p}")
            bad = operators(tpl) - set(P["supported_operators"])
            if bad: errors.append(f"{where}: operators {sorted(bad)} not supported by {p}: {tpl}")
            if re.search(r'~~\s*\w+:|\w+:\S+\s*~~', tpl):
                errors.append(f"{where}: ~~ is a word-level operator, not valid next to site:/host:/url: : {tpl}")
            if P.get("file_types"):
                for ft in re.findall(r'(?:filetype|ext|mime):(\w+)', tpl):
                    if ft not in P["file_types"]: errors.append(f"{where}: file type {ft} not supported by {p}")
            if t.get("enabled", True):
                k = (p, t["identifier_type"], tpl)
                if k in seen_q: warns.append(f"{where}: same query as {seen_q[k]} for {p} (will be deduplicated)")
                seen_q[k] = where
for w in warns: print("WARN ", w)
for e in errors: print("ERROR", e)
print(f"{len(seen_ids)} templates, {len(errors)} errors, {len(warns)} warnings")
sys.exit(1 if errors else 0)