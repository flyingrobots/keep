//! Kind-specific segment-record logical-identity admission.

use super::segment_record_header_decoder::DecodedFields;
use super::segment_record_kind::SegmentRecordKind;
use super::{SegmentRecordHeaderError, SegmentRecordIdentity};
use crate::{ChunkId, ChunkLength, LayoutId};

pub(super) fn admit(
    fields: &DecodedFields,
    kind: SegmentRecordKind,
) -> Result<SegmentRecordIdentity, SegmentRecordHeaderError> {
    match kind {
        SegmentRecordKind::Chunk => admit_chunk(fields),
        SegmentRecordKind::Layout => admit_layout(fields),
    }
}

fn admit_chunk(fields: &DecodedFields) -> Result<SegmentRecordIdentity, SegmentRecordHeaderError> {
    let decoded = decode_chunk_slot(fields)?;
    let length_value = u32::from_be_bytes(decoded.length);
    let length =
        ChunkLength::from_wire(length_value).ok_or(SegmentRecordHeaderError::ZeroChunkLength {
            observed: length_value,
        })?;
    if u64::from(length_value) != fields.payload_length {
        return Err(SegmentRecordHeaderError::ChunkPayloadLengthMismatch {
            identity_length: length_value,
            payload_length: fields.payload_length,
        });
    }
    let expected = [0_u8; 24];
    if decoded.unused != expected {
        return Err(SegmentRecordHeaderError::NonzeroChunkIdentityTail {
            expected,
            observed: decoded.unused,
        });
    }
    Ok(SegmentRecordIdentity::Chunk(ChunkId::from_validated_parts(
        length,
        decoded.digest,
    )))
}

const fn decode_chunk_slot(
    fields: &DecodedFields,
) -> Result<DecodedChunkIdentity, SegmentRecordHeaderError> {
    let Some((length, remainder)) = fields.identity.split_first_chunk::<4>() else {
        return Err(chunk_identity_length(fields.identity_length));
    };
    let Some((digest, remainder)) = remainder.split_first_chunk::<32>() else {
        return Err(chunk_identity_length(fields.identity_length));
    };
    let Some((unused, trailing)) = remainder.split_first_chunk::<24>() else {
        return Err(chunk_identity_length(fields.identity_length));
    };
    if !trailing.is_empty() {
        return Err(chunk_identity_length(fields.identity_length));
    }
    Ok(DecodedChunkIdentity {
        length: *length,
        digest: *digest,
        unused: *unused,
    })
}

const fn chunk_identity_length(observed: u16) -> SegmentRecordHeaderError {
    SegmentRecordHeaderError::IdentityLength {
        record_kind: SegmentRecordKind::Chunk.code(),
        expected: SegmentRecordKind::Chunk.identity_length(),
        observed,
    }
}

fn admit_layout(fields: &DecodedFields) -> Result<SegmentRecordIdentity, SegmentRecordHeaderError> {
    let identity = LayoutId::parse_binary(&fields.identity)
        .map_err(|source| SegmentRecordHeaderError::LayoutIdentity { source })?;
    let identity_length = identity.plan_length().get();
    if identity_length != fields.payload_length {
        return Err(SegmentRecordHeaderError::LayoutPayloadLengthMismatch {
            identity_length,
            payload_length: fields.payload_length,
        });
    }
    Ok(SegmentRecordIdentity::Layout(identity))
}

struct DecodedChunkIdentity {
    length: [u8; 4],
    digest: [u8; 32],
    unused: [u8; 24],
}
