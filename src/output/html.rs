use crate::generator::GenerationResult;

use super::search_url;

/// Render a self-contained, offline HTML report. Query strings and all metadata
/// are escaped before insertion into markup; only the generated HTTPS URL is
/// used as an anchor destination.
pub fn format_html(result: &GenerationResult) -> String {
    let mut out = String::from(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>DorkForge results</title>\n<style>body{font-family:system-ui,-apple-system,sans-serif;max-width:1200px;margin:2rem auto;padding:0 1rem;color:#172033}table{border-collapse:collapse;width:100%;font-size:.9rem}th,td{padding:.65rem;border-bottom:1px solid #d8dee9;text-align:left;vertical-align:top}th{position:sticky;top:0;background:#f6f8fa}code{white-space:pre-wrap;word-break:break-word}a{color:#0969da}.meta{color:#57606a}</style>\n</head>\n<body>\n<h1>DorkForge generation results</h1>\n",
    );
    out.push_str(&format!(
        "<p class=\"meta\">{} provider-query records, {} unique query texts from {} input(s).</p>\n",
        result.stats.total_queries,
        result.stats.unique_query_texts,
        result.inputs.len()
    ));
    if !result.stats.warnings.is_empty() {
        out.push_str("<section aria-label=\"Warnings\"><h2>Warnings</h2><ul>\n");
        for warning in &result.stats.warnings {
            out.push_str(&format!("<li>{}</li>\n", escape_html(warning)));
        }
        out.push_str("</ul></section>\n");
    }
    out.push_str("<table>\n<thead><tr><th>Template</th><th>Validation</th><th>Provider</th><th>Category</th><th>Fingerprint</th><th>Query</th><th>Limitations</th><th>Search</th></tr></thead>\n<tbody>\n");

    for query in &result.queries {
        let url = search_url(query);
        out.push_str("<tr>");
        out.push_str(&format!("<td>{}</td>", escape_html(&query.template_id)));
        out.push_str(&format!(
            "<td>{}</td>",
            escape_html(query.validation_status.as_str())
        ));
        out.push_str(&format!("<td>{}</td>", escape_html(&query.provider)));
        out.push_str(&format!("<td>{}</td>", escape_html(&query.category)));
        out.push_str(&format!(
            "<td><code>{}</code></td>",
            escape_html(&query.fingerprint)
        ));
        out.push_str(&format!(
            "<td><code>{}</code></td>",
            escape_html(&query.query)
        ));
        out.push_str(&format!(
            "<td>{}</td>",
            escape_html(query.limitations.as_deref().unwrap_or(""))
        ));
        out.push_str(&format!(
            "<td><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">Open search</a></td>",
            escape_html(&url)
        ));
        out.push_str("</tr>\n");
    }

    out.push_str("</tbody>\n</table>\n</body>\n</html>\n");
    out
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::escape_html;

    #[test]
    fn escapes_markup_and_attributes() {
        assert_eq!(escape_html("<x>&\"'"), "&lt;x&gt;&amp;&quot;&#39;");
    }
}
