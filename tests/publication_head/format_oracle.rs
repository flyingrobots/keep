//! Independent publication-head checksum test oracle.

#![allow(
    clippy::redundant_pub_crate,
    reason = "the snapshot law module consumes this private test oracle"
)]

use std::error::Error;

use blake3::Hasher;

const VERSION: u16 = 1;
const ALGORITHM: u8 = 1;
const ENCODED_LENGTH: usize = 128;
const CHECKSUM_OFFSET: usize = 96;
const CHECKSUM_DOMAIN: &[u8] = b"KEEP:CATHEAD:SUM\0";

pub(crate) fn seal(encoded: &mut [u8]) -> Result<(), Box<dyn Error>> {
    if encoded.len() != ENCODED_LENGTH {
        return Err("test publication head has the wrong width".into());
    }
    let covered = encoded
        .get(..CHECKSUM_OFFSET)
        .ok_or("test publication head lacks checksum input")?;
    let mut hasher = Hasher::new();
    hasher.update(CHECKSUM_DOMAIN);
    hasher.update(&VERSION.to_be_bytes());
    hasher.update(&[ALGORITHM]);
    hasher.update(covered);
    hasher.update(&u64::try_from(covered.len())?.to_be_bytes());
    let checksum = *hasher.finalize().as_bytes();
    encoded
        .get_mut(CHECKSUM_OFFSET..)
        .ok_or("test publication head lacks checksum field")?
        .copy_from_slice(&checksum);
    Ok(())
}
