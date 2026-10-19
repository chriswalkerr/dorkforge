pub mod matrix;
pub mod report;
pub mod source_profile;

pub use matrix::{CoverageCell, CoverageFilter, CoverageMatrix};
pub use report::{format_coverage_json, format_coverage_txt};
pub use source_profile::{SourceProfile, SourceRegistry, SourcesConfigFile};
