////////////////////////////////////////////////////////////////////////////////
// Tephra combinator library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////
#![forbid(non_ascii_idents)]
// #![deny(keyword_idents)]
// #![deny(macro_use_extern_crate)]
// #![deny(missing_abi)]
// #![deny(pointer_structural_match)]
// #![deny(unsafe_op_in_unsafe_fn)]
// #![warn(absolute_paths_not_starting_with_crate)]
// #![warn(anonymous_parameters)]
// #![warn(bad_style)]
// #![warn(bare_trait_objects)]
// #![warn(dead_code)]
// #![warn(elided_lifetimes_in_paths)]
// #![warn(improper_ctypes)]
// #![warn(missing_copy_implementations)]
// #![warn(missing_debug_implementations)]
// #![warn(rustdoc::missing_doc_code_examples)]
// #![warn(missing_docs)]
// #![warn(no_mangle_generic_items)]
// #![warn(non_shorthand_field_patterns)]
// #![warn(nonstandard_style)]
// #![warn(noop_method_call)]
// #![warn(overflowing_literals)]
// #![warn(path_statements)]
// #![warn(patterns_in_fns_without_body)]
// #![warn(private_in_public)]
// #![warn(rust_2018_idioms)]
// #![warn(trivial_casts)]
// #![warn(trivial_numeric_casts)]
// #![warn(unconditional_recursion)]
// #![warn(unreachable_pub)]
// #![warn(unused)]
// #![warn(unused_allocation)]
// #![warn(unused_comparisons)]
// #![warn(unused_extern_crates)]
// #![warn(unused_import_braces)]
// #![warn(unused_lifetimes)]
// #![warn(unused_parens)]
// #![warn(unused_qualifications)]
// #![warn(unused_results)]
// #![warn(variant_size_differences)]
// #![warn(while_true)]

// #![warn(must_not_suspend)] // Unstable
// #![warn(non_exhaustive_omitted_patterns)] // Unstable.
// #![warn(single_use_lifetimes)] // False positives.
// #![warn(unused_crate_dependencies)] // False positives.

// Clippy groups.
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

// Clippy restriction lints.
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::create_dir)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::decimal_literal_representation)]
#![warn(clippy::exit)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::float_cmp_const)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::lossy_float_literal)]
#![warn(clippy::map_err_ignore)]
#![warn(clippy::mem_forget)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::missing_enforced_import_renames)]
#![warn(clippy::mod_module_files)]
#![warn(clippy::multiple_inherent_impl)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::rest_pat_in_fully_bound_structs)]
#![warn(clippy::string_add)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(clippy::verbose_file_reads)]

// Non-improvement lints.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_bool)]
#![allow(clippy::single_match_else)]
#![allow(clippy::unseparated_literal_suffix)]

// Unreliable lints. May be enabled for spot checking.
#![allow(clippy::inline_always)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::shadow_unrelated)] // Does not work correctly.

// TODO: Remove these when error handling is more mature:
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]


// Internal modules.
mod alt;
mod cond;
mod control;
mod delimit;
mod join;
mod primitive;
mod repeat;
#[cfg(test)]
mod test;

// Exports.
pub use alt::*;
pub use cond::*;
pub use control::*;
pub use delimit::*;
pub use join::*;
pub use primitive::*;
pub use repeat::*;
pub use simple_predicates::Expr;
