use std::sync::LazyLock;

use captur::capture;

use crate::{
	application::AppData,
	display::DisplayColor,
	input::{Event, InputOptions},
	module::{Module, State},
	process::Results,
	select,
	util::handle_view_data_scroll,
	view::{self, LineSegment, RenderContext, ViewData, ViewLine},
};

pub(crate) static INPUT_OPTIONS: LazyLock<InputOptions> =
	LazyLock::new(|| InputOptions::RESIZE | InputOptions::MOVEMENT);

pub(crate) struct Error {
	return_state: State,
	view_data: ViewData,
	view_state: view::State,
}

impl Module for Error {
	fn activate(&mut self, previous_state: State) -> Results {
		self.return_state = previous_state;
		Results::new()
	}

	fn build_view_data(&mut self, _: &RenderContext) -> &ViewData {
		&self.view_data
	}

	fn input_options(&self) -> &InputOptions {
		&INPUT_OPTIONS
	}

	fn handle_event(&mut self, event: Event) -> Results {
		select!(
			default {
				let mut results = Results::new();
				if let Event::Key(_) = event {
					results.state(self.return_state);
				}
				results
			},
			handle_view_data_scroll(event, &self.view_state)
		)
	}

	fn handle_error(&mut self, error: &anyhow::Error) -> Results {
		self.view_data.update_view_data(|updater| {
			capture!(error);
			updater.clear();
			for cause in error.chain() {
				let error_text = format!("{cause:#}");
				for err in error_text.split('\n') {
					updater.push_line(ViewLine::from(err));
				}
			}
			updater.push_trailing_line(ViewLine::from(LineSegment::new_with_color(
				"Press any key to continue",
				DisplayColor::IndicatorColor,
			)));
		});
		Results::new()
	}
}

impl Error {
	pub(crate) fn new(app_data: &AppData) -> Self {
		Self {
			return_state: State::List,
			view_data: ViewData::new(|updater| {
				updater.set_show_title(true);
				updater.set_retain_scroll_position(false);
			}),
			view_state: app_data.view_state(),
		}
	}
}

#[cfg(test)]
mod tests {
	use anyhow::anyhow;

	use super::*;
	use crate::{assert_rendered_output, assert_results, process::Artifact, test_helpers::testers};

	#[test]
	fn simple_error() {
		testers::module(&[], &[], None, |test_context| {
			let mut module = Error::new(&test_context.app_data());
			_ = module.handle_error(&anyhow!("Test Error"));
			let view_data = test_context.build_view_data(&mut module);
			assert_rendered_output!(
				Style view_data,
				"{TITLE}",
				"{BODY}",
				"{Normal}Test Error",
				"{TRAILING}",
				"{IndicatorColor}Press any key to continue"
			);
		});
	}

	#[test]
	fn error_with_contest() {
		testers::module(&[], &[], None, |test_context| {
			let mut module = Error::new(&test_context.app_data());
			_ = module.handle_error(&anyhow!("Test Error").context("Context"));
			let view_data = test_context.build_view_data(&mut module);
			assert_rendered_output!(
				Body view_data,
				"Context",
				"Test Error"
			);
		});
	}

	#[test]
	fn error_with_newlines() {
		testers::module(&[], &[], None, |test_context| {
			let mut module = Error::new(&test_context.app_data());
			_ = module.handle_error(&anyhow!("Test\nError").context("With\nContext"));
			let view_data = test_context.build_view_data(&mut module);
			assert_rendered_output!(
				Body view_data,
				"With",
				"Context",
				"Test",
				"Error"
			);
		});
	}

	#[test]
	fn return_state() {
		testers::module(&[], &[Event::from('a')], None, |mut test_context| {
			let mut module = Error::new(&test_context.app_data());
			_ = test_context.activate(&mut module, State::ConfirmRebase);
			_ = module.handle_error(&anyhow!("Test Error"));
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from('a')),
				Artifact::ChangeState(State::ConfirmRebase)
			);
		});
	}

	#[test]
	fn resize() {
		testers::module(&[], &[Event::Resize(100, 100)], None, |mut test_context| {
			let mut module = Error::new(&test_context.app_data());
			_ = test_context.activate(&mut module, State::ConfirmRebase);
			_ = module.handle_error(&anyhow!("Test Error"));
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::Resize(100, 100))
			);
		});
	}
}
