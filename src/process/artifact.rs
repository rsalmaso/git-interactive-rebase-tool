use std::fmt::{Debug, Formatter};

use anyhow::Error;

use crate::{
	input::Event,
	module::{ExitStatus, State},
	search::Searchable,
};

pub(crate) enum Artifact {
	ChangeState(State),
	EnqueueResize,
	Error(Error, Option<State>),
	Event(Event),
	ExitStatus(ExitStatus),
	ExternalCommand((String, Vec<String>)),
	SearchCancel,
	SearchTerm(String),
	Searchable(Box<dyn Searchable>),
	LoadDiff(String),
	CancelDiff,
}

impl Debug for Artifact {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match *self {
			Self::ChangeState(state) => write!(f, "ChangeState({state:?})"),
			Self::EnqueueResize => write!(f, "EnqueueResize"),
			Self::Error(ref err, state) => write!(f, "Error({err}, {state:?})"),
			Self::Event(event) => write!(f, "Event({event:?})"),
			Self::ExitStatus(status) => write!(f, "ExitStatus({status:?})"),
			Self::ExternalCommand((ref command, ref args)) => write!(f, "ExternalCommand({command:?}, {args:?})"),
			Self::SearchCancel => write!(f, "SearchCancel"),
			Self::SearchTerm(ref term) => write!(f, "SearchTerm({term:?})"),
			Self::Searchable(_) => write!(f, "Searchable(dyn Searchable)"),
			Self::LoadDiff(ref hash) => write!(f, "LoadDiff({hash:?})"),
			Self::CancelDiff => write!(f, "CancelDiff"),
		}
	}
}

#[cfg(test)]
mod tests {
	use anyhow::anyhow;
	use rstest::rstest;

	use super::*;
	use crate::test_helpers::mocks;

	#[rstest]
	#[case::change_state(Artifact::ChangeState(State::List), "ChangeState(List)")]
	#[case::enqueue_resize(Artifact::EnqueueResize, "EnqueueResize")]
	#[case::error(Artifact::Error(anyhow!("Error"), Some(State::List)), "Error(Error, Some(List))")]
	#[case::event(Artifact::Event(Event::None), "Event(None)")]
	#[case::exit_status(Artifact::ExitStatus(ExitStatus::Abort), "ExitStatus(Abort)")]
	#[case::external_command(Artifact::ExternalCommand((String::from("foo"), vec![])), "ExternalCommand(\"foo\", [])")]
	#[case::search_cancel(Artifact::SearchCancel, "SearchCancel")]
	#[case::search_term(Artifact::SearchTerm(String::from("foo")), "SearchTerm(\"foo\")")]
	#[case::searchable(
		Artifact::Searchable(Box::new(mocks::Searchable::new())),
		"Searchable(dyn Searchable)"
	)]
	#[case::diff_load(Artifact::LoadDiff(String::from("hash")), "LoadDiff(\"hash\")")]
	#[case::diff_cancel(Artifact::CancelDiff, "CancelDiff")]
	fn debug(#[case] artifact: Artifact, #[case] expected: &str) {
		assert_eq!(format!("{artifact:?}"), expected);
	}
}
