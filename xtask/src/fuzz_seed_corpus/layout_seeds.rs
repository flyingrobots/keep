//! This module owns canonical flat-layout decoder seed derivation.

use std::path::Path;

use super::filesystem::RepositoryFiles;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex, framed_lines};

const LAYOUT_ROOT: &str = "conformance/layout/v1";
pub(super) const FIXTURES: [&str; 4] = [
    "empty.layout.hex",
    "one-zero.layout.hex",
    "max-plus-one-zeros.layout.hex",
    "zeros-long.layout.hex",
];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    FIXTURES
        .into_iter()
        .map(|fixture| seed_from_fixture(files, fixture))
        .collect()
}

fn seed_from_fixture(
    files: &RepositoryFiles,
    fixture: &'static str,
) -> Result<Seed, FuzzSeedError> {
    let relative = Path::new(LAYOUT_ROOT).join(fixture);
    let transport = files.read_bounded(&relative, MAX_SEED_BYTES)?;
    let lines = framed_lines(&transport, MAX_SEED_BYTES)
        .map_err(|source| FuzzSeedError::violation(format!("{fixture} framing moved: {source}")))?;
    let [encoded] = lines.as_slice() else {
        return Err(FuzzSeedError::violation(format!(
            "{fixture} must contain exactly one hexadecimal line"
        )));
    };
    let record = decode_lower_hex(encoded, MAX_SEED_BYTES, EmptyHex::Refuse).map_err(|source| {
        FuzzSeedError::violation(format!("{fixture} is not canonical hexadecimal: {source}"))
    })?;
    let name = fixture
        .strip_suffix(".layout.hex")
        .ok_or_else(|| FuzzSeedError::violation("layout fixture suffix moved"))?;
    Seed::new("layout_record", name, record)
}
