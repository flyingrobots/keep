//! Typed complete immutable-segment admission failures.

use std::collections::TryReserveError;

use super::{
    SegmentHeaderError, SegmentRecordAdmissionError, SegmentRecordDecodeError,
    SegmentRecordHeaderError, SegmentRecordIdentity, SegmentSealError,
};

/// A complete immutable segment failed bounded structural or identity
/// admission.
#[derive(Debug)]
pub enum SegmentReadError {
    /// The byte slice cannot contain the fixed header and seal.
    WrongLength {
        /// Smallest complete version-1 segment length.
        minimum: usize,
        /// Supplied byte count.
        observed: usize,
    },
    /// The fixed segment header failed admission.
    Header {
        /// Exact nested header refusal.
        source: SegmentHeaderError,
    },
    /// The terminal segment seal failed admission.
    Seal {
        /// Exact nested seal refusal.
        source: SegmentSealError,
    },
    /// The declared record count exceeds the caller's resource policy.
    RecordCountLimit {
        /// Configured record count cap.
        maximum: u32,
        /// Declared record count.
        observed: u32,
    },
    /// The declared record count cannot be represented on this host.
    RecordCountHostWidth {
        /// Declared record count.
        observed: u32,
    },
    /// Memory for bounded duplicate detection could not be reserved.
    IdentityIndexAllocation {
        /// Declared record count used for the exact reservation.
        record_count: u32,
        /// Allocation reservation failure.
        source: TryReserveError,
    },
    /// The next record lacks its complete fixed-width header.
    RecordHeaderTruncated {
        /// Zero-based record position.
        record_index: u32,
        /// Physical segment byte offset.
        offset: u64,
        /// Required fixed header byte count.
        required: usize,
        /// Remaining record-region byte count.
        observed: usize,
    },
    /// A complete record header failed exact admission.
    RecordHeader {
        /// Zero-based record position.
        record_index: u32,
        /// Physical segment byte offset.
        offset: u64,
        /// Exact nested header refusal.
        source: SegmentRecordHeaderError,
    },
    /// A declared record length cannot be represented on this host.
    RecordLengthHostWidth {
        /// Zero-based record position.
        record_index: u32,
        /// Declared complete record byte count.
        observed: u64,
    },
    /// The record region ends before the declared complete record.
    RecordTruncated {
        /// Zero-based record position.
        record_index: u32,
        /// Physical segment byte offset.
        offset: u64,
        /// Declared complete record byte count.
        expected: u64,
        /// Remaining record-region byte count.
        observed: usize,
    },
    /// Complete record framing or checksum verification failed.
    RecordDecode {
        /// Zero-based record position.
        record_index: u32,
        /// Physical segment byte offset.
        offset: u64,
        /// Exact nested record refusal.
        source: SegmentRecordDecodeError,
    },
    /// A checksummed record payload failed logical identity admission.
    RecordAdmission {
        /// Zero-based record position.
        record_index: u32,
        /// Physical segment byte offset.
        offset: u64,
        /// Exact nested content refusal.
        source: SegmentRecordAdmissionError,
    },
    /// Advancing the physical segment cursor overflowed.
    OffsetArithmetic {
        /// Zero-based record position.
        record_index: u32,
        /// Current physical segment byte offset.
        offset: u64,
        /// Declared complete record byte count.
        record_length: u64,
    },
    /// Advancing the zero-based record coordinate overflowed.
    RecordIndexArithmetic {
        /// Last representable record position.
        record_index: u32,
    },
    /// Advancing the remaining record count underflowed.
    RecordCountArithmetic {
        /// Current zero-based record position.
        record_index: u32,
        /// Remaining record count before the failed transition.
        remaining: u32,
    },
    /// Bytes remain after walking the exact declared record count.
    TrailingRecordBytes {
        /// First unexpected physical segment byte offset.
        offset: u64,
        /// Unexpected remaining byte count.
        observed: usize,
    },
    /// More than one record declares the same logical identity.
    DuplicateRecordIdentity {
        /// Repeated logical identity.
        identity: SegmentRecordIdentity,
        /// First zero-based record position.
        first_index: u32,
        /// Repeated zero-based record position.
        duplicate_index: u32,
        /// First physical record offset.
        first_offset: u64,
        /// Repeated physical record offset.
        duplicate_offset: u64,
    },
}
