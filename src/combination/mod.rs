pub mod model;
pub mod parser;
pub mod planner;
pub mod validator;

pub use model::{
    CombinationIdentifierInput, CombinationSpec, CombinationVariantBinding, CombinationsFile,
    ResolvedCombinationIdentifier,
};
pub use parser::{
    is_combinations_json, load_combinations_from_file, parse_cli_combine_arg,
    parse_combinations_json,
};
pub use planner::{
    plan_variant_combinations, substitute_combination_template, CombinationPlannerLimits,
};
pub use validator::validate_and_resolve_combination;
