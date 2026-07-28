//! This module owns independent, repository-wide protocol conformance checks.

mod canonical;
mod cdc_profile;
mod chunk_identity;
mod corpus;
mod error;
mod external_digest;
#[cfg(test)]
mod workflow_tests;

use std::path::Path;

pub(crate) use error::ConformanceError;

/// Verifies every repository-owned protocol conformance corpus.
///
/// Chunk identity is checked before the CDC profile. The check performs
/// blocking, bounded repository reads and invokes the deadline-controlled
/// external digest witness. It returns [`ConformanceError`] at the first
/// admission, I/O, process, or verification failure.
pub(super) fn check(repository_root: &Path) -> Result<(), ConformanceError> {
    check_chunk_identity(repository_root)?;
    check_cdc_profile(repository_root)
}

/// Verifies the `ChunkId` v1 recipes, canonical preimages, and expected digests.
///
/// The check performs blocking, bounded repository reads and invokes the
/// deadline-controlled external digest witness. It returns
/// [`ConformanceError`] on malformed corpus data, I/O or process failure, or
/// any identity disagreement.
pub(super) fn check_chunk_identity(repository_root: &Path) -> Result<(), ConformanceError> {
    chunk_identity::check(repository_root)
}

/// Verifies the CDC profile v1 recipe, sources, mutations, and boundaries.
///
/// The check performs blocking, bounded repository reads, regenerates the
/// public Gear table in process, and invokes the deadline-controlled external
/// digest witness. It returns [`ConformanceError`] on malformed corpus data,
/// I/O or process failure, or any profile or boundary disagreement.
pub(super) fn check_cdc_profile(repository_root: &Path) -> Result<(), ConformanceError> {
    cdc_profile::check(repository_root)
}
