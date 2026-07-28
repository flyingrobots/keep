//! Independent complete-record checksum test oracle.

use std::error::Error;

use blake3::Hasher;

const VERSION: u16 = 1;
const ALGORITHM: u8 = 1;
const CHECKSUM_LENGTH: usize = 32;
const CHECKSUM_DOMAIN: &[u8] = b"KEEP:SEG:RECORD:SUM\0";

/// Rewrites a complete test record's checksum after a deliberate mutation.
///
/// # Errors
///
/// Returns an error when the supplied record lacks its checksum or the covered
/// length cannot be represented by the format coordinate.
pub fn rewrite_record_checksum(record: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let checksum_offset = record
        .len()
        .checked_sub(CHECKSUM_LENGTH)
        .ok_or("test record lacks its checksum")?;
    let covered = record
        .get(..checksum_offset)
        .ok_or("test record checksum offset is invalid")?;
    let checksum = framed_hash(covered, u64::try_from(covered.len())?);
    let target = record
        .get_mut(checksum_offset..)
        .ok_or("test record lacks its checksum target")?;
    target.copy_from_slice(&checksum);
    Ok(())
}

fn framed_hash(covered: &[u8], length: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(&VERSION.to_be_bytes());
    hasher.update(&[ALGORITHM]);
    hasher.update(covered);
    hasher.update(&length.to_be_bytes());
    *hasher.finalize().as_bytes()
}
