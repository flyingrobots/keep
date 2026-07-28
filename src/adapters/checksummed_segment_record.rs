//! Framing- and checksum-verified borrowed segment record.

use super::{
    AdmittedSegmentRecord, SegmentRecordAdmissionError, SegmentRecordChecksum,
    SegmentRecordDecodeError, SegmentRecordHeader, SegmentRecordIdentity, segment_record_admission,
    segment_record_decoder,
};
use crate::LayoutEntryLimit;

/// A borrowed segment record with exact framing and checksum verification.
///
/// This state proves that the fixed header, payload span, and checksum agree.
/// It does not prove that the payload hashes to the logical identity declared
/// by the header. Call [`Self::admit`] for that stronger claim.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChecksummedSegmentRecord<'a> {
    header: SegmentRecordHeader,
    payload: &'a [u8],
    checksum: SegmentRecordChecksum,
}

impl<'a> ChecksummedSegmentRecord<'a> {
    /// Decodes exact complete-record framing and verifies its checksum.
    ///
    /// This operation performs no allocation or I/O.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordDecodeError`] for truncation, trailing data,
    /// malformed header fields, host-width conversion, checked arithmetic, or
    /// checksum disagreement.
    pub fn decode(encoded: &'a [u8]) -> Result<Self, SegmentRecordDecodeError> {
        segment_record_decoder::decode(encoded)
    }

    /// Verifies that the payload has the logical identity declared by the
    /// header.
    ///
    /// Chunk admission performs no allocation. Layout admission may allocate
    /// one entry collection bounded by `layout_entry_limit`.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentRecordAdmissionError`] for chunk hashing or identity
    /// disagreement, or a precise nested layout-decoding failure.
    pub fn admit(
        self,
        layout_entry_limit: LayoutEntryLimit,
    ) -> Result<AdmittedSegmentRecord<'a>, SegmentRecordAdmissionError> {
        segment_record_admission::admit(self, layout_entry_limit)
    }

    /// Returns the admitted record header.
    #[must_use]
    pub const fn header(self) -> SegmentRecordHeader {
        self.header
    }

    /// Returns the exact borrowed payload.
    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.payload
    }

    /// Returns the verified record checksum.
    #[must_use]
    pub const fn checksum(self) -> SegmentRecordChecksum {
        self.checksum
    }

    /// Returns the logical identity declared by the header.
    #[must_use]
    pub const fn identity(self) -> SegmentRecordIdentity {
        self.header.identity()
    }

    pub(super) const fn from_verified_parts(
        header: SegmentRecordHeader,
        payload: &'a [u8],
        checksum: SegmentRecordChecksum,
    ) -> Self {
        Self {
            header,
            payload,
            checksum,
        }
    }
}
