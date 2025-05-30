use std::process::{ExitCode, Termination};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitStatus {
	None,
	Abort,
	ConfigError,
	FileReadError,
	FileWriteError,
	Good,
	StateError,
	Kill,
}

impl ExitStatus {
	pub(crate) const fn to_code(self) -> u8 {
		match self {
			Self::Abort => 5,
			Self::ConfigError => 1,
			Self::FileReadError => 2,
			Self::FileWriteError => 3,
			Self::None | Self::Good => 0,
			Self::StateError => 4,
			Self::Kill => 6,
		}
	}
}

impl Termination for ExitStatus {
	fn report(self) -> ExitCode {
		ExitCode::from(self.to_code())
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	#[case::abort(ExitStatus::None, 0)]
	#[case::abort(ExitStatus::Abort, 5)]
	#[case::config_error(ExitStatus::ConfigError, 1)]
	#[case::file_read_error(ExitStatus::FileReadError, 2)]
	#[case::file_write_error(ExitStatus::FileWriteError, 3)]
	#[case::good(ExitStatus::Good, 0)]
	#[case::state_error(ExitStatus::StateError, 4)]
	#[case::kill(ExitStatus::Kill, 6)]
	fn to_code(#[case] input: ExitStatus, #[case] expected: u8) {
		assert_eq!(ExitStatus::to_code(input), expected);
	}

	#[test]
	fn termination() {
		assert_eq!(ExitStatus::ConfigError.report(), ExitCode::from(1));
	}
}
