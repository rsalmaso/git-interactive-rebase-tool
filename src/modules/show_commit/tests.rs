use anyhow::anyhow;
use git2::ErrorCode;
use rstest::rstest;

use super::*;
use crate::{
	assert_rendered_output,
	assert_results,
	diff::{Commit, Delta, DiffLine, FileMode, Origin, Status, User},
	input::StandardEvent,
	process::Artifact,
	render_line,
	test_helpers::{
		assertions::assert_rendered_output::AssertRenderOptions,
		builders::{CommitBuilder, CommitDiffBuilder, FileStatusBuilder},
		create_config,
		testers,
	},
};

fn render_options() -> AssertRenderOptions {
	AssertRenderOptions::INCLUDE_STYLE | AssertRenderOptions::BODY_ONLY
}

fn update_diff(module: &mut ShowCommit, builder: CommitDiffBuilder) {
	let diff = module.diff_state.diff();
	let mut diff_lock = diff.write();
	*diff_lock = builder.build();
	module.diff_state.set_load_status(LoadStatus::DiffComplete);
}

#[test]
fn load_commit_during_activate() {
	let hash = "abcde12345";
	let line = format!("pick {hash} comment1");
	testers::module(&[line.as_str()], &[], None, |test_context| {
		let mut module = ShowCommit::new(&test_context.app_data());
		assert_results!(
			test_context.activate(&mut module, State::List),
			Artifact::LoadDiff(String::from(hash))
		);
	});
}

#[test]
fn cached_commit_in_activate() {
	let hash = "abcde12345";
	let line = format!("pick {hash} comment1");
	testers::module(&[line.as_str()], &[], None, |test_context| {
		let mut module = ShowCommit::new(&test_context.app_data());
		let diff = test_context.app_data().diff_state().diff();
		// prefill cache
		{
			let mut diff_lock = diff.write();
			diff_lock.reset(Commit::new_with_hash(hash), None);
		}
		_ = test_context.activate(&mut module, State::List);
		// second call should not trigger a load_diff
		assert_results!(test_context.activate(&mut module, State::List));
	});
}

#[test]
fn no_selected_line_in_activate() {
	testers::module(&[], &[], None, |test_context| {
		let mut module = ShowCommit::new(&test_context.app_data());
		assert_results!(
			test_context.activate(&mut module, State::List),
			Artifact::Error(anyhow!("No valid commit to show"), Some(State::List))
		);
	});
}

#[test]
fn render_cancelled_diff_race() {
	testers::module(
		&[
			"pick 0123456789abcdef0123456789abcdef comment1",
			"pick abcdef0123456789abcdef0123456789 comment2",
		],
		&[],
		None,
		|test_context| {
			{
				let diff = test_context.app_data().diff_state().diff();
				let mut diff_lock = diff.write();
				diff_lock.reset(Commit::new_with_hash("abcdef0123456789abcdef0123456789"), None);
			}
			let mut module = ShowCommit::new(&test_context.app_data());

			assert_rendered_output!(
				Options AssertRenderOptions::BODY_ONLY,
				test_context.build_view_data(&mut module),
				"Loading Diff"
			);
		},
	);
}

#[test]
fn render_overview_minimal_commit() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let commit = CommitBuilder::new("0123456789abcdef0123456789abcdef").build();
			let commit_date = commit.committed_date().format("%c %z").to_string();
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(&mut module, CommitDiffBuilder::new(commit));

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				test_context.build_view_data(&mut module),
				"{TITLE}{HELP}",
				"{LEADING}",
				"{IndicatorColor}Commit: {Normal}0123456789abcdef0123456789abcdef",
				"{BODY}",
				format!("{{IndicatorColor}}Date: {{Normal}}{commit_date}"),
				"{Normal}",
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions"
			);
		},
	);
}

#[test]
fn render_overview_minimal_commit_compact() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|mut test_context| {
			test_context.render_context.update(30, 300);
			let commit = CommitBuilder::new("0123456789abcdef0123456789abcdef").build();
			let commit_date = commit.committed_date().format("%c %z").to_string();
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(&mut module, CommitDiffBuilder::new(commit));

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				test_context.build_view_data(&mut module),
				"{TITLE}{HELP}",
				"{LEADING}",
				"{Normal}01234567",
				"{BODY}",
				format!("{{IndicatorColor}}D: {{Normal}}{commit_date}"),
				"{Normal}",
				"{IndicatorColor}0{Normal} / {DiffAddColor}0{Normal} / {DiffRemoveColor}0"
			);
		},
	);
}

