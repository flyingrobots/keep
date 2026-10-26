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
///
/// The caller supplies the exact registered, NUL-terminated domain. Its
/// terminator separates the fixed domain from the preimage; this helper hashes
/// their bytes unchanged and adds no length prefix or delimiter.
pub(crate) fn domain_hash(domain: &[u8], preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}

/// Hashes a named domain, a big-endian `u32` count, and a body independently.
///
/// This mirrors the counted entry-set preimages of version-2 records without
/// calling the production encoder.
pub(crate) fn counted_domain_hash(domain: &[u8], count: u32, body: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&count.to_be_bytes());
    hasher.update(body);
    *hasher.finalize().as_bytes()
}

/// Flips the lowest bit of one fixture byte.
///
/// # Errors
/// Returns a fixture error when `offset` is outside the fixture.
pub(crate) fn flip(bytes: &mut [u8], offset: usize) -> io::Result<()> {
    let byte = bytes
        .get_mut(offset)
        .ok_or_else(|| io::Error::other("fixture byte absent"))?;
    *byte ^= 1;
    Ok(())
}

/// Reads one big-endian `u32` fixture field.
///
/// # Errors
/// Returns a fixture error for overflow or an out-of-bounds field.
pub(crate) fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| io::Error::other("fixture offset overflow"))?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| io::Error::other("fixture field absent"))?;
    <[u8; 4]>::try_from(field)
        .map(u32::from_be_bytes)
        .map_err(|_| io::Error::other("fixture field width mismatch"))
}
