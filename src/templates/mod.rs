pub mod registry;
pub mod validation;

pub use crate::quality::ValidationStatus;
pub use registry::{
    category_matches, normalize_category, TemplateConfigFile, TemplateDefinition, TemplateRegistry,
    TemplateValidation,
};
