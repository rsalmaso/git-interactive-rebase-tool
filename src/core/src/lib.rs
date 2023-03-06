// LINT-REPLACE-START
// This section is autogenerated, do not modify directly
// nightly sometimes removes/renames lints
#![cfg_attr(allow_unknown_lints, allow(unknown_lints))]
#![cfg_attr(allow_unknown_lints, allow(renamed_and_removed_lints))]
// enable all rustc's built-in lints
#![deny(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	rust_2021_compatibility,
	unused,
	warnings
)]
// rustc's additional allowed by default lints
#![deny(
	absolute_paths_not_starting_with_crate,
	deprecated_in_future,
	elided_lifetimes_in_paths,
	explicit_outlives_requirements,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	noop_method_call,
	pointer_structural_match,
	rust_2021_incompatible_closure_captures,
	rust_2021_incompatible_or_patterns,
	rust_2021_prefixes_incompatible_syntax,
	rust_2021_prelude_collisions,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unsafe_code,
	unsafe_op_in_unsafe_fn,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
	unused_lifetimes,
	unused_macro_rules,
	unused_qualifications,
	unused_results,
	unused_tuple_struct_fields,
	variant_size_differences
)]
// enable all of Clippy's lints
#![deny(clippy::all, clippy::cargo, clippy::pedantic, clippy::restriction)]
#![cfg_attr(include_nightly_lints, deny(clippy::nursery))]
#![allow(
	clippy::arithmetic_side_effects,
	clippy::blanket_clippy_restriction_lints,
	clippy::bool_to_int_with_if,
	clippy::default_numeric_fallback,
	clippy::else_if_without_else,
	clippy::expect_used,
	clippy::float_arithmetic,
	clippy::implicit_return,
	clippy::indexing_slicing,
	clippy::integer_arithmetic,
	clippy::map_err_ignore,
	clippy::missing_docs_in_private_items,
	clippy::missing_trait_methods,
	clippy::mod_module_files,
	clippy::module_name_repetitions,
	clippy::new_without_default,
	clippy::non_ascii_literal,
	clippy::option_if_let_else,
	clippy::pub_use,
	clippy::redundant_pub_crate,
	clippy::std_instead_of_alloc,
	clippy::std_instead_of_core,
	clippy::tabs_in_doc_comments,
	clippy::too_many_lines,
	clippy::unwrap_used
)]
#![deny(
	rustdoc::bare_urls,
	rustdoc::broken_intra_doc_links,
	rustdoc::invalid_codeblock_attributes,
	rustdoc::invalid_html_tags,
	rustdoc::missing_crate_level_docs,
	rustdoc::private_doc_tests,
	rustdoc::private_intra_doc_links
)]
// allow some things in tests
#![cfg_attr(
	test,
	allow(
		let_underscore_drop,
		clippy::cognitive_complexity,
		clippy::let_underscore_must_use,
		clippy::needless_pass_by_value,
		clippy::panic,
		clippy::shadow_reuse,
		clippy::shadow_unrelated,
		clippy::undocumented_unsafe_blocks,
		clippy::unimplemented,
		clippy::unreachable
	)
)]
// allowable upcoming nightly lints
#![cfg_attr(
	include_nightly_lints,
	allow(clippy::let_underscore_untyped, clippy::question_mark_used)
)]
// LINT-REPLACE-END
#![allow(
	clippy::as_conversions,
	clippy::cast_possible_truncation,
	clippy::redundant_closure_for_method_calls,
	clippy::wildcard_enum_match_arm,
	missing_docs,
	rustdoc::missing_crate_level_docs,
	unused
)]

mod application;
mod arguments;
mod components;
mod editor;
mod events;
mod exit;
mod help;
mod license;
mod module;
mod modules;
mod process;
mod search;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub mod testutil;
mod util;
mod version;

use std::ffi::OsString;

use crate::{
	arguments::{Args, Mode},
	exit::Exit,
};

#[inline]
#[must_use]
pub fn run(os_args: Vec<OsString>) -> Exit {
	match Args::try_from(os_args) {
		Err(err) => err,
		Ok(args) => {
			match *args.mode() {
				Mode::Help => help::run(),
				Mode::Version => version::run(),
				Mode::License => license::run(),
				Mode::Editor => editor::run(&args),
			}
		},
	}
}
