//! Internal logical identity to admitted-record binding.

use super::{AdmittedSegmentRecord, SegmentRecordIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CatalogRecordBinding<'a> {
    identity: SegmentRecordIdentity,
    record: AdmittedSegmentRecord<'a>,
}

impl<'a> CatalogRecordBinding<'a> {
    pub(super) const fn new(
        identity: SegmentRecordIdentity,
        record: AdmittedSegmentRecord<'a>,
    ) -> Self {
        Self { identity, record }
    }

    pub(super) const fn identity(self) -> SegmentRecordIdentity {
        self.identity
    }

    pub(super) const fn record(self) -> AdmittedSegmentRecord<'a> {
        self.record
    }
}
