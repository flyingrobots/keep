//! Typed complete segment-record decoding failures.

use super::{SegmentRecordChecksum, SegmentRecordHeaderError};

/// Complete record framing or checksum verification failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentRecordDecodeError {
    /// Input ended before the fixed record header was complete.
    TruncatedHeader {
        /// Required fixed header length.
        expected: usize,
        /// Supplied complete input length.
        observed: usize,
    },
    /// The fixed record header was malformed.
    Header {
        /// Precise header admission failure.
        source: SegmentRecordHeaderError,
    },
    /// The declared complete record length cannot fit the host width.
    RecordLengthHostWidth {
        /// Declared complete record length.
        observed: u64,
    },
    /// Input ended before the declared complete record was available.
    TruncatedRecord {
        /// Declared complete record length.
        expected: u64,
        /// Supplied complete input length.
        observed: usize,
    },
    /// Bytes remain after the declared complete record.
    TrailingData {
        /// Declared complete record length.
        expected: u64,
        /// Supplied complete input length.
        observed: usize,
    },
    /// The declared payload length cannot fit the host width.
    PayloadLengthHostWidth {
        /// Declared payload length.
        observed: u64,
    },
    /// Checked record framing arithmetic failed.
    RecordLengthArithmetic {
        /// Declared complete record length.
        observed: u64,
    },
    /// The supplied checksum does not bind the exact header and payload.
    ChecksumMismatch {
        /// Checksum calculated from the exact framing.
        expected: SegmentRecordChecksum,
        /// Checksum supplied by the record.
        observed: SegmentRecordChecksum,
    },
}
