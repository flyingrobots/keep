//! This module owns independent, repository-wide protocol conformance checks.

mod chunk_identity;
mod corpus;
mod error;
mod external_digest;

use std::path::Path;

pub(crate) use error::ConformanceError;

pub(super) fn check_chunk_identity(repository_root: &Path) -> Result<(), ConformanceError> {
    chunk_identity::check(repository_root)
}
