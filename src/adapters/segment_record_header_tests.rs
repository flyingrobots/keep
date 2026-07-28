//! Segment-record-header construction boundary laws.

use crate::{ChunkId, ChunkLength};

use super::{SegmentRecordHeader, SegmentRecordHeaderError};

#[test]
fn chunk_header_refuses_payload_above_the_protocol_bound() -> Result<(), &'static str> {
    let observed = 67_108_865_u32;
    let length = ChunkLength::from_wire(observed).ok_or("test chunk length must be positive")?;
    let identity = ChunkId::from_validated_parts(length, [0_u8; 32]);

    assert_eq!(
        SegmentRecordHeader::for_chunk(identity),
        Err(SegmentRecordHeaderError::PayloadLengthOutOfBounds {
            record_kind: 1,
            minimum: 1,
            maximum: 67_108_864,
            observed: u64::from(observed),
        })
    );
    Ok(())
}
