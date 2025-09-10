pub(crate) mod cli;
pub(crate) mod errors;
pub(crate) mod helpers;

pub use crate::helpers::{
    acceptable_crate_name, capitalize_string, crate_name_from_path, extend_table,
    into_acceptable_crate_name, into_acceptable_error_type_name, into_acceptable_name,
    into_acceptable_package_name, package_name_from_string_or_path, path_to_entry_path, slug,
    strip_ends, struct_name_from_package_name, to_pascal_case, valid_crate_name,
    valid_manifest_path, valid_package_name, words, absolute_path
};
pub use cli::{ClapExecuter, Craft, Dependency};
pub use errors::{Error, Result, ExecutionResult};

pub(crate) mod templates;
pub use templates::{render, render_cli, render_info_string, tera, tera_info};
