//! Stateless decoding for one complete record at a segment cursor.

use super::{
    AdmittedSegmentRecord, ChecksummedSegmentRecord, SegmentReadError, SegmentReadPolicy,
    SegmentRecordHeader,
};

const RECORD_HEADER_LENGTH: usize = SegmentRecordHeader::ENCODED_LENGTH;

pub(super) struct DecodedRecord<'a> {
    pub(super) record: AdmittedSegmentRecord<'a>,
    pub(super) host_length: usize,
    pub(super) record_length: u64,
}

struct CompleteRecordBytes<'a> {
    encoded: &'a [u8],
    host_length: usize,
    record_length: u64,
}

pub(super) fn decode(
    remaining: &[u8],
    record_index: u32,
    offset: u64,
    policy: SegmentReadPolicy,
) -> Result<DecodedRecord<'_>, SegmentReadError> {
    let header = decode_header(remaining, record_index, offset)?;
    let complete = complete_record_bytes(remaining, header, record_index, offset)?;
    let record = admit_record(complete.encoded, record_index, offset, policy)?;
    Ok(DecodedRecord {
        record,
        host_length: complete.host_length,
        record_length: complete.record_length,
    })
}

fn decode_header(
    remaining: &[u8],
    record_index: u32,
    offset: u64,
) -> Result<SegmentRecordHeader, SegmentReadError> {
    let header_bytes =
        remaining
            .get(..RECORD_HEADER_LENGTH)
            .ok_or(SegmentReadError::RecordHeaderTruncated {
                record_index,
                offset,
                required: RECORD_HEADER_LENGTH,
                observed: remaining.len(),
            })?;
    SegmentRecordHeader::decode(header_bytes).map_err(|source| SegmentReadError::RecordHeader {
        record_index,
        offset,
        source,
    })
}

fn complete_record_bytes(
    remaining: &[u8],
    header: SegmentRecordHeader,
    record_index: u32,
    offset: u64,
) -> Result<CompleteRecordBytes<'_>, SegmentReadError> {
    let record_length = header.record_length().get();
    let host_length = usize::try_from(record_length).map_err(|_source| {
        SegmentReadError::RecordLengthHostWidth {
            record_index,
            observed: record_length,
        }
    })?;
    let encoded = remaining
        .get(..host_length)
        .ok_or(SegmentReadError::RecordTruncated {
            record_index,
            offset,
            expected: record_length,
            observed: remaining.len(),
        })?;
    Ok(CompleteRecordBytes {
        encoded,
        host_length,
        record_length,
    })
}

fn admit_record(
    encoded: &[u8],
    record_index: u32,
    offset: u64,
    policy: SegmentReadPolicy,
) -> Result<AdmittedSegmentRecord<'_>, SegmentReadError> {
    let checksummed = ChecksummedSegmentRecord::decode(encoded).map_err(|source| {
        SegmentReadError::RecordDecode {
            record_index,
            offset,
            source,
        }
    })?;
    checksummed
        .admit(policy.layout_entry_limit())
        .map_err(|source| SegmentReadError::RecordAdmission {
            record_index,
            offset,
            source,
        })
}
