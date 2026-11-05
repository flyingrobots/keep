//! This module owns `FORMAT` marker fuzz seeds.

use super::filesystem::RepositoryFiles;
use super::segment_store_v2_fixture;
use super::{FuzzSeedError, Seed};

const FORMAT_MARKER_FIXTURE: &str = "format-marker.hex";

/// Every version-two fixture the seeds read, with the selector the copy law keys on.
pub(super) const FIXTURES: [(u8, &str); 1] = [(0, FORMAT_MARKER_FIXTURE)];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let [(_selector, fixture)] = FIXTURES;
    let marker = segment_store_v2_fixture::read_hex(files, fixture)?;
    let mut unsupported_version = marker.clone();
    *unsupported_version
        .get_mut(17)
        .ok_or_else(|| FuzzSeedError::violation("marker lacks version"))? = 3;
    let mut corrupt_checksum = marker.clone();
    *corrupt_checksum
        .last_mut()
        .ok_or_else(|| FuzzSeedError::violation("marker lacks checksum"))? ^= 1;
    Ok(vec![
        Seed::new("format_marker", "format-marker", marker)?,
        Seed::new(
            "format_marker",
            "unsupported-marker-version",
            unsupported_version,
        )?,
        Seed::new("format_marker", "corrupt-marker-checksum", corrupt_checksum)?,
    ])
}