#[test]
fn render_overview_with_author() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.author(User::new(Some("John Doe"), Some("john.doe@example.com")))
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Author: {Normal}John Doe <john.doe@example.com>"
			);
		},
	);
}

#[test]
fn render_overview_with_author_compact() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|mut test_context| {
			test_context.render_context.update(30, 300);
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.author(User::new(Some("John Doe"), Some("john.doe@example.com")))
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}A: {Normal}John Doe <john.doe@example.com>"
			);
		},
	);
}

#[test]
fn render_overview_with_committer() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.committer(User::new(Some("John Doe"), Some("john.doe@example.com")))
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Committer: {Normal}John Doe <john.doe@example.com>"
			);
		},
	);
}

#[test]
fn render_overview_with_committer_compact() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|mut test_context| {
			test_context.render_context.update(30, 300);

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.committer(User::new(Some("John Doe"), Some("john.doe@example.com")))
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}C: {Normal}John Doe <john.doe@example.com>"
			);
		},
	);
}

#[test]
fn render_overview_with_commit_summary() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.summary("Commit title")
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{Normal}Commit title"
			);
		},
	);
}

#[test]
fn render_overview_with_commit_body() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.message("Commit body")
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{Normal}Commit body"
			);
		},
	);
}

#[test]
fn render_overview_with_commit_summary_and_body() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(
					CommitBuilder::new("0123456789abcdef0123456789abcdef")
						.summary("Commit title")
						.message("Commit body")
						.build(),
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1;2,
				test_context.build_view_data(&mut module),
				"{Normal}Commit title",
				"{Normal}",
				"{Normal}Commit body"
			);
		},
	);
}

#[test]
fn render_overview_with_file_stats() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.1a")
							.destination_path("file.1b")
							.status(Status::Renamed)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.2a")
							.destination_path("file.2a")
							.status(Status::Added)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.3a")
							.destination_path("file.3a")
							.status(Status::Deleted)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.4a")
							.destination_path("file.4b")
							.status(Status::Copied)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.5a")
							.destination_path("file.5a")
							.status(Status::Modified)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.6a")
							.destination_path("file.6a")
							.destination_mode(FileMode::Executable)
							.status(Status::Typechange)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.7a")
							.destination_path("file.7a")
							.status(Status::Other)
							.build(),
					],
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{DiffChangeColor} renamed: {DiffRemoveColor}file.1a{Normal} → {DiffAddColor}file.1b",
				"{DiffAddColor}   added: file.2a",
				"{DiffRemoveColor} deleted: file.3a",
				"{DiffAddColor}  copied: {Normal}file.4a → {DiffAddColor}file.4b",
				"{DiffChangeColor}modified: file.5a",
				"{DiffChangeColor} changed: file.6a",
				"{Normal} unknown: file.7a"
			);
		},
	);
}

#[test]
fn render_overview_with_file_stats_compact() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|mut test_context| {
			test_context.render_context.update(30, 300);

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.1a")
							.destination_path("file.1b")
							.status(Status::Renamed)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.2a")
							.destination_path("file.2a")
							.status(Status::Added)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.3a")
							.destination_path("file.3a")
							.status(Status::Deleted)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.4a")
							.destination_path("file.4b")
							.status(Status::Copied)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.5a")
							.source_path("file.5a")
							.destination_path("file.5a")
							.status(Status::Modified)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.6a")
							.destination_path("file.6a")
							.destination_mode(FileMode::Executable)
							.status(Status::Typechange)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.7a")
							.destination_path("file.7a")
							.status(Status::Other)
							.build(),
					],
				),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} / {DiffAddColor}0{Normal} / {DiffRemoveColor}0",
				"{DiffChangeColor}R {DiffRemoveColor}file.1a{Normal}→{DiffAddColor}file.1b",
				"{DiffAddColor}A file.2a",
				"{DiffRemoveColor}D file.3a",
				"{DiffAddColor}C {Normal}file.4a→{DiffAddColor}file.4b",
				"{DiffChangeColor}M file.5a",
				"{DiffChangeColor}T file.6a",
				"{Normal}X file.7a"
			);
		},
	);
}
#[test]
fn render_overview_single_file_changed() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build())
					.number_files_changed(1),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}1{Normal} file with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions"
			);
		},
	);
}

