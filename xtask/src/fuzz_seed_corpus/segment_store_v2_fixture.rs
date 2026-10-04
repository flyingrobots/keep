//! This module owns bounded admission of version-2 hexadecimal seed fixtures.

use std::path::Path;

use super::filesystem::RepositoryFiles;
use super::{FuzzSeedError, MAX_SEED_BYTES};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex, framed_lines};

const SEGMENT_STORE_ROOT: &str = "conformance/segment-store/v2";

pub(super) fn read_hex(
    files: &RepositoryFiles,
    fixture: &'static str,
) -> Result<Vec<u8>, FuzzSeedError> {
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
