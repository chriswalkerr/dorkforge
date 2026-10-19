use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

use crate::combination::{
    plan_variant_combinations, substitute_combination_template, validate_and_resolve_combination,
    CombinationPlannerLimits, CombinationSpec,
};
use crate::error::Result;
use crate::generator::dedup::{DedupPolicy, QueryDeduplicator};
use crate::identifiers::ParsedIdentifier;
use crate::providers::Provider;
use crate::templates::{TemplateRegistry, ValidationStatus};
use crate::variants::{VariantConfig, VariantGenerator};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryIdentifierTrace {
    pub r#type: String,
    pub value: String,
    pub variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DorkQuery {
    pub query_id: String,
    #[serde(default)]
    pub record_id: String,
    pub id: String,
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub template_version: Option<u32>,
    #[serde(default)]
    pub validation_status: ValidationStatus,
    #[serde(default)]
    pub limitations: Option<String>,
    pub fingerprint: String,
    pub identifier_type: String,
    pub identifier: String,
    pub variant: String,
    pub provider: String,
    pub category: String,
    pub query: String,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifiers: Option<Vec<QueryIdentifierTrace>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationParams {
    pub providers: Vec<String>,
    pub categories: Option<Vec<String>>,
    pub max_variants: usize,
    pub max_queries_per_identifier: usize,
    pub max_queries_per_provider: usize,
    pub country: Option<String>,
    pub network_or_asset: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationStats {
    pub total_queries: usize,
    #[serde(default)]
    pub unique_query_texts: usize,
    #[serde(default)]
    pub unique_provider_queries: usize,
    #[serde(default)]
    pub templates_used: usize,
    #[serde(default)]
    pub queries_by_validation_status: BTreeMap<String, usize>,
    pub queries_by_provider: BTreeMap<String, usize>,
    pub queries_by_category: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationResult {
    pub format_version: String,
    pub generator_version: String,
    pub parameters: GenerationParams,
    pub inputs: Vec<String>,
    pub queries: Vec<DorkQuery>,
    pub stats: GenerationStats,
}

#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    pub providers: Vec<Provider>,
    pub categories: Option<Vec<String>>,
    pub max_variants: usize,
    pub max_queries_per_identifier: usize,
    pub max_queries_per_provider: usize,
    pub max_combination_variants: usize,
    pub max_variant_combinations: usize,
    pub max_combination_queries: usize,
    pub dedup_policy: DedupPolicy,
    pub country_hint: Option<String>,
    pub network_hint: Option<String>,
    pub strict: bool,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            providers: Provider::ALL.to_vec(),
            categories: None,
            max_variants: 10,
            max_queries_per_identifier: 50,
            max_queries_per_provider: 100,
            max_combination_variants: 5,
            max_variant_combinations: 25,
            max_combination_queries: 100,
            dedup_policy: DedupPolicy::Strict,
            country_hint: None,
            network_hint: None,
            strict: false,
        }
    }
}

pub struct DorkGenerator<'a> {
    registry: &'a TemplateRegistry,
    options: GeneratorOptions,
}

impl<'a> DorkGenerator<'a> {
    pub fn new(registry: &'a TemplateRegistry, options: GeneratorOptions) -> Self {
        Self { registry, options }
    }