#[test]
fn render_overview_more_than_one_file_changed() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build())
					.number_files_changed(2),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1,
				test_context.build_view_data(&mut module),
				"{Normal}",
				"{IndicatorColor}2{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions"
			);
		},
	);
}

#[test]
fn render_overview_single_insertion() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build())
					.number_insertions(1),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1,
				test_context.build_view_data(&mut module),
				"{Normal}",
				"{IndicatorColor}0{Normal} files with {DiffAddColor}1{Normal} insertion and \
				 {DiffRemoveColor}0{Normal} deletions"
			);
		},
	);
}

#[test]
fn render_overview_more_than_one_insertion() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build())
					.number_insertions(2),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1,
				test_context.build_view_data(&mut module),
				"{Normal}",
				"{IndicatorColor}0{Normal} files with {DiffAddColor}2{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions"
			);
		},
	);
}

#[test]
fn render_overview_single_deletion() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build())
					.number_deletions(1),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1,
				test_context.build_view_data(&mut module),
				"{Normal}",
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}1{Normal} deletion"
			);
		},
	);
}

#[test]
fn render_overview_more_than_one_deletion() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build())
					.number_deletions(2),
			);

			assert_rendered_output!(
				Options render_options(),
				Skip 1,
				test_context.build_view_data(&mut module),
				"{Normal}",
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}2{Normal} deletions"
			);
		},
	);
}

#[test]
fn render_diff_minimal_commit() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				test_context.build_view_data(&mut module),
				"{TITLE}{HELP}",
				"{LEADING}",
				"{IndicatorColor}Commit: {Normal}0123456789abcdef0123456789abcdef",
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{BODY}",
				"{Normal}{Pad(―)}"
			);
		},
	);
}

#[test]
fn render_diff_minimal_commit_compact() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|mut test_context| {
			test_context.render_context.update(30, 300);

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				test_context.build_view_data(&mut module),
				"{TITLE}{HELP}",
				"{LEADING}",
				"{Normal}01234567",
				"{IndicatorColor}0{Normal} / {DiffAddColor}0{Normal} / {DiffRemoveColor}0",
				"{BODY}",
				"{Normal}{Pad(―)}"
			);
		},
	);
}

#[test]
fn render_diff_basic_file_stats() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.1a")
							.destination_path("file.1b")
							.status(Status::Renamed)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.2a")
							.destination_path("file.2a")
							.status(Status::Added)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.3a")
							.destination_path("file.3a")
							.status(Status::Deleted)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.4a")
							.destination_path("file.4b")
							.status(Status::Copied)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.5a")
							.destination_path("file.5a")
							.status(Status::Modified)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.6a")
							.destination_path("file.6a")
							.destination_mode(FileMode::Executable)
							.status(Status::Typechange)
							.build(),
						FileStatusBuilder::new()
							.source_path("file.7a")
							.destination_path("file.7a")
							.status(Status::Other)
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 3,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{BODY}",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor} renamed: {DiffRemoveColor}file.1a{Normal} → {DiffAddColor}file.1b",
				"{Normal}{Pad(―)}",
				"{DiffAddColor}   added: file.2a",
				"{Normal}{Pad(―)}",
				"{DiffRemoveColor} deleted: file.3a",
				"{Normal}{Pad(―)}",
				"{DiffAddColor}  copied: {Normal}file.4a → {DiffAddColor}file.4b",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor}modified: file.5a",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor} changed: file.6a",
				"{Normal}{Pad(―)}",
				"{Normal} unknown: file.7a"
			);
		},
	);
}

#[test]
fn render_diff_end_new_line_missing() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut delta = Delta::new("@@ -14,2 +13,3 @@ context", 14, 14, 0, 1);
			delta.add_line(DiffLine::new(Origin::Addition, "new line", None, Some(14), false));
			delta.add_line(DiffLine::new(Origin::Addition, "", None, Some(15), true));

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(delta)
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 3,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{BODY}",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor}modified: file.txt",
				"",
				"{Normal,Dimmed}@@{DiffContextColor} -14,0 +14,1 {Normal,Dimmed}@@{DiffContextColor} context",
				"{Normal,Dimmed}{Pad(―)}",
				"{Normal}   14| {DiffAddColor}new line",
				"{Normal}       {DiffContextColor}\\ No newline at end of file"
			);
		},
	);
}

