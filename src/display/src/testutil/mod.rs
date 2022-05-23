//! Utilities for writing tests that interact with the display.
mod mockable_tui;
mod mockcrossterm;
mod state;

pub use self::{mockable_tui::MockableTui, mockcrossterm::CrossTerm, state::State};
use crate::Display;

/// Assert the the content of the Display is an expected value.
///
/// # Panics
///
/// Will panic is the expected output does not match the rendered output.
#[inline]
pub fn assert_output(display: &Display<CrossTerm>, expected: &[&str]) {
	assert_eq!(display.tui.get_output().join(""), format!("{}\n", expected.join("\n")));
}
