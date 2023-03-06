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

//! Git Interactive Rebase Tool - Configuration Module
//!
//! # Description
//! This module is used to handle the loading of configuration from the Git config system.
//!
//! ```
//! use config::Config;
//! use git::Repository;
//! let config = Config::try_from(&Repository::open_from_env().unwrap());
//! ```
//!
//! ## Test Utilities
//! To facilitate testing the usages of this crate, a set of testing utilities are provided. Since
//! these utilities are not tested, and often are optimized for developer experience than
//! performance should only be used in test code.
mod color;
mod diff_ignore_whitespace_setting;
mod diff_show_whitespace_setting;
pub mod errors;
mod git_config;
mod key_bindings;
mod theme;
mod utils;

#[cfg(test)]
mod testutils;

use git::Repository;

use self::utils::{get_bool, get_diff_ignore_whitespace, get_diff_show_whitespace, get_string, get_unsigned_integer};
pub use self::{
	color::Color,
	diff_ignore_whitespace_setting::DiffIgnoreWhitespaceSetting,
	diff_show_whitespace_setting::DiffShowWhitespaceSetting,
	git_config::GitConfig,
	key_bindings::KeyBindings,
	theme::Theme,
};
use crate::errors::{ConfigError, ConfigErrorCause};

const DEFAULT_SPACE_SYMBOL: &str = "\u{b7}"; // ·
const DEFAULT_TAB_SYMBOL: &str = "\u{2192}"; // →

/// Represents the configuration options.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Config {
	/// If to select the next line in the list after performing an action.
	pub auto_select_next: bool,
	/// How to handle whitespace when calculating diffs.
	pub diff_ignore_whitespace: DiffIgnoreWhitespaceSetting,
	/// If to ignore blank lines when calculating diffs.
	pub diff_ignore_blank_lines: bool,
	/// How to show whitespace in diffs.
	pub diff_show_whitespace: DiffShowWhitespaceSetting,
	/// The symbol used to replace space characters.
	pub diff_space_symbol: String,
	/// The symbol used to replace tab characters.
	pub diff_tab_symbol: String,
	/// The display width of the tab character.
	pub diff_tab_width: u32,
	/// The maximum number of undo steps.
	pub undo_limit: u32,
	/// Configuration options loaded directly from Git.
	pub git: GitConfig,
	/// Key binding configuration.
	pub key_bindings: KeyBindings,
	/// Theme configuration.
	pub theme: Theme,
}

impl Config {
	/// Create a new configuration with default values.
	#[inline]
	#[must_use]
	#[allow(clippy::missing_panics_doc)]
	pub fn new() -> Self {
		Self::new_with_config(None).unwrap() // should never error with None config
	}

	fn new_with_config(git_config: Option<&git::Config>) -> Result<Self, ConfigError> {
		Ok(Self {
			auto_select_next: get_bool(git_config, "interactive-rebase-tool.autoSelectNext", false)?,
			diff_ignore_whitespace: get_diff_ignore_whitespace(
				git_config,
				"interactive-rebase-tool.diffIgnoreWhitespace",
			)?,
			diff_ignore_blank_lines: get_bool(git_config, "interactive-rebase-tool.diffIgnoreBlankLines", false)?,
			diff_show_whitespace: get_diff_show_whitespace(git_config, "interactive-rebase-tool.diffShowWhitespace")?,
			diff_space_symbol: get_string(
				git_config,
				"interactive-rebase-tool.diffSpaceSymbol",
				DEFAULT_SPACE_SYMBOL,
			)?,
			diff_tab_symbol: get_string(git_config, "interactive-rebase-tool.diffTabSymbol", DEFAULT_TAB_SYMBOL)?,
			diff_tab_width: get_unsigned_integer(git_config, "interactive-rebase-tool.diffTabWidth", 4)?,
			undo_limit: get_unsigned_integer(git_config, "interactive-rebase-tool.undoLimit", 5000)?,
			git: GitConfig::new_with_config(git_config)?,
			key_bindings: KeyBindings::new_with_config(git_config)?,
			theme: Theme::new_with_config(git_config)?,
		})
	}
}

impl TryFrom<&Repository> for Config {
	type Error = ConfigError;

	/// Creates a new Config instance loading the Git Config using [`git::Repository`].
	///
	/// # Errors
	///
	/// Will return an `Err` if there is a problem loading the configuration.
	#[inline]
	fn try_from(repo: &Repository) -> Result<Self, Self::Error> {
		let config = repo
			.load_config()
			.map_err(|e| ConfigError::new_read_error("", ConfigErrorCause::GitError(e)))?;
		Self::new_with_config(Some(&config))
	}
}

