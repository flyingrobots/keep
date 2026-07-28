//! Public complete immutable-segment admission laws.

#[path = "segment/format_oracle.rs"]
pub mod format_oracle;
#[path = "segment/framing_laws.rs"]
mod framing_laws;
#[path = "segment/identity_laws.rs"]
mod identity_laws;
#[path = "segment/record_checksum_oracle.rs"]
pub mod record_checksum_oracle;
mod support;

use std::error::Error;

use keep::{
    AdmittedSegment, ChunkId, LayoutEntryLimit, SegmentReadPolicy, SegmentRecordIdentity,
    SegmentRecordLimit,
};
use support::decode_hex;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const ONE_ZERO_BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const FIRST_RECORD_OFFSET: usize = 64;
const FIRST_SEAL_OFFSET: usize = 209;

#[test]
fn complete_empty_segment_is_admitted_without_records() -> Result<(), Box<dyn Error>> {
    let encoded = segment_bytes(EMPTY_SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&encoded, maximum_policy())?;

    assert_eq!(segment.encoded(), encoded);
    assert_eq!(segment.record_count(), 0);
    assert_eq!(segment.records().count(), 0);
    Ok(())
}

#[test]
fn complete_chunk_segment_yields_only_content_admitted_records() -> Result<(), Box<dyn Error>> {
    let encoded = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&encoded, maximum_policy())?;
    let identities = segment
        .records()
        .map(|record| record.map(keep::AdmittedSegmentRecord::identity))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(segment.encoded(), encoded);
    assert_eq!(segment.record_count(), 1);
    assert_eq!(
        identities,
        vec![SegmentRecordIdentity::Chunk(ChunkId::hash_bytes(&[0])?)]
    );
    Ok(())
}

#[test]
fn complete_multirecord_segment_preserves_physical_record_order() -> Result<(), Box<dyn Error>> {
    let encoded = segment_bytes(ONE_ZERO_BUNDLE_SEGMENT_HEX)?;
    let segment = AdmittedSegment::decode(&encoded, maximum_policy())?;
    let identities = segment
        .records()
        .map(|record| record.map(keep::AdmittedSegmentRecord::identity))
        .collect::<Result<Vec<_>, _>>()?;
    let [
        SegmentRecordIdentity::Chunk(chunk_id),
        SegmentRecordIdentity::Layout(_layout_id),
    ] = identities.as_slice()
    else {
        return Err(format!("unexpected admitted record sequence: {identities:?}").into());
    };

    assert_eq!(segment.record_count(), 2);
    assert_eq!(*chunk_id, ChunkId::hash_bytes(&[0])?);
    Ok(())
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn segment_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )
    .map_err(Into::into)
}

fn one_record_prefix() -> Result<Vec<u8>, Box<dyn Error>> {
    let segment = segment_bytes(ONE_ZERO_SEGMENT_HEX)?;
    segment
        .get(..FIRST_SEAL_OFFSET)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| "segment fixture lacks its pre-seal bytes".into())
}

fn one_record_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    let prefix = one_record_prefix()?;
    prefix
        .get(FIRST_RECORD_OFFSET..)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| "segment fixture lacks its complete record".into())
}
