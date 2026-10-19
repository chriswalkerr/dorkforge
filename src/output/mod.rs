pub mod graph;
pub mod html;
pub mod json;
pub mod txt;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::{DorkForgeError, Result};
use crate::generator::GenerationResult;
pub use graph::format_graph_dot;
pub use html::format_html;
pub use json::format_json;
pub use txt::{format_txt, format_urls, TxtMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Txt,
    Json,
    Html,
    #[default]
    Both,
    All,
}

impl FromStr for OutputFormat {
    type Err = DorkForgeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "txt" | "text" => Ok(OutputFormat::Txt),
            "json" => Ok(OutputFormat::Json),
            "both" => Ok(OutputFormat::Both),
            "html" | "htm" => Ok(OutputFormat::Html),
            "all" => Ok(OutputFormat::All),
            other => Err(DorkForgeError::ValidationError(format!(
                "Unknown output format: '{other}'. Choose from: txt, json, html, both, all"
            ))),
        }
    }
}

pub struct OutputWriter {
    format: OutputFormat,
    txt_mode: TxtMode,
    pretty_json: bool,
    overwrite: bool,
    urls: bool,
}

impl OutputWriter {
    pub fn new(
        format: OutputFormat,
        txt_mode: TxtMode,
        pretty_json: bool,
        overwrite: bool,
    ) -> Self {
        Self {
            format,
            txt_mode,
            pretty_json,
            overwrite,
            urls: false,
        }
    }

    /// Use clickable provider URLs for the TXT artifact.
    pub fn with_urls(mut self, urls: bool) -> Self {
        self.urls = urls;
        self
    }

    /// Writes generation results to a target destination.
    /// A directory receives `dorks.txt`, `dorks.json`, and/or `dorks.html`.
    /// A direct path is used as-is for a single format; multi-format output
    /// replaces its extension for each sibling artifact.
    pub fn write_to_destination(
        &self,
        result: &GenerationResult,
        target_path: Option<&Path>,
    ) -> Result<Vec<PathBuf>> {
        let Some(dest) = target_path else {
            return Ok(Vec::new());
        };

        let is_directory = dest.is_dir() || dest.extension().is_none();
        if is_directory {
            fs::create_dir_all(dest).map_err(|source| DorkForgeError::IoError {
                path: dest.to_path_buf(),
                source,
            })?;
        } else if let Some(parent) = dest.parent().filter(|path| !path.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|source| DorkForgeError::IoError {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let destination = |extension: &str| {
            if is_directory {
                dest.join(format!("dorks.{extension}"))
            } else if matches!(
                self.format,
                OutputFormat::Txt | OutputFormat::Json | OutputFormat::Html
            ) {
                dest.to_path_buf()
            } else {
                dest.with_extension(extension)
            }
        };
        let mut planned: Vec<(PathBuf, String)> = Vec::new();
        if matches!(
            self.format,
            OutputFormat::Txt | OutputFormat::Both | OutputFormat::All
        ) {
            planned.push((destination("txt"), self.format_txt(result)));
        }
        if matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Both | OutputFormat::All
        ) {
            planned.push((destination("json"), format_json(result, self.pretty_json)?));
        }
        if matches!(self.format, OutputFormat::Html | OutputFormat::All) {
            planned.push((destination("html"), format_html(result)));
        }

        // Detect every conflict before writing anything. This prevents a partial
        // TXT/JSON/HTML export if one sibling file already exists.
        if !self.overwrite {
            if let Some((path, _)) = planned.iter().find(|(path, _)| path.exists()) {
                return Err(DorkForgeError::OutputConflict { path: path.clone() });
            }
        }

        let mut written_files = Vec::with_capacity(planned.len());
        for (path, content) in planned {
            self.write_file(&path, &content)?;
            written_files.push(path);
        }
        Ok(written_files)
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        if path.exists() && !self.overwrite {
            return Err(DorkForgeError::OutputConflict {
                path: path.to_path_buf(),
            });
        }

        fs::write(path, content).map_err(|e| DorkForgeError::IoError {
            path: path.to_path_buf(),
            source: e,
        })
    }

    fn format_txt(&self, result: &GenerationResult) -> String {
        if self.urls {
            format_urls(result)
        } else {
            format_txt(result, self.txt_mode)
        }
    }
}

/// Build the provider-specific HTTPS search URL for a query.
pub fn search_url(query: &crate::generator::DorkQuery) -> String {
    use crate::providers::Provider;
    use std::str::FromStr;

    let base = Provider::from_str(&query.provider)
        .map(|provider| provider.search_base_url())
        .unwrap_or("https://www.google.com/search?q=");
    format!("{}{}", base, url_encode(&query.query))
}

pub(crate) fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
