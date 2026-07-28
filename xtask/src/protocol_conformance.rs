//! This module owns independent, repository-wide protocol conformance checks.

mod canonical;
mod cdc_profile;
mod chunk_identity;
mod corpus;
mod error;
mod external_digest;

use std::path::Path;

pub(crate) use error::ConformanceError;

pub(super) fn check_chunk_identity(repository_root: &Path) -> Result<(), ConformanceError> {
    chunk_identity::check(repository_root)
}

pub(super) fn check_cdc_profile(repository_root: &Path) -> Result<(), ConformanceError> {
    cdc_profile::check(repository_root)
}
