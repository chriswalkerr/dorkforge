use crate::coverage::matrix::CoverageMatrix;
use crate::error::Result;

pub fn format_coverage_json(matrix: &CoverageMatrix, pretty: bool) -> Result<String> {
    if pretty {
        serde_json::to_string_pretty(matrix).map_err(|e| {
            crate::error::DorkForgeError::SerializationError(format!(
                "Failed to serialize coverage matrix to JSON: {e}"
            ))
        })
    } else {
        serde_json::to_string(matrix).map_err(|e| {
            crate::error::DorkForgeError::SerializationError(format!(
                "Failed to serialize coverage matrix to JSON: {e}"
            ))
        })
    }
}

pub fn format_coverage_txt(matrix: &CoverageMatrix) -> String {
    let mut out = String::new();

    out.push_str(
        "================================================================================\n",
    );
    out.push_str(
        "                        DORKFORGE SOURCE COVERAGE MATRIX                        \n",
    );
    out.push_str(
        "================================================================================\n\n",
    );

    out.push_str(&format!(
        "Total Templates: {} | Source Profiles: {}\n",
        matrix.total_templates, matrix.total_source_profiles
    ));

    if matrix.filter.provider.is_some()
        || matrix.filter.category.is_some()
        || matrix.filter.r#type.is_some()
    {
        out.push_str(&format!(
            "Active Filters: provider={:?}, category={:?}, type={:?}\n",
            matrix.filter.provider, matrix.filter.category, matrix.filter.r#type
        ));
    }
    out.push('\n');

    // 1. Category x Provider
    out.push_str("--- 1. CATEGORY x PROVIDER MATRIX (Enabled Templates / Combinations) ---\n\n");
    let mut providers: Vec<String> = Vec::new();
    if let Some((_, first_p_map)) = matrix.category_by_provider.iter().next() {
        for p in first_p_map.keys() {
            providers.push(p.clone());
        }
    }

    out.push_str(&format!("{:<25}", "Category"));
    for p in &providers {
        out.push_str(&format!("{:<15}", p));
    }
    out.push('\n');
    out.push_str(&format!("{:-<25}", ""));
    for _ in &providers {
        out.push_str(&format!("{:-<15}", ""));
    }
    out.push('\n');

    for (cat, p_map) in &matrix.category_by_provider {
        out.push_str(&format!("{:<25}", cat));
        for p in &providers {
            if let Some(cell) = p_map.get(p) {
                let text = if cell.combination_templates > 0 {
                    format!(
                        "{}+({}c)",
                        cell.enabled_templates, cell.combination_templates
                    )
                } else {
                    format!("{}", cell.enabled_templates)
                };
                out.push_str(&format!("{:<15}", text));
            } else {
                out.push_str(&format!("{:<15}", "-"));
            }
        }
        out.push('\n');
    }
    out.push('\n');

    // 2. Identifier Type x Category
    out.push_str("--- 2. IDENTIFIER TYPE x CATEGORY MATRIX ---\n\n");
    let mut categories: Vec<String> = Vec::new();
    if let Some((_, first_c_map)) = matrix.type_by_category.iter().next() {
        for c in first_c_map.keys() {
            categories.push(c.clone());
        }
    }

    out.push_str(&format!("{:<18}", "Type"));
    for c in &categories {
        let short = if c.len() > 12 { &c[..12] } else { c };
        out.push_str(&format!("{:<14}", short));
    }
    out.push('\n');
    out.push_str(&format!("{:-<18}", ""));
    for _ in &categories {
        out.push_str(&format!("{:-<14}", ""));
    }
    out.push('\n');

    for (id_type, c_map) in &matrix.type_by_category {
        out.push_str(&format!("{:<18}", id_type));
        for c in &categories {
            let count = c_map.get(c).copied().unwrap_or(0);
            out.push_str(&format!("{:<14}", count));
        }
        out.push('\n');
    }
    out.push('\n');

    // 3. Combinations x Category
    if !matrix.combination_by_category.is_empty() {
        out.push_str("--- 3. COMBINATIONS x CATEGORY MATRIX ---\n\n");
        out.push_str(&format!("{:<28}", "Combination"));
        for c in &categories {
            let short = if c.len() > 12 { &c[..12] } else { c };
            out.push_str(&format!("{:<14}", short));
        }
        out.push('\n');
        out.push_str(&format!("{:-<28}", ""));
        for _ in &categories {
            out.push_str(&format!("{:-<14}", ""));
        }
        out.push('\n');

        for (combo, c_map) in &matrix.combination_by_category {
            out.push_str(&format!("{:<28}", combo));
            for c in &categories {
                let count = c_map.get(c).copied().unwrap_or(0);
                out.push_str(&format!("{:<14}", count));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    // 4. Source Profiles Coverage
    out.push_str("--- 4. SOURCE PROFILES x PROVIDER COVERAGE ---\n\n");
    out.push_str(&format!("{:<28}", "Source Profile"));
    for p in &providers {
        out.push_str(&format!("{:<15}", p));
    }
    out.push('\n');
    out.push_str(&format!("{:-<28}", ""));
    for _ in &providers {
        out.push_str(&format!("{:-<15}", ""));
    }
    out.push('\n');

    for (src, p_map) in &matrix.source_by_provider {
        out.push_str(&format!("{:<28}", src));
        for p in &providers {
            if let Some(cell) = p_map.get(p) {
                out.push_str(&format!("{:<15}", cell.enabled_templates));
            } else {
                out.push_str(&format!("{:<15}", "-"));
            }
        }
        out.push('\n');
    }
    out.push('\n');

    // 5. Operator status summary
    out.push_str("--- 5. OPERATOR VALIDATION STATUS SUMMARY ---\n\n");
    for (status, count) in &matrix.operator_status_summary {
        out.push_str(&format!("  - {:<20}: {}\n", status, count));
    }
    out.push('\n');

    out
}
