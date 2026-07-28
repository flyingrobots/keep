//! Public canonical segment-record-header codec laws.

#[path = "segment_record_header/framing_laws.rs"]
mod framing_laws;
#[path = "segment_record_header/identity_laws.rs"]
mod identity_laws;
mod support;

use std::error::Error;

use keep::{ChunkId, LayoutId, SegmentRecordHeader, SegmentRecordIdentity};
use support::decode_hex;

const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const ONE_ZERO_BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const FIRST_RECORD_OFFSET: usize = 64;
const SECOND_RECORD_OFFSET: usize = 209;

#[test]
fn canonical_chunk_record_header_matches_the_frozen_protocol() -> Result<(), Box<dyn Error>> {
    let encoded = record_header(ONE_ZERO_SEGMENT_HEX, FIRST_RECORD_OFFSET)?;
    let header = SegmentRecordHeader::decode(&encoded)?;
    let chunk_id = ChunkId::hash_bytes(&[0])?;

    assert_eq!(header.identity(), SegmentRecordIdentity::Chunk(chunk_id));
    assert_eq!(header.payload_length().get(), 1);
    assert_eq!(header.record_length().get(), 145);
    assert_eq!(header.encode().as_slice(), encoded);
    assert_eq!(
        SegmentRecordHeader::for_chunk(chunk_id)?
            .encode()
            .as_slice(),
        encoded
    );
    Ok(())
}

#[test]
fn canonical_layout_record_header_matches_the_frozen_protocol() -> Result<(), Box<dyn Error>> {
    let encoded = record_header(ONE_ZERO_BUNDLE_SEGMENT_HEX, SECOND_RECORD_OFFSET)?;
    let identity = encoded
        .get(48..108)
        .ok_or("layout record header lacks its identity slot")?;
    let layout_id = LayoutId::parse_binary(identity)?;
    let header = SegmentRecordHeader::decode(&encoded)?;

    assert_eq!(header.identity(), SegmentRecordIdentity::Layout(layout_id));
    assert_eq!(header.payload_length().get(), 220);
    assert_eq!(header.record_length().get(), 364);
    assert_eq!(header.encode().as_slice(), encoded);
    assert_eq!(
        SegmentRecordHeader::for_layout(layout_id)?
            .encode()
            .as_slice(),
        encoded
    );
    Ok(())
}

fn record_header(hex: &str, offset: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let canonical = hex
        .strip_suffix('\n')
        .ok_or("segment fixture must end in one LF")?;
    let segment = decode_hex(canonical)?;
    let end = offset
        .checked_add(SegmentRecordHeader::ENCODED_LENGTH)
        .ok_or("record-header fixture offset overflow")?;
    segment
        .get(offset..end)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| "segment fixture lacks the requested record header".into())
}

fn assert_mutation(
    canonical: &[u8],
    offset: usize,
    replacement: &[u8],
    expected: keep::SegmentRecordHeaderError,
) -> Result<(), Box<dyn Error>> {
    let mutated = mutate_header(canonical, offset, replacement)?;
    assert_eq!(SegmentRecordHeader::decode(&mutated), Err(expected));
    Ok(())
}

fn mutate_header(
    canonical: &[u8],
    offset: usize,
    replacement: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let end = offset
        .checked_add(replacement.len())
        .ok_or("record-header mutation end overflow")?;
    let mut mutated = canonical.to_vec();
    let target = mutated
        .get_mut(offset..end)
        .ok_or("record-header mutation is out of bounds")?;
    target.copy_from_slice(replacement);
    Ok(mutated)
}
