//! Byte-level fixture patching for corruption matrices.

use std::io;

/// Overwrites the bytes at `offset` with `value`.
///
/// # Errors
///
/// Returns an error when the patch would fall outside the fixture.
pub(crate) fn patch(bytes: &mut [u8], offset: usize, value: &[u8]) -> io::Result<()> {
    let end = offset
        .checked_add(value.len())
        .ok_or_else(|| io::Error::other("patch offset overflows"))?;
    bytes
        .get_mut(offset..end)
        .ok_or_else(|| io::Error::other("patch falls outside the fixture"))?
        .copy_from_slice(value);
    Ok(())
}

/// Flips the lowest bit of one byte.
///
/// # Errors
///
/// Returns an error when `offset` is outside the fixture.
pub(crate) fn flip(bytes: &mut [u8], offset: usize) -> io::Result<()> {
    let byte = bytes
        .get_mut(offset)
        .ok_or_else(|| io::Error::other("flip offset falls outside the fixture"))?;
    *byte ^= 1;
    Ok(())
}

/// Reads one big-endian `u16`.
///
/// # Errors
///
/// Returns an error when the field is outside the fixture.
pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    read_array(bytes, offset).map(u16::from_be_bytes)
}

/// Reads one big-endian `u32`.
///
/// # Errors
///
/// Returns an error when the field is outside the fixture.
pub(crate) fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    read_array(bytes, offset).map(u32::from_be_bytes)
}

fn read_array<const WIDTH: usize>(bytes: &[u8], offset: usize) -> io::Result<[u8; WIDTH]> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or_else(|| io::Error::other("read offset overflows"))?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| io::Error::other("read falls outside the fixture"))?;
    <[u8; WIDTH]>::try_from(field).map_err(|_| io::Error::other("read width mismatch"))
}

/// BLAKE3-256 over a domain string followed by the preimage.
pub(crate) fn domain_hash(domain: &[u8], preimage: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(preimage);
    *hasher.finalize().as_bytes()
}

/// BLAKE3-256 over a domain string, a big-endian `u32` count, and a body.
pub(crate) fn counted_domain_hash(domain: &[u8], count: u32, body: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&count.to_be_bytes());
    hasher.update(body);
    *hasher.finalize().as_bytes()
}
