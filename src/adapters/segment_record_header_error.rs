//! Typed segment-record-header admission failures.

use super::LayoutIdBinaryParseError;

/// A canonical segment-record header failed exact version-1 admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentRecordHeaderError {
    /// The supplied header has the wrong exact byte length.
    WrongLength {
        /// Required version-1 length.
        expected: usize,
        /// Supplied byte length.
        observed: usize,
    },
    /// The record magic does not name the version-1 grammar.
    InvalidMagic {
        /// Required version-1 magic.
        expected: [u8; 16],
        /// Supplied 16-byte magic.
        observed: [u8; 16],
    },
    /// The record version is unsupported.
    UnsupportedVersion {
        /// Supported version.
        expected: u16,
        /// Supplied version.
        observed: u16,
    },
    /// The record kind is unsupported.
    UnknownRecordKind {
        /// Supplied kind.
        observed: u8,
    },
    /// Unknown mandatory flags were supplied.
    UnknownFlags {
        /// Version-1 value.
        expected: u8,
        /// Supplied flags.
        observed: u8,
    },
    /// The fixed header length disagrees with version 1.
    HeaderLength {
        /// Version-1 length.
        expected: u16,
        /// Supplied length.
        observed: u16,
    },
    /// The identity width disagrees with the record kind.
    IdentityLength {
        /// Record kind being admitted.
        record_kind: u8,
        /// Required identity width.
        expected: u16,
        /// Supplied identity width.
        observed: u16,
    },
    /// The payload length is outside the record-kind bounds.
    PayloadLengthOutOfBounds {
        /// Record kind being admitted.
        record_kind: u8,
        /// Smallest permitted payload.
        minimum: u64,
        /// Largest permitted payload.
        maximum: u64,
        /// Supplied payload length.
        observed: u64,
    },
    /// Checked complete-record-length arithmetic failed.
    RecordLengthArithmetic {
        /// Payload length being framed.
        payload_length: u64,
    },
    /// The complete record length disagrees with the payload framing.
    RecordLength {
        /// Required complete length.
        expected: u64,
        /// Supplied complete length.
        observed: u64,
    },
    /// The record-checksum algorithm is unsupported.
    RecordChecksumAlgorithm {
        /// Supported algorithm.
        expected: u8,
        /// Supplied algorithm.
        observed: u8,
    },
    /// The logical-identity version is unsupported.
    IdentityVersion {
        /// Supported version.
        expected: u16,
        /// Supplied version.
        observed: u16,
    },
    /// The logical-identity algorithm is unsupported.
    IdentityAlgorithm {
        /// Supported algorithm.
        expected: u8,
        /// Supplied algorithm.
        observed: u8,
    },
    /// A reserved four-byte field is nonzero.
    ReservedBytes {
        /// Byte offset of the field.
        offset: u16,
        /// Required zero bytes.
        expected: [u8; 4],
        /// Supplied bytes.
        observed: [u8; 4],
    },
    /// The chunk identity carries an invalid zero length.
    ZeroChunkLength {
        /// Supplied identity length.
        observed: u32,
    },
    /// The chunk identity length disagrees with the payload length.
    ChunkPayloadLengthMismatch {
        /// Length embedded in the chunk identity.
        identity_length: u32,
        /// Header payload length.
        payload_length: u64,
    },
    /// The unused tail of a chunk identity slot is nonzero.
    NonzeroChunkIdentityTail {
        /// Required zero bytes.
        expected: [u8; 24],
        /// Supplied bytes.
        observed: [u8; 24],
    },
    /// The canonical flat-layout identity could not be parsed.
    LayoutIdentity {
        /// Precise identity-coordinate failure.
        source: LayoutIdBinaryParseError,
    },
    /// The layout identity length disagrees with the payload length.
    LayoutPayloadLengthMismatch {
        /// Length embedded in the layout identity.
        identity_length: u64,
        /// Header payload length.
        payload_length: u64,
    },
}
