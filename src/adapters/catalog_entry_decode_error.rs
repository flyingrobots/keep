//! Catalog-entry decoding failures.

use crate::LayoutIdBinaryParseError;

/// Failure to admit one fixed-width catalog entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogEntryDecodeError {
    /// The supplied entry was not exactly the fixed version-1 width.
    WrongLength {
        /// Required fixed width.
        expected: usize,
        /// Observed input width.
        observed: usize,
    },
    /// The record-kind coordinate is unsupported.
    UnknownRecordKind {
        /// Kind supplied by the entry.
        observed: u8,
    },
    /// Version-1 entry flags were nonzero.
    Flags {
        /// Canonical flag field.
        expected: u8,
        /// Observed flag field.
        observed: u8,
    },
    /// The identity width disagreed with the record kind.
    IdentityLength {
        /// Admitted record-kind code.
        record_kind: u8,
        /// Canonical meaningful identity width.
        expected: u16,
        /// Observed identity width.
        observed: u16,
    },
    /// A chunk identity declared the forbidden zero length.
    ZeroChunkLength {
        /// Observed chunk length.
        observed: u32,
    },
    /// Unused bytes in a chunk identity slot were nonzero.
    NonzeroChunkIdentityTail {
        /// Required all-zero tail.
        expected: [u8; 24],
        /// Observed tail.
        observed: [u8; 24],
    },
    /// The chunk identity and entry payload lengths disagreed.
    ChunkPayloadLengthMismatch {
        /// Length committed by the identity.
        identity_length: u32,
        /// Length declared by the entry.
        payload_length: u64,
    },
    /// The layout identity was structurally invalid.
    LayoutIdentity {
        /// Exact nested identity failure.
        source: LayoutIdBinaryParseError,
    },
    /// The layout identity and entry payload lengths disagreed.
    LayoutPayloadLengthMismatch {
        /// Length committed by the identity.
        identity_length: u64,
        /// Length declared by the entry.
        payload_length: u64,
    },
    /// The payload length exceeded kind-specific protocol bounds.
    PayloadLengthOutOfBounds {
        /// Smallest lawful payload.
        minimum: u64,
        /// Largest lawful payload.
        maximum: u64,
        /// Observed payload.
        observed: u64,
    },
    /// The top-level record offset preceded the segment header.
    RecordOffset {
        /// First lawful top-level record offset.
        minimum: u64,
        /// Observed offset.
        observed: u64,
    },
    /// The complete-record length did not equal payload plus framing.
    RecordLengthMismatch {
        /// Declared payload length.
        payload_length: u64,
        /// Canonical complete-record length.
        expected: u64,
        /// Observed complete-record length.
        observed: u64,
    },
    /// Checked complete-record length arithmetic overflowed.
    RecordLengthArithmetic {
        /// Declared payload length.
        payload_length: u64,
    },
    /// Checked record-span arithmetic overflowed.
    RecordSpanArithmetic {
        /// Declared record offset.
        record_offset: u64,
        /// Declared complete-record length.
        record_length: u64,
    },
    /// Version-1 reserved bytes were nonzero.
    Reserved {
        /// Required all-zero bytes.
        expected: [u8; 8],
        /// Observed reserved bytes.
        observed: [u8; 8],
    },
}
