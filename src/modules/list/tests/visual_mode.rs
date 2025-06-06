use super::*;
use crate::{
	action_line,
	assert_rendered_output,
	assert_results,
	input::KeyCode,
	process::Artifact,
	render_line,
	test_helpers::assertions::assert_rendered_output::AssertRenderOptions,
};

fn render_options() -> AssertRenderOptions {
	AssertRenderOptions::BODY_ONLY | AssertRenderOptions::INCLUDE_STYLE
}

#[test]
fn start() {
	testers::module(
		&["pick aaa c1", "pick aaa c2", "pick aaa c3"],
		&[Event::from(StandardEvent::ToggleVisualMode)],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Pick "aaa", "c1")),
				action_line!(Pick "aaa", "c2"),
				action_line!(Pick "aaa", "c3")
			);
		},
	);
}

#[test]
fn start_cursor_down_one() {
	testers::module(
		&["pick aaa c1", "pick aaa c2", "pick aaa c3"],
		&[
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::MoveCursorDown),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c1")),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Pick "aaa", "c2")),
				action_line!(Pick "aaa", "c3")
			);
		},
	);
}

#[test]
fn start_cursor_page_down() {
	testers::module(
		&[
			"pick aaa c1",
			"pick aaa c2",
			"pick aaa c3",
			"pick aaa c4",
			"pick aaa c5",
		],
		&[
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::MoveCursorPageDown),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			module.height = 4;
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c1")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c2")),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Pick "aaa", "c3")),
				action_line!(Pick "aaa", "c4"),
				action_line!(Pick "aaa", "c5")
			);
		},
	);
}

#[test]
fn start_cursor_from_bottom_move_up() {
	testers::module(
		&[
			"pick aaa c1",
			"pick aaa c2",
			"pick aaa c3",
			"pick aaa c4",
			"pick aaa c5",
		],
		&[
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::MoveCursorUp),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				action_line!(Pick "aaa", "c1"),
				action_line!(Pick "aaa", "c2"),
				action_line!(Pick "aaa", "c3"),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Pick "aaa", "c4")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c5"))
			);
		},
	);
}

#[test]
fn start_cursor_from_bottom_to_top() {
	testers::module(
		&[
			"pick aaa c1",
			"pick aaa c2",
			"pick aaa c3",
			"pick aaa c4",
			"pick aaa c5",
		],
		&[
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::MoveCursorUp),
			Event::from(StandardEvent::MoveCursorUp),
			Event::from(StandardEvent::MoveCursorUp),
			Event::from(StandardEvent::MoveCursorUp),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Pick "aaa", "c1")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c2")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c3")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c4")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Pick "aaa", "c5"))
			);
		},
	);
}

#[test]
fn action_change_top_bottom() {
	testers::module(
		&["pick aaa c1", "pick aaa c2", "pick aaa c3"],
		&[
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::ActionReword),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Reword "aaa", "c1")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Reword "aaa", "c2")),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Reword "aaa", "c3"))
			);
		},
	);
}

#[test]
fn action_change_bottom_top() {
	testers::module(
		&["pick aaa c1", "pick aaa c2", "pick aaa c3"],
		&[
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::MoveCursorDown),
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::MoveCursorUp),
			Event::from(StandardEvent::MoveCursorUp),
			Event::from(StandardEvent::ActionReword),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Options render_options(),
				test_context.build_view_data(&mut module),
				render_line!(All render_line!(Not Contains "Dimmed"), action_line!(Selected Reword "aaa", "c1")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Reword "aaa", "c2")),
				render_line!(All render_line!(Contains "Dimmed"), action_line!(Selected Reword "aaa", "c3"))
			);
		},
	);
}

#[test]
fn toggle_visual_mode() {
	testers::module(
		&["pick aaa c1"],
		&[
			Event::from(StandardEvent::ToggleVisualMode),
			Event::from(StandardEvent::ToggleVisualMode),
		],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			_ = test_context.handle_event(&mut module);
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from(StandardEvent::ToggleVisualMode))
			);
			assert_eq!(module.visual_index_start, None);
			assert_eq!(module.state, ListState::Normal);
		},
	);
}

#[test]
fn other_event() {
	testers::module(
		&["pick aaa c1"],
		&[Event::from(KeyCode::Null)],
		None,
		|mut test_context| {
			let mut module = List::new(&test_context.app_data());
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from(KeyCode::Null))
			);
		},
	);
}
