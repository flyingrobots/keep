//! This module owns canonical retention-record fuzz seeds.

use super::filesystem::RepositoryFiles;
use super::segment_store_v2_fixture;
use super::{FuzzSeedError, Seed, prefixed};

pub(super) const FIXTURES: [(u8, &str); 3] = [
    (0, "one-anchor-root.hex"),
    (1, "one-root-manifest.hex"),
    (2, "one-root-head.hex"),
];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let mut seeds = Vec::new();
    for (selector, fixture) in FIXTURES {
        let name = fixture
            .strip_suffix(".hex")
            .ok_or_else(|| FuzzSeedError::violation("retention fixture lacks .hex suffix"))?;
        let encoded = segment_store_v2_fixture::read_hex(files, fixture)?;
        seeds.push(Seed::new(
            "retention_format",
            name,
            prefixed(selector, &encoded)?,
        )?);
    }
    Ok(seeds)
}