impl TryFrom<&git::Config> for Config {
	type Error = ConfigError;

	#[inline]
	fn try_from(config: &git::Config) -> Result<Self, Self::Error> {
		Self::new_with_config(Some(config))
	}
}

#[cfg(test)]
mod tests {
	use std::fmt::Debug;

	use ::testutils::assert_err_eq;
	use claim::assert_ok;
	use git::testutil::with_temp_bare_repository;
	use rstest::rstest;

	use super::*;
	use crate::testutils::{invalid_utf, with_git_config};

	#[test]
	fn new() {
		let _config = Config::new();
	}

	#[test]
	fn try_from_repository() {
		with_temp_bare_repository(|repository| {
			assert_ok!(Config::try_from(&repository));
		});
	}

	#[test]
	fn try_from_git_config() {
		with_git_config(&[], |git_config| {
			assert_ok!(Config::try_from(&git_config));
		});
	}

	#[test]
	fn try_from_git_config_error() {
		with_git_config(
			&["[interactive-rebase-tool]", "autoSelectNext = invalid"],
			|git_config| {
				let _ = Config::try_from(&git_config).unwrap_err();
			},
		);
	}

	#[rstest]
	#[case::auto_select_next_default("autoSelectNext", "", false, |config: Config| config.auto_select_next)]
	#[case::auto_select_next_false("autoSelectNext", "false", false, |config: Config| config.auto_select_next)]
	#[case::auto_select_next_true("autoSelectNext", "true", true, |config: Config| config.auto_select_next)]
	#[case::diff_ignore_whitespace_default(
		"diffIgnoreWhitespace",
		"",
		DiffIgnoreWhitespaceSetting::None,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_true(
		"diffIgnoreWhitespace",
		"true",
		DiffIgnoreWhitespaceSetting::All,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_on(
		"diffIgnoreWhitespace",
		"on",
		DiffIgnoreWhitespaceSetting::All,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_all(
		"diffIgnoreWhitespace",
		"all",
		DiffIgnoreWhitespaceSetting::All,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_change(
		"diffIgnoreWhitespace",
		"change",
		DiffIgnoreWhitespaceSetting::Change,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_false(
		"diffIgnoreWhitespace",
		"false",
		DiffIgnoreWhitespaceSetting::None,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_off(
		"diffIgnoreWhitespace",
		"off",
		DiffIgnoreWhitespaceSetting::None,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_none(
		"diffIgnoreWhitespace",
		"none",
		DiffIgnoreWhitespaceSetting::None,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_whitespace_mixed_case(
		"diffIgnoreWhitespace",
		"ChAnGe",
		DiffIgnoreWhitespaceSetting::Change,
		|config: Config| config.diff_ignore_whitespace)
	]
	#[case::diff_ignore_blank_lines_default(
		"diffIgnoreBlankLines",
		"",
		false,
		|config: Config| config.diff_ignore_blank_lines
	)]
	#[case::diff_ignore_blank_lines_false(
		"diffIgnoreBlankLines",
		"false",
		false,
		|config: Config| config.diff_ignore_blank_lines
	)]
	#[case::diff_ignore_blank_lines_true(
		"diffIgnoreBlankLines",
		"true",
		true,
		|config: Config| config.diff_ignore_blank_lines
	)]
	#[case::diff_show_whitespace_default(
		"diffShowWhitespace",
		"",
		DiffShowWhitespaceSetting::Both,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_true(
		"diffShowWhitespace",
		"true",
		DiffShowWhitespaceSetting::Both,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_on(
		"diffShowWhitespace",
		"on",
		DiffShowWhitespaceSetting::Both,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_both(
		"diffShowWhitespace",
		"both",
		DiffShowWhitespaceSetting::Both,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_trailing(
		"diffShowWhitespace",
		"trailing",
		DiffShowWhitespaceSetting::Trailing,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_leading(
		"diffShowWhitespace",
		"leading",
		DiffShowWhitespaceSetting::Leading,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_false(
		"diffShowWhitespace",
		"false",
		DiffShowWhitespaceSetting::None,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_off(
		"diffShowWhitespace",
		"off",
		DiffShowWhitespaceSetting::None,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_none(
		"diffShowWhitespace",
		"none",
		DiffShowWhitespaceSetting::None,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_show_whitespace_mixed_case(
		"diffShowWhitespace",
		"tRaIlInG",
		DiffShowWhitespaceSetting::Trailing,
		|config: Config| config.diff_show_whitespace)
	]
	#[case::diff_tab_width_default("diffTabWidth", "", 4, |config: Config| config.diff_tab_width)]
	#[case::diff_tab_width("diffTabWidth", "42", 42, |config: Config| config.diff_tab_width)]
	#[case::diff_tab_symbol_default("diffTabSymbol", "", String::from("→"), |config: Config| config.diff_tab_symbol)]
	#[case::diff_tab_symbol("diffTabSymbol", "|", String::from("|"), |config: Config| config.diff_tab_symbol)]
	#[case::diff_tab_symbol("diffTabSymbol", "|", String::from("|"), |config: Config| config.diff_tab_symbol)]
	#[case::diff_space_symbol_default(
		"diffSpaceSymbol",
		"",
		String::from("·"),
		|config: Config| config.diff_space_symbol)
	]
	#[case::diff_space_symbol("diffSpaceSymbol", "-", String::from("-"), |config: Config| config.diff_space_symbol)]
	#[case::undo_limit_default("undoLimit", "", 5000, |config: Config| config.undo_limit)]
	#[case::undo_limit_default("undoLimit", "42", 42, |config: Config| config.undo_limit)]
	pub(crate) fn theme_color<F, T>(
		#[case] config_name: &str,
		#[case] config_value: &str,
		#[case] expected: T,
		#[case] access: F,
	) where
		F: Fn(Config) -> T + 'static,
		T: Debug + PartialEq,
	{
		let value = format!("{config_name} = \"{config_value}\"");
		let lines = if config_value.is_empty() {
			vec![]
		}
		else {
			vec!["[interactive-rebase-tool]", value.as_str()]
		};
		with_git_config(&lines, |config| {
			let config = Config::new_with_config(Some(&config)).unwrap();
			assert_eq!(access(config), expected);
		});
	}

	#[rstest]
	#[case::auto_select_next("autoSelectNext", "invalid", ConfigErrorCause::InvalidBoolean)]
	#[case::diff_ignore_whitespace("diffIgnoreWhitespace", "invalid", ConfigErrorCause::InvalidDiffIgnoreWhitespace)]
	#[case::diff_ignore_blank_lines("diffIgnoreBlankLines", "invalid", ConfigErrorCause::InvalidBoolean)]
	#[case::diff_show_whitespace("diffShowWhitespace", "invalid", ConfigErrorCause::InvalidShowWhitespace)]
	#[case::diff_tab_width_non_integer("diffTabWidth", "invalid", ConfigErrorCause::InvalidUnsignedInteger)]
	#[case::diff_tab_width_non_poitive_integer("diffTabWidth", "-100", ConfigErrorCause::InvalidUnsignedInteger)]
	#[case::undo_limit_non_integer("undoLimit", "invalid", ConfigErrorCause::InvalidUnsignedInteger)]
	#[case::undo_limit_non_positive_integer("undoLimit", "-100", ConfigErrorCause::InvalidUnsignedInteger)]
	fn value_parsing_invalid(#[case] config_name: &str, #[case] config_value: &str, #[case] cause: ConfigErrorCause) {
		with_git_config(
			&[
				"[interactive-rebase-tool]",
				format!("{config_name} = {config_value}").as_str(),
			],
			|git_config| {
				assert_err_eq!(
					Config::new_with_config(Some(&git_config)),
					ConfigError::new(
						format!("interactive-rebase-tool.{config_name}").as_str(),
						config_value,
						cause
					)
				);
			},
		);
	}

	#[rstest]
	#[case::diff_tab_symbol("diffIgnoreWhitespace")]
	#[case::diff_tab_symbol("diffShowWhitespace")]
	#[case::diff_tab_symbol("diffTabSymbol")]
	#[case::diff_space_symbol("diffSpaceSymbol")]
	fn value_parsing_invalid_utf(#[case] config_name: &str) {
		with_git_config(
			&[
				"[interactive-rebase-tool]",
				format!("{config_name} = {}", invalid_utf()).as_str(),
			],
			|git_config| {
				assert_err_eq!(
					Config::new_with_config(Some(&git_config)),
					ConfigError::new_read_error(
						format!("interactive-rebase-tool.{config_name}").as_str(),
						ConfigErrorCause::InvalidUtf
					)
				);
			},
		);
	}
}
