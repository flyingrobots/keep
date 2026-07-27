//! This module owns admitted, platform-neutral repository paths.

use std::path::{Path, PathBuf};

use super::SourceStructureError;
use xtask::protocol_admission::posix_relative_path;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RepositoryPath {
    relative: PathBuf,
    text: String,
}

impl RepositoryPath {
    pub(super) fn admit(text: String) -> Result<Self, SourceStructureError> {
        let relative = posix_relative_path(&text)
            .map_err(|_| SourceStructureError::InvalidPath(text.clone()))?;
        Ok(Self { relative, text })
    }

    pub(super) fn as_path(&self) -> &Path {
        &self.relative
    }

    pub(super) fn as_str(&self) -> &str {
        &self.text
    }
}