#[test]
fn render_diff_add_line() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut delta = Delta::new("@@ -14,2 +13,3 @@ context", 14, 14, 0, 1);
			delta.add_line(DiffLine::new(Origin::Addition, "new line", None, Some(14), false));

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(delta)
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 3,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{BODY}",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor}modified: file.txt",
				"",
				"{Normal,Dimmed}@@{DiffContextColor} -14,0 +14,1 {Normal,Dimmed}@@{DiffContextColor} context",
				"{Normal,Dimmed}{Pad(―)}",
				"{Normal}   14| {DiffAddColor}new line"
			);
		},
	);
}

#[test]
fn render_diff_delete_line() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut delta = Delta::new("@@ -14,2 +13,3 @@ context", 14, 14, 0, 1);
			delta.add_line(DiffLine::new(Origin::Deletion, "old line", Some(14), None, false));

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(delta)
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 3,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{BODY}",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor}modified: file.txt",
				"",
				"{Normal,Dimmed}@@{DiffContextColor} -14,0 +14,1 {Normal,Dimmed}@@{DiffContextColor} context",
				"{Normal,Dimmed}{Pad(―)}",
				"{Normal}14   | {DiffRemoveColor}old line"
			);
		},
	);
}

#[test]
fn render_diff_context_add_remove_lines() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut delta = Delta::new("@@ -14,2 +13,3 @@ context", 14, 14, 0, 1);
			delta.add_line(DiffLine::new(Origin::Context, "context 1", Some(13), Some(13), false));
			delta.add_line(DiffLine::new(Origin::Deletion, "old line", Some(14), None, false));
			delta.add_line(DiffLine::new(Origin::Addition, "new line", None, Some(14), false));
			delta.add_line(DiffLine::new(Origin::Context, "context 2", Some(15), Some(15), false));

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(delta)
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 3,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}0{Normal} files with {DiffAddColor}0{Normal} insertions and \
				 {DiffRemoveColor}0{Normal} deletions",
				"{BODY}",
				"{Normal}{Pad(―)}",
				"{DiffChangeColor}modified: file.txt",
				"",
				"{Normal,Dimmed}@@{DiffContextColor} -14,0 +14,1 {Normal,Dimmed}@@{DiffContextColor} context",
				"{Normal,Dimmed}{Pad(―)}",
				"{Normal}13 13| {DiffContextColor}context 1",
				"{Normal}14   | {DiffRemoveColor}old line",
				"{Normal}   14| {DiffAddColor}new line",
				"{Normal}15 15| {DiffContextColor}context 2"
			);
		},
	);
}

fn generate_diff_line_context(content: &str, line_num: u32) -> DiffLine {
	DiffLine::new(Origin::Context, content, Some(line_num), Some(line_num), false)
}

fn generate_white_space_delta() -> Delta {
	let mut delta = Delta::new("@@ -1,7 +1,7 @@ context", 1, 1, 7, 7);
	// leading tabs
	delta.add_line(generate_diff_line_context("\t\tsp tabs\t\tcontent", 1));
	// trailing tabs
	delta.add_line(generate_diff_line_context("sp tabs\t\tcontent\t\t", 2));
	// leading and trailing tabs
	delta.add_line(generate_diff_line_context("\t\tsp tabs\t\tcontent\t\t", 3));
	// leading spaces
	delta.add_line(generate_diff_line_context("    sp tabs\t\tcontent", 4));
	// trailing spaces
	delta.add_line(generate_diff_line_context("sp tabs\t\tcontent    ", 5));
	// leading and trailing spaces
	delta.add_line(generate_diff_line_context("    sp tabs\t\tcontent    ", 6));
	// mix of spaces and tabs
	delta.add_line(generate_diff_line_context(" \t\t sp tabs\t\tcontent\t  \t", 7));
	delta
}

#[test]
fn render_diff_show_both_whitespace() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::Both;
	config.diff_tab_symbol = String::from("#>");
	config.diff_space_symbol = String::from("%");
	config.diff_tab_width = 2;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(generate_white_space_delta())
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 10,
				test_context.build_view_data(&mut module),
				render_line!(EndsWith "#>#>{DiffContextColor}sp tabs    content"),
				render_line!(EndsWith "sp tabs    content{DiffWhitespaceColor}#>#>"),
				render_line!(EndsWith "#>#>{DiffContextColor}sp tabs    content{DiffWhitespaceColor}#>#>"),
				render_line!(EndsWith "%%%%{DiffContextColor}sp tabs    content"),
				render_line!(EndsWith "sp tabs    content{DiffWhitespaceColor}%%%%"),
				render_line!(EndsWith "%%%%{DiffContextColor}sp tabs    content{DiffWhitespaceColor}%%%%"),
				render_line!(EndsWith "%#>#>%{DiffContextColor}sp tabs    content{DiffWhitespaceColor}#>%%#>")
			);
		},
	);
}

