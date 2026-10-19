pub mod dedup;
pub mod engine;

pub use crate::templates::ValidationStatus;
pub use dedup::{DedupPolicy, QueryDeduplicator};
pub use engine::{
    DorkGenerator, DorkQuery, GenerationParams, GenerationResult, GenerationStats, GeneratorOptions,
};
