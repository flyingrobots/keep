//! Complete segment-record logical content admission.

use super::{
    AdmittedSegmentRecord, CanonicalLayoutRecord, ChecksummedSegmentRecord,
    SegmentRecordAdmissionError, SegmentRecordChecksum, SegmentRecordHeader, SegmentRecordIdentity,
};
use crate::{AdmittedLayout, ChunkId, LayoutDecodePolicy, LayoutEntryLimit};

const CHECKSUM_LENGTH: u64 = 32;

pub(super) fn admit(
    record: ChecksummedSegmentRecord<'_>,
    layout_entry_limit: LayoutEntryLimit,
) -> Result<AdmittedSegmentRecord<'_>, SegmentRecordAdmissionError> {
    match record.identity() {
        SegmentRecordIdentity::Chunk(expected) => admit_chunk(record.payload(), expected)?,
        SegmentRecordIdentity::Layout(expected) => {
            let policy = LayoutDecodePolicy::new(layout_entry_limit).with_expected_id(expected);
            AdmittedLayout::decode_record(record.payload(), policy)
                .map_err(|source| SegmentRecordAdmissionError::Layout { source })?;
        }
    }
    Ok(AdmittedSegmentRecord::from_checksummed(record))
}

pub(super) fn from_chunk(
    payload: &[u8],
) -> Result<AdmittedSegmentRecord<'_>, SegmentRecordAdmissionError> {
    let identity = ChunkId::hash_bytes(payload)
        .map_err(|source| SegmentRecordAdmissionError::ChunkHash { source })?;
    let header = SegmentRecordHeader::for_chunk(identity)
        .map_err(|source| SegmentRecordAdmissionError::Header { source })?;
    prepare(header, payload)
}

pub(super) fn from_layout(
    record: &CanonicalLayoutRecord,
) -> Result<AdmittedSegmentRecord<'_>, SegmentRecordAdmissionError> {
    let header = SegmentRecordHeader::for_layout(record.id())
        .map_err(|source| SegmentRecordAdmissionError::Header { source })?;
    prepare(header, record.bytes())
}

fn admit_chunk(payload: &[u8], expected: ChunkId) -> Result<(), SegmentRecordAdmissionError> {
    let observed = ChunkId::hash_bytes(payload)
        .map_err(|source| SegmentRecordAdmissionError::ChunkHash { source })?;
    if observed == expected {
        return Ok(());
    }
    Err(SegmentRecordAdmissionError::ChunkIdentityMismatch { expected, observed })
}

fn prepare(
    header: SegmentRecordHeader,
    payload: &[u8],
) -> Result<AdmittedSegmentRecord<'_>, SegmentRecordAdmissionError> {
    validate_payload_length(header, payload.len())?;
    let covered_length = header
        .record_length()
        .get()
        .checked_sub(CHECKSUM_LENGTH)
        .ok_or_else(|| SegmentRecordAdmissionError::RecordLengthArithmetic {
            observed: header.record_length().get(),
        })?;
    let checksum = SegmentRecordChecksum::calculate(header, payload, covered_length);
    let checksummed = ChecksummedSegmentRecord::from_verified_parts(header, payload, checksum);
    Ok(AdmittedSegmentRecord::from_checksummed(checksummed))
}

fn validate_payload_length(
    header: SegmentRecordHeader,
    observed: usize,
) -> Result<(), SegmentRecordAdmissionError> {
    let observed_u64 = u64::try_from(observed)
        .map_err(|_source| SegmentRecordAdmissionError::PayloadLengthHostWidth { observed })?;
    let expected = header.payload_length().get();
    if observed_u64 == expected {
        return Ok(());
    }
    Err(SegmentRecordAdmissionError::PayloadLengthMismatch {
        expected,
        observed: observed_u64,
    })
}
