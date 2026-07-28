//! This module owns canonical immutable-segment decoder seed derivation.

use std::path::Path;

use super::filesystem::RepositoryFiles;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed, prefixed};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex, framed_lines};

const SEGMENT_ROOT: &str = "conformance/segment-store/v1";
const HEADER_LENGTH: usize = 64;
const RECORD_HEADER_LENGTH: usize = 112;
const SEAL_LENGTH: usize = 128;

pub(super) const FIXTURES: [&str; 3] = [
    "empty-segment.hex",
    "one-zero-segment.hex",
    "one-zero-bundle-segment.hex",
];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let [empty_fixture, one_fixture, bundle_fixture] = FIXTURES;
    let empty = fixture(files, empty_fixture)?;
    let one = fixture(files, one_fixture)?;
    let bundle = fixture(files, bundle_fixture)?;
    let record_end = one
        .len()
        .checked_sub(SEAL_LENGTH)
        .ok_or_else(|| FuzzSeedError::violation("one-zero segment lacks its seal"))?;
    let record = one
        .get(HEADER_LENGTH..record_end)
        .ok_or_else(|| FuzzSeedError::violation("one-zero segment lacks its record"))?;
    let record_header = record
        .get(..RECORD_HEADER_LENGTH)
        .ok_or_else(|| FuzzSeedError::violation("one-zero record lacks its header"))?;
    let header = empty
        .get(..HEADER_LENGTH)
        .ok_or_else(|| FuzzSeedError::violation("empty segment lacks its header"))?;
    let mut seeds = vec![
        seed(0, "header-empty", header)?,
        seed(1, "record-header-one-zero", record_header)?,
        seed(2, "record-one-zero", record)?,
        seed(3, "seal-empty", &empty)?,
        seed(3, "seal-one-zero", &one)?,
    ];
    for (name, segment) in [
        ("complete-empty", empty.as_slice()),
        ("complete-one-zero", one.as_slice()),
        ("complete-bundle", bundle.as_slice()),
    ] {
        seeds.push(seed(4, name, segment)?);
    }
    Ok(seeds)
}

fn seed(selector: u8, name: &'static str, bytes: &[u8]) -> Result<Seed, FuzzSeedError> {
    Seed::new("segment_format", name, prefixed(selector, bytes)?)
}

fn fixture(files: &RepositoryFiles, fixture: &'static str) -> Result<Vec<u8>, FuzzSeedError> {
    let relative = Path::new(SEGMENT_ROOT).join(fixture);
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
