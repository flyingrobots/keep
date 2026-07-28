//! Revalidating iterator over an admitted segment's borrowed records.

use super::segment_record_cursor::SegmentRecordCursor;
use super::{AdmittedSegmentRecord, SegmentReadError, SegmentReadPolicy};

/// Iterator over content-admitted records in physical segment order.
///
/// Each item repeats bounded record framing, checksum, and logical-identity
/// verification against immutable borrowed bytes. A failure is returned
/// explicitly rather than hidden behind the prior whole-segment proof.
pub struct SegmentRecords<'a> {
    cursor: Option<SegmentRecordCursor<'a>>,
}

impl<'a> SegmentRecords<'a> {
    pub(super) const fn new(
        records: &'a [u8],
        record_count: u32,
        policy: SegmentReadPolicy,
    ) -> Self {
        Self {
            cursor: Some(SegmentRecordCursor::new(records, record_count, policy)),
        }
    }
}

impl<'a> Iterator for SegmentRecords<'a> {
    type Item = Result<AdmittedSegmentRecord<'a>, SegmentReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor.as_mut()?;
        match cursor.next_record() {
            Ok(Some(located)) => Some(Ok(located.record)),
            Ok(None) => {
                let finished = self.cursor.take()?;
                match finished.finish() {
                    Ok(()) => None,
                    Err(error) => Some(Err(error)),
                }
            }
            Err(error) => {
                self.cursor = None;
                Some(Err(error))
            }
        }
    }
}