#[test]
fn render_diff_show_leading_whitespace() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::Leading;
	config.diff_tab_symbol = String::from("#>");
	config.diff_space_symbol = String::from("%");
	config.diff_tab_width = 2;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(generate_white_space_delta())
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 10,
				test_context.build_view_data(&mut module),
				render_line!(EndsWith "#>#>{DiffContextColor}sp tabs    content"),
				render_line!(EndsWith "sp tabs    content{DiffWhitespaceColor}"),
				render_line!(EndsWith "#>#>{DiffContextColor}sp tabs    content{DiffWhitespaceColor}"),
				render_line!(EndsWith "%%%%{DiffContextColor}sp tabs    content"),
				render_line!(EndsWith "sp tabs    content{DiffWhitespaceColor}"),
				render_line!(EndsWith "%%%%{DiffContextColor}sp tabs    content{DiffWhitespaceColor}"),
				render_line!(EndsWith "%#>#>%{DiffContextColor}sp tabs    content{DiffWhitespaceColor}")
			);
		},
	);
}

#[test]
fn render_diff_show_no_whitespace() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::None;
	config.diff_tab_symbol = String::from("#>");
	config.diff_space_symbol = String::from("%");
	config.diff_tab_width = 2;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(generate_white_space_delta())
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 10,
				test_context.build_view_data(&mut module),
				render_line!(EndsWith "    sp tabs    content"),
				render_line!(EndsWith "sp tabs    content"),
				render_line!(EndsWith "    sp tabs    content"),
				render_line!(EndsWith "    sp tabs    content"),
				render_line!(EndsWith "sp tabs    content"),
				render_line!(EndsWith "    sp tabs    content"),
				render_line!(EndsWith "      sp tabs    content")
			);
		},
	);
}

#[test]
fn render_diff_show_whitespace_all_spaces() {
	let mut config = create_config();
	config.diff_show_whitespace = DiffShowWhitespaceSetting::Both;
	config.diff_tab_symbol = String::from("#>");
	config.diff_space_symbol = String::from("%");
	config.diff_tab_width = 2;
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		Some(config),
		|test_context| {
			let mut delta = Delta::new("@@ -1,7 +1,7 @@ context", 1, 1, 7, 7);
			delta.add_line(DiffLine::new(Origin::Addition, "    ", None, Some(1), false));

			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()).file_statuses(
					vec![
						FileStatusBuilder::new()
							.source_path("file.txt")
							.destination_path("file.txt")
							.status(Status::Modified)
							.push_delta(delta)
							.build(),
					],
				),
			);

			module.state = ShowCommitState::Diff;
			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 10,
				test_context.build_view_data(&mut module),
				"{Normal}  1| {DiffWhitespaceColor}%%%%"
			);
		},
	);
}

#[test]
fn render_loading_new() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			test_context.app_data().diff_state().set_load_status(LoadStatus::New);

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Loading Diff"
			);
		},
	);
}

#[test]
fn render_loading_quick_diff_progress() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			test_context
				.app_data()
				.diff_state()
				.set_load_status(LoadStatus::QuickDiff(5, 12));

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 8,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Loading Diff (-) [5/12]"
			);
		},
	);
}

#[test]
fn render_loading_quick_diff_complete() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			test_context
				.app_data()
				.diff_state()
				.set_load_status(LoadStatus::CompleteQuickDiff);

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 8,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Detecting renames and copies"
			);
		},
	);
}

#[test]
fn render_loading_diff_progress() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			test_context
				.app_data()
				.diff_state()
				.set_load_status(LoadStatus::Diff(7, 12));

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 8,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Detecting renames and copies (-) [7/12]"
			);
		},
	);
}

#[test]
fn render_loading_diff_error_not_found() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			test_context.app_data().diff_state().set_load_status(LoadStatus::Error {
				msg: String::from("message"),
				code: ErrorCode::NotFound,
			});

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Error loading diff. Press any key to return.",
				"{Normal}",
				"{Normal}Reason:",
				"{Normal}Commit not found"
			);
		},
	);
}

