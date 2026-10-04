//! Owns compaction rereads bounded by previously admitted segment lengths.

use cap_std::fs::Dir;

use super::FilesystemCompactionError;
use crate::adapters::{
    CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase, SegmentDigest,
    catalog_restart_io,
};

pub(super) fn read(
    directory: &Dir,
    name: &str,
    digest: SegmentDigest,
    expected: u64,
) -> Result<Vec<u8>, FilesystemCompactionError> {
    read_exact_segment(directory, name, digest, expected)
        .map_err(|source| FilesystemCompactionError::Materialize(Box::new(source)))
}

fn read_exact_segment(
    directory: &Dir,
    name: &str,
    digest: SegmentDigest,
    expected: u64,
) -> Result<Vec<u8>, CatalogRestartError> {
    let artifact = CatalogRestartArtifact::Segment { digest };
    let (file, observed) = catalog_restart_io::open_regular(
        directory,
        name,
        artifact,
        CatalogRestartPhase::OpenSegment,
    )?;
    if observed != expected {
        return Err(CatalogRestartError::Length {
            artifact,
            minimum: expected,
            maximum: expected,
            observed,
        });
    }
    // Metadata is only an early guard. The bounded reader also refuses growth
    // after this check instead of allocating an unbounded suffix.
    catalog_restart_io::read_exact(file, artifact, CatalogRestartPhase::ReadSegment, expected)
}
