//! This module owns canonical verification receipt fuzz seeds.

use std::path::Path;

use super::filesystem::RepositoryFiles;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex, framed_lines};

const RECEIPT_ROOT: &str = "conformance/verification-receipt/v1";

/// Every frozen receipt fixture, each one raw seed for `verification_receipt`.
pub(super) const FIXTURES: [&str; 3] = [
    "reference-complete-blob-report.hex",
    "durable-corrupt-chunk-refusal.hex",
    "reference-unsupported-framing-refusal.hex",
];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let mut seeds = Vec::new();
    for fixture in FIXTURES {
        let name = fixture
            .strip_suffix(".hex")
            .ok_or_else(|| FuzzSeedError::violation(format!("{fixture} is not a hex fixture")))?;
        seeds.push(Seed::new(
            "verification_receipt",
            name,
            read_hex(files, fixture)?,
        )?);
    }
    Ok(seeds)
}

fn read_hex(files: &RepositoryFiles, fixture: &'static str) -> Result<Vec<u8>, FuzzSeedError> {
    let relative = Path::new(RECEIPT_ROOT).join(fixture);
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