    fn resolve_domain(&self, identifier: &ParsedIdentifier) -> String {
        match identifier {
            ParsedIdentifier::Domain(d) => d.domain.clone(),
            ParsedIdentifier::Email(e) => e.domain.clone(),
            ParsedIdentifier::Url(u) => u.host.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn resolve_country(&self, identifier: &ParsedIdentifier) -> String {
        if let Some(ref c) = self.options.country_hint {
            return c.clone();
        }
        match identifier {
            ParsedIdentifier::LicensePlate(p) => p.country.clone().unwrap_or_default(),
            ParsedIdentifier::PersonId(pid) => pid.country.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn substitute_template(
        &self,
        template_str: &str,
        variant: &str,
        domain: &str,
        country: &str,
    ) -> String {
        let mut result = template_str.replace("{value}", variant);
        result = result.replace("{domain}", domain);
        result = result.replace("{country}", country);
        result.trim().to_string()
    }

    pub fn generate_for_identifiers(
        &self,
        identifiers: &[ParsedIdentifier],
    ) -> Result<GenerationResult> {
        self.generate_all(identifiers, &[])
    }

    pub fn generate_for_combinations(
        &self,
        combinations: &[CombinationSpec],
    ) -> Result<GenerationResult> {
        self.generate_all(&[], combinations)
    }

    pub fn generate_all(
        &self,
        identifiers: &[ParsedIdentifier],
        combinations: &[CombinationSpec],
    ) -> Result<GenerationResult> {
        let variant_gen = VariantGenerator::new(VariantConfig {
            max_variants_per_identifier: self.options.max_variants,
        });

        let mut deduplicator = QueryDeduplicator::new(self.options.dedup_policy);
        let mut all_queries: Vec<DorkQuery> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        let mut provider_counts: BTreeMap<String, usize> = BTreeMap::new();
        for p in &self.options.providers {
            provider_counts.insert(p.id().to_string(), 0);
        }

        let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut validation_counts: BTreeMap<String, usize> = BTreeMap::new();

        // 1. Generate for single identifiers
        for parsed_id in identifiers {
            let id_type = parsed_id.identifier_type();
            let raw_val = parsed_id.raw_value().to_string();
            let variants = variant_gen.generate(parsed_id);
            let domain_val = self.resolve_domain(parsed_id);
            let country_val = self.resolve_country(parsed_id);

            let mut id_query_count = 0;
            let mut id_hit_limit = false;

            for provider in &self.options.providers {
                let templates = self.registry.find_templates(
                    id_type,
                    *provider,
                    self.options.categories.as_deref(),
                );

                for template in templates {
                    for variant in &variants {
                        if id_query_count >= self.options.max_queries_per_identifier {
                            if !id_hit_limit {
                                warnings.push(format!(
                                    "Identifier '{}' ({}) reached maximum query limit of {}",
                                    raw_val, id_type, self.options.max_queries_per_identifier
                                ));
                                id_hit_limit = true;
                            }
                            break;
                        }

                        let p_count = provider_counts
                            .entry(provider.id().to_string())
                            .or_insert(0);
                        if *p_count >= self.options.max_queries_per_provider {
                            let warn_msg = format!(
                                "Provider '{}' reached maximum query limit of {}",
                                provider.id(),
                                self.options.max_queries_per_provider
                            );
                            if !warnings.contains(&warn_msg) {
                                warnings.push(warn_msg);
                            }
                            break;
                        }

                        let rendered_query = self.substitute_template(
                            &template.template,
                            variant,
                            &domain_val,
                            &country_val,
                        );

                        if rendered_query.is_empty() {
                            continue;
                        }

                        if deduplicator.is_unique_and_mark(provider.id(), &rendered_query) {
                            let hash_bytes = Sha256::digest(rendered_query.as_bytes());
                            let hash: String =
                                hash_bytes.iter().map(|b| format!("{b:02x}")).collect();
                            let fingerprint = hash[..8].to_string();
                            let query_id =
                                format!("{}_{}_{}", template.id, provider.id(), fingerprint);
                            let stable_key = format!(
                                "dorkforge-record-v1\0{}\0{}",
                                provider.id(),
                                rendered_query
                            );
                            let stable_hash = Sha256::digest(stable_key.as_bytes());
                            let record_id = stable_hash
                                .iter()
                                .map(|byte| format!("{byte:02x}"))
                                .collect::<String>();
                            let validation_status = template.effective_validation_status();
                            *validation_counts
                                .entry(validation_status.as_str().to_string())
                                .or_insert(0) += 1;

                            all_queries.push(DorkQuery {
                                query_id,
                                record_id: format!("dfq1_{record_id}"),
                                id: template.id.clone(),
                                template_id: template.id.clone(),
                                template_version: template.template_version,
                                validation_status,
                                limitations: template
                                    .limitations
                                    .clone()
                                    .or_else(|| template.note.clone()),
                                fingerprint,
                                identifier_type: id_type.as_str().to_string(),
                                identifier: raw_val.clone(),
                                variant: variant.clone(),
                                provider: provider.id().to_string(),
                                category: template.category.clone(),
                                query: rendered_query,
                                tags: template.tags.clone(),
                                identifiers: None,
                            });

                            id_query_count += 1;
                            *p_count += 1;
                            *category_counts
                                .entry(template.category.clone())
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // 2. Generate for combination specifications
        for combo in combinations {
            let resolved = validate_and_resolve_combination(
                combo,
                self.options.max_combination_variants,
                self.options.strict,
            )?;

            let types: Vec<crate::identifiers::IdentifierType> =
                resolved.iter().map(|r| r.id_type).collect();

            let planner_limits = CombinationPlannerLimits {
                max_variant_combinations: self.options.max_variant_combinations,
                max_combination_queries: self.options.max_combination_queries,
                strict: self.options.strict,
            };

            let bindings = plan_variant_combinations(&resolved, planner_limits, &mut warnings)?;

            let target_providers: Vec<Provider> = if let Some(ref combo_providers) = combo.providers
            {
                let mut filtered = Vec::new();
                for p_str in combo_providers {
                    if let Ok(p) = p_str.parse::<Provider>() {
                        if self.options.providers.contains(&p) {
                            filtered.push(p);
                        }
                    }
                }
                filtered
            } else {
                self.options.providers.clone()
            };

            let mut combo_query_count = 0;

            for provider in &target_providers {
                let categories_filter = combo
                    .categories
                    .as_deref()
                    .or(self.options.categories.as_deref());

                let templates =
                    self.registry
                        .find_combination_templates(&types, *provider, categories_filter);

                for template in templates {
                    for binding in &bindings {
                        if combo_query_count >= self.options.max_combination_queries {
                            let warn_msg = format!(
                                "Combination '{}' reached maximum query limit of {}",
                                combo.id, self.options.max_combination_queries
                            );
                            if !warnings.contains(&warn_msg) {
                                warnings.push(warn_msg);
                            }
                            break;
                        }

                        let p_count = provider_counts
                            .entry(provider.id().to_string())
                            .or_insert(0);
                        if *p_count >= self.options.max_queries_per_provider {
                            break;
                        }

                        let rendered_query =
                            substitute_combination_template(&template.template, binding);

                        if rendered_query.is_empty() {
                            continue;
                        }

                        if deduplicator.is_unique_and_mark(provider.id(), &rendered_query) {
                            let hash_bytes = Sha256::digest(rendered_query.as_bytes());
                            let hash: String =
                                hash_bytes.iter().map(|b| format!("{b:02x}")).collect();
                            let fingerprint = hash[..8].to_string();
                            let query_id =
                                format!("{}_{}_{}", template.id, provider.id(), fingerprint);
                            let stable_key = format!(
                                "dorkforge-record-v1\0{}\0{}",
                                provider.id(),
                                rendered_query
                            );
                            let stable_hash = Sha256::digest(stable_key.as_bytes());
                            let record_id = stable_hash
                                .iter()
                                .map(|byte| format!("{byte:02x}"))
                                .collect::<String>();
                            let validation_status = template.effective_validation_status();
                            *validation_counts
                                .entry(validation_status.as_str().to_string())
                                .or_insert(0) += 1;

                            let trace = resolved
                                .iter()
                                .map(|r| QueryIdentifierTrace {
                                    r#type: r.id_type.as_str().to_string(),
                                    value: r.original_value.clone(),
                                    variant: binding
                                        .variants_by_type
                                        .get(r.id_type.as_str())
                                        .cloned()
                                        .unwrap_or_else(|| "original".to_string()),
                                })
                                .collect();

                            let combo_type_str = types
                                .iter()
                                .map(|t| t.as_str())
                                .collect::<Vec<_>>()
                                .join("+");

                            all_queries.push(DorkQuery {
                                query_id,
                                record_id: format!("dfq1_{record_id}"),
                                id: template.id.clone(),
                                template_id: template.id.clone(),
                                template_version: template.template_version,
                                validation_status,
                                limitations: template
                                    .limitations
                                    .clone()
                                    .or_else(|| template.note.clone()),
                                fingerprint,
                                identifier_type: combo_type_str,
                                identifier: combo.id.clone(),
                                variant: "combination".to_string(),
                                provider: provider.id().to_string(),
                                category: template.category.clone(),
                                query: rendered_query,
                                tags: template.tags.clone(),
                                identifiers: Some(trace),
                            });

                            combo_query_count += 1;
                            *p_count += 1;
                            *category_counts
                                .entry(template.category.clone())
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Sort deterministically: provider, category, id, query
        all_queries.sort_by(|a, b| {
            a.provider
                .cmp(&b.provider)
                .then_with(|| a.category.cmp(&b.category))
                .then_with(|| a.id.cmp(&b.id))
                .then_with(|| a.query.cmp(&b.query))
        });

        let total_queries = all_queries.len();
        let unique_query_texts = all_queries
            .iter()
            .map(|query| query.query.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let unique_provider_queries = all_queries
            .iter()
            .map(|query| (query.provider.as_str(), query.query.as_str()))
            .collect::<BTreeSet<_>>()
            .len();
        let templates_used = all_queries
            .iter()
            .map(|query| query.template_id.as_str())
            .collect::<BTreeSet<_>>()
            .len();

        let params = GenerationParams {
            providers: self
                .options
                .providers
                .iter()
                .map(|p| p.id().to_string())
                .collect(),
            categories: self.options.categories.clone(),
            max_variants: self.options.max_variants,
            max_queries_per_identifier: self.options.max_queries_per_identifier,
            max_queries_per_provider: self.options.max_queries_per_provider,
            country: self.options.country_hint.clone(),
            network_or_asset: self.options.network_hint.clone(),
        };

        let mut inputs: Vec<String> = identifiers
            .iter()
            .map(|i| i.raw_value().to_string())
            .collect();
        for c in combinations {
            inputs.push(c.id.clone());
        }

        let stats = GenerationStats {
            total_queries,
            unique_query_texts,
            unique_provider_queries,
            templates_used,
            queries_by_validation_status: validation_counts,
            queries_by_provider: provider_counts,
            queries_by_category: category_counts,
            warnings,
        };

        Ok(GenerationResult {
            format_version: "1.0".to_string(),
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
            parameters: params,
            inputs,
            queries: all_queries,
            stats,
        })
    }
}