#[test]
fn render_loading_diff_error_other() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef comment1"],
		&[],
		None,
		|test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			update_diff(
				&mut module,
				CommitDiffBuilder::new(CommitBuilder::new("0123456789abcdef0123456789abcdef").build()),
			);

			test_context.app_data().diff_state().set_load_status(LoadStatus::Error {
				msg: String::from("message"),
				code: ErrorCode::GenericError,
			});

			assert_rendered_output!(
				Options AssertRenderOptions::INCLUDE_STYLE,
				Skip 2,
				test_context.build_view_data(&mut module),
				"{IndicatorColor}Error loading diff. Press any key to return.",
				"{Normal}",
				"{Normal}Reason:",
				"{Normal}message"
			);
		},
	);
}

#[test]
fn handle_event_toggle_diff_to_overview() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef c1"],
		&[Event::from(StandardEvent::ShowDiff)],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			module
				.diff_view_data
				.update_view_data(|updater| updater.push_line(ViewLine::from("foo")));
			module.state = ShowCommitState::Diff;
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from(StandardEvent::ShowDiff))
			);
			assert_eq!(module.state, ShowCommitState::Overview);
		},
	);
}

#[test]
fn handle_event_toggle_overview_to_diff() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef c1"],
		&[Event::from('d')],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			module
				.overview_view_data
				.update_view_data(|updater| updater.push_line(ViewLine::from("foo")));
			module.state = ShowCommitState::Overview;
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from(StandardEvent::ShowDiff))
			);
			assert_eq!(module.state, ShowCommitState::Diff);
		},
	);
}

#[test]
fn handle_event_resize() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef c1"],
		&[Event::Resize(100, 100)],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::Resize(100, 100))
			);
		},
	);
}

#[test]
fn render_help() {
	testers::module(
		&["pick aaa c1"],
		&[Event::from(StandardEvent::Help)],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert_rendered_output!(
				Body test_context.build_view_data(&mut module),
				" Up      |Scroll up",
				" Down    |Scroll down",
				" PageUp  |Scroll up half a page",
				" PageDown|Scroll down half a page",
				" Home    |Scroll to the top",
				" End     |Scroll to the bottom",
				" Right   |Scroll right",
				" Left    |Scroll left",
				" d       |Show full diff",
				" ?       |Show help"
			);
		},
	);
}

#[test]
fn handle_help_event_show() {
	testers::module(
		&["pick aaa c1"],
		&[Event::from(StandardEvent::Help)],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert!(module.help.is_active());
		},
	);
}
#[test]
fn handle_help_event_hide() {
	testers::module(
		&["pick aaa c1"],
		&[Event::from(StandardEvent::Help), Event::from('?')],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			_ = test_context.handle_all_events(&mut module);
			assert!(!module.help.is_active());
		},
	);
}

#[test]
fn handle_event_other_key_from_diff() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef c1"],
		&[Event::from('a')],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			module.state = ShowCommitState::Diff;
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from('a'))
			);
			assert_eq!(module.state, ShowCommitState::Overview);
		},
	);
}

#[test]
fn handle_event_other_key_from_overview() {
	testers::module(
		&["pick 0123456789abcdef0123456789abcdef c1"],
		&[Event::from('a')],
		None,
		|mut test_context| {
			let mut module = ShowCommit::new(&test_context.app_data());
			module.state = ShowCommitState::Overview;
			assert_results!(
				test_context.handle_event(&mut module),
				Artifact::Event(Event::from('a')),
				Artifact::CancelDiff,
				Artifact::ChangeState(State::List)
			);
		},
	);
}

#[rstest]
#[case::scroll_left(StandardEvent::ScrollLeft)]
#[case::scroll_right(StandardEvent::ScrollRight)]
#[case::scroll_down(StandardEvent::ScrollDown)]
#[case::scroll_up(StandardEvent::ScrollUp)]
#[case::scroll_jump_down(StandardEvent::ScrollJumpDown)]
#[case::scroll_jump_up(StandardEvent::ScrollJumpUp)]
fn scroll_events(#[case] event: StandardEvent) {
	testers::module(&[], &[Event::from(event)], None, |mut test_context| {
		let mut module = ShowCommit::new(&test_context.app_data());
		assert_results!(
			test_context.handle_event(&mut module),
			Artifact::Event(Event::from(event))
		);
	});
}
