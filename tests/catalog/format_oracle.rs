//! Independent catalog checksum and digest test oracle.
#![allow(
    clippy::redundant_pub_crate,
    reason = "the sibling mutation module consumes this private test oracle"
)]

use std::error::Error;

use blake3::Hasher;

const VERSION: u16 = 1;
const ALGORITHM: u8 = 1;
const TRAILER_LENGTH: usize = 64;
const DIGEST_LENGTH: usize = 32;
const CHECKSUM_DOMAIN: &[u8] = b"KEEP:CATALOG:SUM\0";
const DIGEST_DOMAIN: &[u8] = b"KEEP:CATALOG:DIGEST\0";

pub(crate) fn seal(encoded: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let checksum_offset = encoded
        .len()
        .checked_sub(TRAILER_LENGTH)
        .ok_or("test catalog lacks its trailer")?;
    let digest_offset = encoded
        .len()
        .checked_sub(DIGEST_LENGTH)
        .ok_or("test catalog lacks its digest")?;
    let checksum = framed_hash(
        CHECKSUM_DOMAIN,
        encoded
            .get(..checksum_offset)
            .ok_or("test catalog lacks checksum input")?,
    )?;
    encoded
        .get_mut(checksum_offset..digest_offset)
        .ok_or("test catalog lacks checksum field")?
        .copy_from_slice(&checksum);
    let digest = framed_hash(
        DIGEST_DOMAIN,
        encoded
            .get(..digest_offset)
            .ok_or("test catalog lacks digest input")?,
    )?;
    encoded
        .get_mut(digest_offset..)
        .ok_or("test catalog lacks digest field")?
        .copy_from_slice(&digest);
    Ok(())
}

fn framed_hash(domain: &[u8], input: &[u8]) -> Result<[u8; 32], Box<dyn Error>> {
    let mut hasher = Hasher::new();
    hasher.update(domain);
    hasher.update(&VERSION.to_be_bytes());
    hasher.update(&[ALGORITHM]);
    hasher.update(input);
    hasher.update(&u64::try_from(input.len())?.to_be_bytes());
    Ok(*hasher.finalize().as_bytes())
}
