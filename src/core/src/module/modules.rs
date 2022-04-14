use std::collections::HashMap;

use input::{EventHandler, Sender as EventSender};
use todo_file::TodoFile;
use view::{RenderContext, ViewData, ViewSender};

use super::{Module, ProcessResult, State};
use crate::events::{AppKeyBindings, MetaEvent};

pub(crate) struct Modules<'modules> {
	event_handler: EventHandler<AppKeyBindings, MetaEvent>,
	modules: HashMap<State, Box<dyn Module + 'modules>>,
}

impl<'modules> Modules<'modules> {
	pub(crate) fn new(event_handler: EventHandler<AppKeyBindings, MetaEvent>) -> Self {
		Self {
			event_handler,
			modules: HashMap::new(),
		}
	}

	pub(crate) fn register_module<T: Module + 'modules>(&mut self, state: State, module: T) {
		let _previous = self.modules.insert(state, Box::new(module));
	}

	#[allow(clippy::panic)]
	fn get_mut_module(&mut self, state: State) -> &mut Box<dyn Module + 'modules> {
		self.modules
			.get_mut(&state)
			.unwrap_or_else(|| panic!("Invalid module for provided state: {:?}. Please report.", state))
	}

	#[allow(clippy::borrowed_box, clippy::panic)]
	fn get_module(&self, state: State) -> &Box<dyn Module + 'modules> {
		self.modules
			.get(&state)
			.unwrap_or_else(|| panic!("Invalid module for provided state: {:?}", state))
	}

	pub(crate) fn activate(&mut self, state: State, rebase_todo: &TodoFile, previous_state: State) -> ProcessResult {
		self.get_mut_module(state).activate(rebase_todo, previous_state)
	}

	pub(crate) fn deactivate(&mut self, state: State) {
		self.get_mut_module(state).deactivate();
	}

	pub(crate) fn build_view_data(
		&mut self,
		state: State,
		render_context: &RenderContext,
		rebase_todo: &TodoFile,
	) -> &ViewData {
		self.get_mut_module(state).build_view_data(render_context, rebase_todo)
	}

	pub(crate) fn handle_event(
		&mut self,
		state: State,
		event_sender: &mut EventSender<MetaEvent>,
		view_sender: &ViewSender,
		rebase_todo: &mut TodoFile,
	) -> ProcessResult {
		let module = self.get_module(state);
		let input_options = module.input_options();
		let event = self
			.event_handler
			.read_event(event_sender.read_event(), input_options, |event, key_bindings| {
				module.read_event(event, key_bindings)
			});
		self.get_mut_module(state).handle_event(event, view_sender, rebase_todo)
	}

	pub(crate) fn error(&mut self, state: State, error: &anyhow::Error) {
		self.get_mut_module(state).handle_error(error);
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use anyhow::{anyhow, Error};
	use input::StandardEvent;
	use parking_lot::Mutex;

	use super::*;
	use crate::{events::Event, testutil::module_test};

	#[derive(Debug, Clone)]
	struct TestModule {
		view_data: Arc<ViewData>,
		trace: Arc<Mutex<Vec<String>>>,
	}

	impl TestModule {
		fn new() -> Self {
			Self {
				view_data: Arc::new(ViewData::new(|_| {})),
				trace: Arc::new(Mutex::new(vec![])),
			}
		}

		fn trace(&self) -> String {
			self.trace.lock().join(",")
		}
	}

	impl Module for TestModule {
		fn activate(&mut self, _rebase_todo: &TodoFile, _previous_state: State) -> ProcessResult {
			self.trace.lock().push(String::from("Activate"));
			ProcessResult::new()
		}

		fn deactivate(&mut self) {
			self.trace.lock().push(String::from("Deactivate"));
		}

		fn build_view_data(&mut self, _render_context: &RenderContext, _rebase_todo: &TodoFile) -> &ViewData {
			self.trace.lock().push(String::from("Build View Data"));
			&self.view_data
		}

		fn handle_event(&mut self, _: Event, _: &ViewSender, _: &mut TodoFile) -> ProcessResult {
			self.trace.lock().push(String::from("Handle Events"));
			ProcessResult::new()
		}

		fn handle_error(&mut self, error: &Error) {
			self.trace.lock().push(error.to_string());
		}
	}

	#[test]
	fn module_lifecycle() {
		module_test(
			&["pick aaa comment"],
			&[Event::Standard(StandardEvent::Exit)],
			|mut context| {
				let mut modules = Modules::new(context.event_handler_context.event_handler);
				let test_module = TestModule::new();
				modules.register_module(State::List, test_module.clone());

				let _ = modules.activate(State::List, &context.rebase_todo_file, State::Insert);
				let _ = modules.handle_event(
					State::List,
					&mut context.event_handler_context.sender,
					&context.view_sender_context.sender,
					&mut context.rebase_todo_file,
				);
				let _ = modules.build_view_data(State::List, &RenderContext::new(100, 100), &context.rebase_todo_file);
				modules.deactivate(State::List);
				assert_eq!(test_module.trace(), "Activate,Handle Events,Build View Data,Deactivate");
			},
		);
	}

	#[test]
	fn error() {
		module_test(
			&["pick aaa comment"],
			&[Event::Standard(StandardEvent::Exit)],
			|context| {
				let mut modules = Modules::new(context.event_handler_context.event_handler);
				let test_module = TestModule::new();
				modules.register_module(State::Error, test_module.clone());
				modules.error(State::Error, &anyhow!("Test Error"));
				assert_eq!(test_module.trace(), "Test Error");
			},
		);
	}
}
