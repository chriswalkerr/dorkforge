use crate::generator::GenerationResult;

use super::search_url;

/// Render a deterministic Graphviz DOT pivot graph. Query nodes carry a URL
/// attribute, so Graphviz SVG renderers preserve clickable search links.
pub fn format_graph_dot(result: &GenerationResult) -> String {
    let mut out = String::from(
        "digraph dorkforge {\n  graph [rankdir=LR, bgcolor=\"white\"];\n  node [fontname=\"Arial\", fontsize=10];\n  edge [color=\"#8c959f\"];\n",
    );

    for (index, input) in result.inputs.iter().enumerate() {
        out.push_str(&format!(
            "  input_{index} [label=\"{}\", shape=box, style=\"rounded,filled\", fillcolor=\"#ddf4ff\"];\n",
            escape_dot(input)
        ));
    }

    for (index, query) in result.queries.iter().enumerate() {
        let query_node = format!("query_{index}");
        let provider_node = format!("provider_{}", dot_id(&query.provider));
        let category_node = format!("category_{}", dot_id(&query.category));
        let input_index = result
            .inputs
            .iter()
            .position(|input| input == &query.identifier)
            .unwrap_or(0);
        let label = format!("{}\n{}", query.fingerprint, query.query);

        out.push_str(&format!(
            "  {query_node} [label=\"{}\", shape=note, URL=\"{}\", target=\"_blank\"];\n",
            escape_dot(&label),
            escape_dot(&search_url(query))
        ));
        out.push_str(&format!(
            "  {provider_node} [label=\"{}\", shape=ellipse, style=filled, fillcolor=\"#dafbe1\"];\n",
            escape_dot(&query.provider)
        ));
        out.push_str(&format!(
            "  {category_node} [label=\"{}\", shape=folder, style=filled, fillcolor=\"#fff8c5\"];\n",
            escape_dot(&query.category)
        ));
        out.push_str(&format!("  input_{input_index} -> {query_node};\n"));
        out.push_str(&format!("  {query_node} -> {provider_node};\n"));
        out.push_str(&format!("  {query_node} -> {category_node};\n"));
    }

    out.push_str("}\n");
    out
}

fn dot_id(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn escape_dot(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}
