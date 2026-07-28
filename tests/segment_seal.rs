//! Public canonical segment-seal codec laws.

#[path = "segment_seal/framing_laws.rs"]
mod framing_laws;
#[path = "segment_seal/integrity_laws.rs"]
mod integrity_laws;
#[path = "segment_seal/length_laws.rs"]
mod length_laws;
mod support;

use std::error::Error;

use keep::{SegmentSeal, SegmentSealError};
use support::decode_hex;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEAL_OFFSET: usize = 64;

struct SegmentParts {
    prefix: Vec<u8>,
    seal: Vec<u8>,
}

#[test]
fn empty_segment_seal_matches_the_frozen_protocol() -> Result<(), Box<dyn Error>> {
    assert_golden_seal(EMPTY_SEGMENT_HEX, EMPTY_SEAL_OFFSET, 0, 192, 0)
}

#[test]
fn one_record_segment_seal_matches_the_frozen_protocol() -> Result<(), Box<dyn Error>> {
    assert_golden_seal(ONE_ZERO_SEGMENT_HEX, 209, 1, 337, 145)
}

fn assert_golden_seal(
    hex: &str,
    seal_offset: usize,
    record_count: u32,
    segment_length: u64,
    record_bytes: u64,
) -> Result<(), Box<dyn Error>> {
    let segment = segment_bytes(hex)?;
    let prefix = segment
        .get(..seal_offset)
        .ok_or("segment fixture lacks its pre-seal bytes")?;
    let encoded = segment
        .get(seal_offset..)
        .ok_or("segment fixture lacks its seal")?;
    let seal = SegmentSeal::decode(prefix, encoded)?;

    assert_eq!(seal.record_count(), record_count);
    assert_eq!(seal.bytes_before_seal(), u64::try_from(seal_offset)?);
    assert_eq!(seal.segment_length(), segment_length);
    assert_eq!(seal.record_bytes(), record_bytes);
    assert_eq!(seal.encode().as_slice(), encoded);
    Ok(())
}

fn segment_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )
    .map_err(Into::into)
}

fn canonical_empty_parts() -> Result<SegmentParts, Box<dyn Error>> {
    segment_parts(EMPTY_SEGMENT_HEX, EMPTY_SEAL_OFFSET)
}

fn segment_parts(hex: &str, seal_offset: usize) -> Result<SegmentParts, Box<dyn Error>> {
    let mut segment = segment_bytes(hex)?;
    let seal = segment
        .get(seal_offset..)
        .ok_or("segment fixture lacks its seal")?
        .to_vec();
    segment.truncate(seal_offset);
    Ok(SegmentParts {
        prefix: segment,
        seal,
    })
}

fn assert_mutation(
    prefix: &[u8],
    canonical: &[u8],
    offset: usize,
    replacement: &[u8],
    expected: SegmentSealError,
) -> Result<(), Box<dyn Error>> {
    let mutated = mutate_seal(canonical, offset, replacement)?;
    assert_eq!(SegmentSeal::decode(prefix, &mutated), Err(expected));
    Ok(())
}

fn mutate_seal(
    canonical: &[u8],
    offset: usize,
    replacement: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let end = offset
        .checked_add(replacement.len())
        .ok_or("segment-seal mutation end overflow")?;
    let mut mutated = canonical.to_vec();
    let target = mutated
        .get_mut(offset..end)
        .ok_or("segment-seal mutation is out of bounds")?;
    target.copy_from_slice(replacement);
    Ok(mutated)
}
