//! Single-pass bounded cursor over a segment's complete-record region.

use super::segment_record_cursor_decode;
use super::{AdmittedSegmentRecord, SegmentReadError, SegmentReadPolicy};

const FIRST_RECORD_OFFSET: u64 = 64;

pub(super) struct LocatedRecord<'a> {
    pub(super) record: AdmittedSegmentRecord<'a>,
    pub(super) record_index: u32,
    pub(super) offset: u64,
}

pub(super) struct SegmentRecordCursor<'a> {
    remaining: &'a [u8],
    remaining_count: u32,
    record_index: u32,
    offset: u64,
    policy: SegmentReadPolicy,
}

impl<'a> SegmentRecordCursor<'a> {
    pub(super) const fn new(
        records: &'a [u8],
        record_count: u32,
        policy: SegmentReadPolicy,
    ) -> Self {
        Self {
            remaining: records,
            remaining_count: record_count,
            record_index: 0,
            offset: FIRST_RECORD_OFFSET,
            policy,
        }
    }

    pub(super) fn next_record(&mut self) -> Result<Option<LocatedRecord<'a>>, SegmentReadError> {
        if self.remaining_count == 0 {
            return Ok(None);
        }
        let record_index = self.record_index;
        let offset = self.offset;
        let decoded = segment_record_cursor_decode::decode(
            self.remaining,
            record_index,
            offset,
            self.policy,
        )?;
        self.advance(decoded.host_length, decoded.record_length)?;
        Ok(Some(LocatedRecord {
            record: decoded.record,
            record_index,
            offset,
        }))
    }

    pub(super) const fn finish(self) -> Result<(), SegmentReadError> {
        if self.remaining.is_empty() {
            return Ok(());
        }
        Err(SegmentReadError::TrailingRecordBytes {
            offset: self.offset,
            observed: self.remaining.len(),
        })
    }

    fn advance(&mut self, host_length: usize, record_length: u64) -> Result<(), SegmentReadError> {
        let next_offset =
            self.offset
                .checked_add(record_length)
                .ok_or(SegmentReadError::OffsetArithmetic {
                    record_index: self.record_index,
                    offset: self.offset,
                    record_length,
                })?;
        let next_index =
            self.record_index
                .checked_add(1)
                .ok_or(SegmentReadError::RecordIndexArithmetic {
                    record_index: self.record_index,
                })?;
        let remaining =
            self.remaining
                .get(host_length..)
                .ok_or(SegmentReadError::RecordTruncated {
                    record_index: self.record_index,
                    offset: self.offset,
                    expected: record_length,
                    observed: self.remaining.len(),
                })?;
        let remaining_count =
            self.remaining_count
                .checked_sub(1)
                .ok_or(SegmentReadError::RecordCountArithmetic {
                    record_index: self.record_index,
                    remaining: self.remaining_count,
                })?;
        self.remaining = remaining;
        self.remaining_count = remaining_count;
        self.record_index = next_index;
        self.offset = next_offset;
        Ok(())
    }
}
