//! Validated environment and source coordinates for a report.

use std::num::NonZeroUsize;

use crate::ReportError;

/// Git worktree state recorded with a baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceTreeState {
    /// No tracked or untracked changes were present.
    Clean,
    /// Tracked or untracked changes were present.
    Dirty,
}

/// Validated build and source environment for one baseline artifact.
pub struct BaselineEnvironment {
    pub(super) git_commit: String,
    pub(super) source_tree: SourceTreeState,
    pub(super) rustc_version: String,
    pub(super) target_triple: String,
    pub(super) host: HostDescription,
}

/// Validated operating-system and processor coordinates for one report.
///
/// Fields remain private so callers cannot bypass admission. Construction
/// takes ownership of existing strings, performs no I/O, and does not allocate
/// beyond caller-provided storage.
pub struct HostDescription {
    pub(super) os_description: String,
    pub(super) cpu_model: String,
    pub(super) logical_cpu_count: NonZeroUsize,
}

impl SourceTreeState {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Dirty => "dirty",
        }
    }
}

impl BaselineEnvironment {
    /// Admits unambiguous, bounded environment fields.
    ///
    /// # Errors
    ///
    /// Returns [`ReportError`] if the commit is not a 40-digit lowercase
    /// hexadecimal object ID or any text field is empty, contains controls,
    /// or exceeds 1,024 bytes.
    pub fn new(
        git_commit: String,
        source_tree: SourceTreeState,
        rustc_version: String,
        target_triple: String,
        host: HostDescription,
    ) -> Result<Self, ReportError> {
        if git_commit.len() != 40
            || !git_commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ReportError::InvalidEnvironmentField {
                field: "git-commit",
            });
        }
        validate_field("rustc-version", &rustc_version)?;
        validate_field("target-triple", &target_triple)?;
        Ok(Self {
            git_commit,
            source_tree,
            rustc_version,
            target_triple,
            host,
        })
    }
}

impl HostDescription {
    /// Admits unambiguous, nonzero host coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`ReportError`] if the OS or CPU description is empty, contains
    /// controls, or exceeds 1,024 bytes. This synchronous validation performs
    /// no I/O and takes ownership of both strings without copying them.
    pub fn new(
        os_description: String,
        cpu_model: String,
        logical_cpu_count: NonZeroUsize,
    ) -> Result<Self, ReportError> {
        validate_field("os-description", &os_description)?;
        validate_field("cpu-model", &cpu_model)?;
        Ok(Self {
            os_description,
            cpu_model,
            logical_cpu_count,
        })
    }
}

fn validate_field(field: &'static str, value: &str) -> Result<(), ReportError> {
    if value.is_empty()
        || value.len() > 1_024
        || value
            .chars()
            .any(|character| character.is_control() || character == '\t')
    {
        Err(ReportError::InvalidEnvironmentField { field })
    } else {
        Ok(())
    }
}
