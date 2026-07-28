//! This module owns admitted documentation-tool versions and arguments.

use super::error::DocumentationError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DocumentationTool {
    Actionlint,
    Lychee,
    Markdownlint,
}

impl DocumentationTool {
    pub(super) const fn program(self) -> &'static str {
        match self {
            Self::Actionlint => "actionlint",
            Self::Lychee => "lychee",
            Self::Markdownlint => "markdownlint-cli2",
        }
    }

    pub(super) const fn install_version(self) -> &'static str {
        match self {
            Self::Actionlint => "1.7.12",
            Self::Lychee => "0.21.0",
            Self::Markdownlint => "0.23.2",
        }
    }

    pub(super) const fn expected_version(self) -> &'static str {
        match self {
            Self::Actionlint => "1.7.12",
            Self::Lychee => "lychee 0.21.0",
            Self::Markdownlint => "markdownlint-cli2 v0.23.2 (markdownlint v0.41.1)",
        }
    }

    pub(super) const fn version_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Actionlint => &["-version"],
            Self::Lychee => &["--version"],
            Self::Markdownlint => &["--no-globs", "--version"],
        }
    }

    pub(super) const fn check_prefix(self) -> &'static [&'static str] {
        match self {
            Self::Actionlint => &["-shellcheck=", "-pyflakes="],
            Self::Lychee => &[
                "--offline",
                "--include-fragments",
                "--no-progress",
                "--format",
                "detailed",
                "--",
            ],
            Self::Markdownlint => &["--no-globs", "--"],
        }
    }

    pub(super) fn admit_version(self, observed: &str) -> Result<(), DocumentationError> {
        let expected = self.expected_version();
        if observed == expected {
            Ok(())
        } else {
            Err(DocumentationError::VersionMismatch {
                program: self.program(),
                expected,
                observed: observed.to_owned(),
            })
        }
    }
}

#[cfg(test)]
#[path = "tool/tests.rs"]
mod tests;
