//! This module owns bounded UTF-8 reads of fixed repository policy files.

use std::io::{self, Read};
use std::path::Path;

use crate::repository_file::{OpenRepositoryFileError, RepositoryFileIdentity, RepositoryRoot};

use super::error::DocumentationError;

const MAX_REPOSITORY_FILE_BYTES: u64 = 1_048_576;

/// Bounded UTF-8 policy text with the file identity admitted for its bytes.
pub(super) struct RepositoryText {
    identity: RepositoryFileIdentity,
    path: &'static str,
    text: String,
}

impl RepositoryText {
    /// Returns the admitted UTF-8 policy text.
    pub(super) fn as_str(&self) -> &str {
        &self.text
    }

    /// Revalidates that the policy path still names the admitted file identity.
    pub(super) fn verify(
        &self,
        repository_root: &RepositoryRoot,
    ) -> Result<(), DocumentationError> {
        let current = match repository_root.open_file(Path::new(self.path)) {
            Ok(file) => file,
            Err(OpenRepositoryFileError::Io(source))
                if source.kind() == io::ErrorKind::NotFound =>
            {
                return Err(DocumentationError::RepositoryFileChanged(self.path));
            }
            Err(OpenRepositoryFileError::Io(source)) => {
                return Err(DocumentationError::RepositoryFileInspect {
                    path: self.path,
                    source,
                });
            }
            Err(OpenRepositoryFileError::NonRegular) => {
                return Err(DocumentationError::RepositoryFileChanged(self.path));
            }
        };
        let current = RepositoryFileIdentity::read(&current).map_err(|source| {
            DocumentationError::RepositoryFileInspect {
                path: self.path,
                source,
            }
        })?;
        if current == self.identity {
            Ok(())
        } else {
            Err(DocumentationError::RepositoryFileChanged(self.path))
        }
    }
}

/// Reads one fixed policy path through the retained repository authority.
pub(super) fn read(
    repository_root: &RepositoryRoot,
    path: &'static str,
) -> Result<RepositoryText, DocumentationError> {
    let file = repository_root
        .open_file(Path::new(path))
        .map_err(|error| open_error(path, error))?;
    let identity = RepositoryFileIdentity::read(&file)
        .map_err(|source| DocumentationError::RepositoryFileInspect { path, source })?;
    let read_bound = MAX_REPOSITORY_FILE_BYTES.checked_add(1).ok_or(
        DocumentationError::RepositoryFileTooLarge {
            path,
            maximum: MAX_REPOSITORY_FILE_BYTES,
        },
    )?;
    let mut bytes = Vec::new();
    file.take(read_bound)
        .read_to_end(&mut bytes)
        .map_err(|source| DocumentationError::RepositoryFileInspect { path, source })?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > MAX_REPOSITORY_FILE_BYTES) {
        return Err(DocumentationError::RepositoryFileTooLarge {
            path,
            maximum: MAX_REPOSITORY_FILE_BYTES,
        });
    }
    let text = String::from_utf8(bytes)
        .map_err(|source| DocumentationError::RepositoryFileEncoding { path, source })?;
    Ok(RepositoryText {
        identity,
        path,
        text,
    })
}

fn open_error(path: &'static str, error: OpenRepositoryFileError) -> DocumentationError {
    match error {
        OpenRepositoryFileError::Io(source) => {
            DocumentationError::RepositoryFileInspect { path, source }
        }
        OpenRepositoryFileError::NonRegular => DocumentationError::RepositoryFileNonRegular(path),
    }
}

#[cfg(test)]
#[path = "repository_text/tests.rs"]
mod tests;
