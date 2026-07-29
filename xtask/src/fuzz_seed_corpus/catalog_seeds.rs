//! This module owns canonical catalog and publication-head fuzz seeds.

use std::path::Path;

use super::filesystem::RepositoryFiles;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed, prefixed};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex, framed_lines};

const SEGMENT_STORE_ROOT: &str = "conformance/segment-store/v1";

pub(super) const FIXTURES: [(u8, &str); 6] = [
    (0, "one-zero-catalog.hex"),
    (0, "one-zero-catalog-generation-two.hex"),
    (0, "one-zero-bundle-catalog.hex"),
    (1, "one-zero-head.hex"),
    (1, "one-zero-head-generation-two.hex"),
    (1, "one-zero-bundle-head.hex"),
];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let mut seeds = Vec::new();
    for (selector, fixture) in FIXTURES {
        let name = fixture
            .strip_suffix(".hex")
            .ok_or_else(|| FuzzSeedError::violation("catalog fixture lacks .hex suffix"))?;
        let encoded = fixture_bytes(files, fixture)?;
        seeds.push(Seed::new(
            "catalog_format",
            name,
            prefixed(selector, &encoded)?,
        )?);
    }
    Ok(seeds)
}

fn fixture_bytes(files: &RepositoryFiles, fixture: &'static str) -> Result<Vec<u8>, FuzzSeedError> {
    let relative = Path::new(SEGMENT_STORE_ROOT).join(fixture);
    let transport = files.read_bounded(&relative, MAX_SEED_BYTES)?;
    let lines = framed_lines(&transport, MAX_SEED_BYTES)
        .map_err(|source| FuzzSeedError::violation(format!("{fixture} framing moved: {source}")))?;
    let [encoded] = lines.as_slice() else {
        return Err(FuzzSeedError::violation(format!(
            "{fixture} must contain exactly one hexadecimal line"
        )));
    };
    decode_lower_hex(encoded, MAX_SEED_BYTES, EmptyHex::Refuse).map_err(|source| {
        FuzzSeedError::violation(format!("{fixture} is not canonical hexadecimal: {source}"))
    })
}
