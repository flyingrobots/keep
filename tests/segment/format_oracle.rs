//! Independent immutable-segment seal test oracle.

use std::error::Error;

use blake3::Hasher;

const VERSION: u16 = 1;
const ALGORITHM: u8 = 1;
const SEGMENT_HEADER_LENGTH: u64 = 64;
const SEAL_LENGTH: u64 = 128;
const SEGMENT_DIGEST_DOMAIN: &[u8] = b"KEEP:SEGMENT:DIGEST\0";
const SEAL_CHECKSUM_DOMAIN: &[u8] = b"KEEP:SEGMENT:SEAL:SUM\0";

/// Independently seals an exact version-1 segment prefix for hostile tests.
///
/// # Errors
///
/// Returns an error when host-width conversion or checked format arithmetic
/// fails, or when the supplied prefix is shorter than the fixed header.
pub fn seal_segment(prefix: &[u8], record_count: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let prefix_length = u64::try_from(prefix.len())?;
    let segment_length = prefix_length
        .checked_add(SEAL_LENGTH)
        .ok_or("test segment length overflow")?;
    let record_bytes = prefix_length
        .checked_sub(SEGMENT_HEADER_LENGTH)
        .ok_or("test segment prefix lacks its header")?;
    let q = seal_prefix(record_count, prefix_length, segment_length, record_bytes)?;
    let digest_length = prefix_length
        .checked_add(u64::try_from(q.len())?)
        .ok_or("test segment digest length overflow")?;
    let digest = framed_hash(SEGMENT_DIGEST_DOMAIN, &[prefix, &q], digest_length);
    let mut covered = q;
    covered.extend_from_slice(&digest);
    let checksum = framed_hash(
        SEAL_CHECKSUM_DOMAIN,
        &[&covered],
        u64::try_from(covered.len())?,
    );
    let mut segment = prefix.to_vec();
    segment.extend_from_slice(&covered);
    segment.extend_from_slice(&checksum);
    Ok(segment)
}

fn seal_prefix(
    record_count: u32,
    prefix_length: u64,
    segment_length: u64,
    record_bytes: u64,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut q = Vec::with_capacity(64);
    q.extend_from_slice(b"KEEP:SEGMENT:END");
    q.extend_from_slice(&VERSION.to_be_bytes());
    q.extend_from_slice(&0_u16.to_be_bytes());
    q.extend_from_slice(&u16::try_from(SEAL_LENGTH)?.to_be_bytes());
    q.extend_from_slice(&0_u16.to_be_bytes());
    q.extend_from_slice(&record_count.to_be_bytes());
    q.extend_from_slice(&0_u32.to_be_bytes());
    q.extend_from_slice(&prefix_length.to_be_bytes());
    q.extend_from_slice(&segment_length.to_be_bytes());
    q.extend_from_slice(&record_bytes.to_be_bytes());
    q.extend_from_slice(&[ALGORITHM, ALGORITHM]);
    q.extend_from_slice(&[0_u8; 6]);
    if q.len() != 64 {
        return Err(format!("test seal prefix must be 64 bytes, observed {}", q.len()).into());
    }
    Ok(q)
}

fn framed_hash(domain: &[u8], parts: &[&[u8]], length: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(domain);
    hasher.update(&VERSION.to_be_bytes());
    hasher.update(&[ALGORITHM]);
    for part in parts {
        hasher.update(part);
    }
    hasher.update(&length.to_be_bytes());
    *hasher.finalize().as_bytes()
}
