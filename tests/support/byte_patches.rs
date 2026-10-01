//! This module owns bounded fixture mutation and independent checksum preimages.

use std::io;

/// Patches one field without extending the fixture.
///
/// # Errors
/// Returns a fixture error for overflow or an out-of-bounds field.
pub(crate) fn patch(bytes: &mut [u8], offset: usize, value: &[u8]) -> io::Result<()> {
    let end = offset
        .checked_add(value.len())
        .ok_or_else(|| io::Error::other("fixture offset overflow"))?;
    bytes
        .get_mut(offset..end)
        .ok_or_else(|| io::Error::other("fixture field absent"))?
        .copy_from_slice(value);
    Ok(())
}

/// Hashes the fixture's named domain and canonical preimage independently.
pub(crate) fn domain_hash(domain: &[u8], preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}
