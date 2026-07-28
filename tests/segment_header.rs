//! Public canonical segment-header codec laws.

#[path = "segment_header/mutation_laws.rs"]
mod mutation_laws;
mod support;

use std::error::Error;

use keep::{SegmentHeader, SegmentHeaderError};
use support::decode_hex;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");

#[test]
fn canonical_segment_header_matches_the_frozen_protocol() -> Result<(), Box<dyn Error>> {
    let segment = empty_segment()?;
    let encoded = segment
        .get(..SegmentHeader::ENCODED_LENGTH)
        .ok_or("empty-segment fixture lacks its header")?;
    let header = SegmentHeader::decode(encoded)?;

    assert_eq!(header.encode(), encoded);
    assert_eq!(header.maximum_record_payload_length(), 67_108_864);
    assert_eq!(header.maximum_segment_length(), 1_073_741_824);
    assert_eq!(header.maximum_record_count(), 1_048_576);
    Ok(())
}

#[test]
fn segment_header_refuses_wrong_width_before_field_admission() -> Result<(), Box<dyn Error>> {
    let segment = empty_segment()?;
    let truncated = segment
        .get(..SegmentHeader::ENCODED_LENGTH - 1)
        .ok_or("empty-segment fixture lacks a truncation target")?;

    assert_eq!(
        SegmentHeader::decode(truncated),
        Err(SegmentHeaderError::WrongLength {
            expected: SegmentHeader::ENCODED_LENGTH,
            observed: SegmentHeader::ENCODED_LENGTH - 1,
        })
    );
    Ok(())
}

#[test]
fn segment_header_refuses_unsupported_version_exactly() -> Result<(), Box<dyn Error>> {
    let segment = empty_segment()?;
    let mut header = segment
        .get(..SegmentHeader::ENCODED_LENGTH)
        .ok_or("empty-segment fixture lacks its header")?
        .to_vec();
    let version = header
        .get_mut(16..18)
        .ok_or("segment header lacks its version field")?;
    version.copy_from_slice(&2u16.to_be_bytes());

    assert_eq!(
        SegmentHeader::decode(&header),
        Err(SegmentHeaderError::UnsupportedVersion {
            expected: 1,
            observed: 2,
        })
    );
    Ok(())
}

fn empty_segment() -> Result<Vec<u8>, Box<dyn Error>> {
    let canonical = EMPTY_SEGMENT_HEX
        .strip_suffix('\n')
        .ok_or("empty-segment fixture must end in one LF")?;
    decode_hex(canonical).map_err(Into::into)
}
