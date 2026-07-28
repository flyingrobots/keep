//! Content-admitted borrowed segment record.

use super::{
    CanonicalLayoutRecord, ChecksummedSegmentRecord, SegmentRecordAdmissionError,
    SegmentRecordChecksum, SegmentRecordHeader, SegmentRecordIdentity, segment_record_admission,
};

/// A borrowed segment record whose payload matches its declared logical
/// identity.
///
/// This state also carries the framing and checksum proof established by
/// [`ChecksummedSegmentRecord`]. It performs no I/O and makes no durability,
/// publication, retention, or natural-boundary claim.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmittedSegmentRecord<'a> {
    checksummed: ChecksummedSegmentRecord<'a>,
}

impl<'a> AdmittedSegmentRecord<'a> {
    /// Prepares a canonical admitted record over exact chunk bytes.
    ///
    /// This operation performs no allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordAdmissionError`] when chunk hashing, protocol
    /// bounds, host-width conversion, or checked framing arithmetic fails.
    pub fn for_chunk(payload: &'a [u8]) -> Result<Self, SegmentRecordAdmissionError> {
        segment_record_admission::from_chunk(payload)
    }

    /// Prepares a canonical admitted record over a canonical flat-layout
    /// record.
    ///
    /// This operation performs no additional allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordAdmissionError`] when protocol bounds,
    /// host-width conversion, or checked framing arithmetic fails.
    pub fn for_layout(
        record: &'a CanonicalLayoutRecord,
    ) -> Result<Self, SegmentRecordAdmissionError> {
        segment_record_admission::from_layout(record)
    }

    /// Returns the canonical record header.
    #[must_use]
    pub const fn header(self) -> SegmentRecordHeader {
        self.checksummed.header()
    }

    /// Returns the exact borrowed payload.
    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.checksummed.payload()
    }

    /// Returns the verified or calculated record checksum.
    #[must_use]
    pub const fn checksum(self) -> SegmentRecordChecksum {
        self.checksummed.checksum()
    }

    /// Returns the content-verified logical identity.
    #[must_use]
    pub const fn identity(self) -> SegmentRecordIdentity {
        self.checksummed.identity()
    }

    pub(super) const fn from_checksummed(checksummed: ChecksummedSegmentRecord<'a>) -> Self {
        Self { checksummed }
    }
}
