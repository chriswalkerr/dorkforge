use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::coverage::source_profile::SourceRegistry;
use crate::identifiers::IdentifierType;
use crate::providers::Provider;
use crate::templates::{category_matches, TemplateRegistry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CoverageCell {
    pub enabled_templates: usize,
    pub disabled_templates: usize,
    pub combination_templates: usize,
    pub operator_status: String,
    pub uncoverage_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CoverageFilter {
    pub provider: Option<String>,
    pub category: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageMatrix {
    pub filter: CoverageFilter,
    pub total_templates: usize,
    pub total_source_profiles: usize,
    /// Category -> Provider -> CoverageCell
    pub category_by_provider: BTreeMap<String, BTreeMap<String, CoverageCell>>,
    /// IdentifierType -> Category -> count
    pub type_by_category: BTreeMap<String, BTreeMap<String, usize>>,
    /// Combination ("type1+type2") -> Category -> count
    pub combination_by_category: BTreeMap<String, BTreeMap<String, usize>>,
    /// SourceProfile -> Provider -> CoverageCell
    pub source_by_provider: BTreeMap<String, BTreeMap<String, CoverageCell>>,
    /// Operator validation status -> count
    pub operator_status_summary: BTreeMap<String, usize>,
}

impl CoverageMatrix {
    pub fn build(
        registry: &TemplateRegistry,
        sources: &SourceRegistry,
        filter: CoverageFilter,
    ) -> Self {
        let all_providers = Provider::ALL;
        let mut target_providers: Vec<Provider> = Vec::new();
        for p in all_providers {
            if let Some(ref req_p) = filter.provider {
                if !p.id().eq_ignore_ascii_case(req_p) {
                    continue;
                }
            }
            target_providers.push(*p);
        }

        // Collect categories
        let mut target_categories: BTreeSet<String> = BTreeSet::new();
        for t in registry.templates() {
            if let Some(ref req_c) = filter.category {
                if !category_matches(req_c, &t.category) {
                    continue;
                }
            }
            target_categories.insert(t.category.clone());
        }
        if let Some(ref req_c) = filter.category {
            target_categories.insert(req_c.clone());
        }

        // 1. Category x Provider
        let mut category_by_provider: BTreeMap<String, BTreeMap<String, CoverageCell>> =
            BTreeMap::new();

        for cat in &target_categories {
            let mut provider_cells: BTreeMap<String, CoverageCell> = BTreeMap::new();
            for provider in &target_providers {
                let mut enabled = 0;
                let mut disabled = 0;
                let mut combinations = 0;
                let mut uncoverage_reasons = Vec::new();

                for t in registry.templates() {
                    if !category_matches(cat, &t.category) {
                        continue;
                    }
                    if !t.providers.contains(provider) {
                        continue;
                    }

                    if let Some(ref req_t) = filter.r#type {
                        let matches_type = if t.is_combination() {
                            t.identifier_types.iter().any(|it| it.as_str() == req_t)
                        } else {
                            t.identifier_type.is_some_and(|it| it.as_str() == req_t)
                        };
                        if !matches_type {
                            continue;
                        }
                    }

                    if t.enabled {
                        enabled += 1;
                        if t.is_combination() {
                            combinations += 1;
                        }
                    } else {
                        disabled += 1;
                    }
                }

                if enabled == 0 {
                    uncoverage_reasons.push(format!(
                        "No enabled templates in catalog for category '{cat}' on provider '{}'",
                        provider.id()
                    ));
                }

                provider_cells.insert(
                    provider.id().to_string(),
                    CoverageCell {
                        enabled_templates: enabled,
                        disabled_templates: disabled,
                        combination_templates: combinations,
                        operator_status: "verified_operators".to_string(),
                        uncoverage_reasons,
                    },
                );
            }
            category_by_provider.insert(cat.clone(), provider_cells);
        }

        // 2. Identifier Type x Category
        let mut type_by_category: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for id_type in IdentifierType::ALL {
            if let Some(ref req_t) = filter.r#type {
                if !id_type.as_str().eq_ignore_ascii_case(req_t) {
                    continue;
                }
            }

            let mut cat_counts = BTreeMap::new();
            for cat in &target_categories {
                let count = registry
                    .templates()
                    .iter()
                    .filter(|t| {
                        if !t.enabled {
                            return false;
                        }
                        if t.is_combination() {
                            return false;
                        }
                        t.identifier_type == Some(*id_type) && category_matches(cat, &t.category)
                    })
                    .count();
                cat_counts.insert(cat.clone(), count);
            }
            type_by_category.insert(id_type.as_str().to_string(), cat_counts);
        }

        // 3. Combination x Category
        let mut combination_by_category: BTreeMap<String, BTreeMap<String, usize>> =
            BTreeMap::new();
        let mut combo_keys = BTreeSet::new();
        for t in registry.templates() {
            if t.is_combination() && t.enabled {
                let key = t
                    .identifier_types
                    .iter()
                    .map(|it| it.as_str())
                    .collect::<Vec<_>>()
                    .join("+");
                combo_keys.insert(key);
            }
        }

        for combo in &combo_keys {
            let mut cat_counts = BTreeMap::new();
            for cat in &target_categories {
                let count = registry
                    .templates()
                    .iter()
                    .filter(|t| {
                        if !t.enabled || !t.is_combination() {
                            return false;
                        }
                        let key = t
                            .identifier_types
                            .iter()
                            .map(|it| it.as_str())
                            .collect::<Vec<_>>()
                            .join("+");
                        key == *combo && category_matches(cat, &t.category)
                    })
                    .count();
                cat_counts.insert(cat.clone(), count);
            }
            combination_by_category.insert(combo.clone(), cat_counts);
        }

        // 4. SourceProfile x Provider
        let mut source_by_provider: BTreeMap<String, BTreeMap<String, CoverageCell>> =
            BTreeMap::new();

        for source in sources.profiles() {
            let mut provider_cells = BTreeMap::new();
            for provider in &target_providers {
                let mut matching_templates = 0;
                let mut uncoverage_reasons = Vec::new();

                for t in registry.templates() {
                    if !t.enabled || !t.providers.contains(provider) {
                        continue;
                    }

                    // Check if template matches source domains or source category
                    let domain_match = source
                        .domains
                        .iter()
                        .any(|d| t.template.contains(d) || t.source_domains.contains(d));
                    let cat_match = source
                        .categories
                        .iter()
                        .any(|sc| category_matches(sc, &t.category));

                    if domain_match && cat_match {
                        matching_templates += 1;
                    }
                }

                if matching_templates == 0 {
                    uncoverage_reasons.push(format!(
                        "No dedicated templates targeting domains of source '{}' on provider '{}'",
                        source.id,
                        provider.id()
                    ));
                }

                provider_cells.insert(
                    provider.id().to_string(),
                    CoverageCell {
                        enabled_templates: matching_templates,
                        disabled_templates: 0,
                        combination_templates: 0,
                        operator_status: "profile_validated".to_string(),
                        uncoverage_reasons,
                    },
                );
            }
            source_by_provider.insert(source.id.clone(), provider_cells);
        }

        // 5. Operator status summary
        let mut operator_status_summary: BTreeMap<String, usize> = BTreeMap::new();
        for t in registry.templates() {
            if t.enabled {
                let status = t.effective_validation_status().as_str().to_string();
                *operator_status_summary.entry(status).or_insert(0) += 1;
            }
        }

        CoverageMatrix {
            filter,
            total_templates: registry.templates().len(),
            total_source_profiles: sources.profiles().len(),
            category_by_provider,
            type_by_category,
            combination_by_category,
            source_by_provider,
            operator_status_summary,
        }
    }
}
