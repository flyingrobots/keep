//! This module owns bounded UTF-8 reads of fixed repository policy files.

use std::io::Read;
use std::path::Path;

use crate::repository_file::{OpenRepositoryFileError, RepositoryRoot};

use super::error::DocumentationError;

const MAX_REPOSITORY_FILE_BYTES: u64 = 1_048_576;

pub(super) fn read(
    repository_root: &RepositoryRoot,
    path: &'static str,
) -> Result<String, DocumentationError> {
    let file = repository_root
        .open_file(Path::new(path))
        .map_err(|error| open_error(path, error))?;
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
    String::from_utf8(bytes)
        .map_err(|source| DocumentationError::RepositoryFileEncoding { path, source })
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
