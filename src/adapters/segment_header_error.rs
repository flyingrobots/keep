//! Typed segment-header admission failures.

/// A canonical segment header failed exact version-1 admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentHeaderError {
    /// The supplied header has the wrong exact byte length.
    WrongLength {
        /// Required version-1 length.
        expected: usize,
        /// Supplied byte length.
        observed: usize,
    },
    /// The segment magic does not name the version-1 grammar.
    InvalidMagic {
        /// Required version-1 magic.
        expected: [u8; 16],
        /// Supplied 16-byte magic.
        observed: [u8; 16],
    },
    /// The format version is unsupported.
    UnsupportedVersion {
        /// Supported version.
        expected: u16,
        /// Supplied version.
        observed: u16,
    },
    /// Unknown mandatory flags were supplied.
    UnknownFlags {
        /// Version-1 value.
        expected: u16,
        /// Supplied flags.
        observed: u16,
    },
    /// The fixed segment-header length disagrees with version 1.
    HeaderLength {
        /// Version-1 length.
        expected: u16,
        /// Supplied length.
        observed: u16,
    },
    /// The fixed record-header length disagrees with version 1.
    RecordHeaderLength {
        /// Version-1 length.
        expected: u16,
        /// Supplied length.
        observed: u16,
    },
    /// The fixed seal length disagrees with version 1.
    SealLength {
        /// Version-1 length.
        expected: u16,
        /// Supplied length.
        observed: u16,
    },
    /// A reserved two-byte field is nonzero.
    ReservedU16 {
        /// Byte offset of the field.
        offset: u16,
        /// Required zero value.
        expected: u16,
        /// Supplied value.
        observed: u16,
    },
    /// The immutable record-payload bound disagrees with version 1.
    MaximumRecordPayloadLength {
        /// Version-1 bound.
        expected: u64,
        /// Supplied bound.
        observed: u64,
    },
    /// The immutable segment-length bound disagrees with version 1.
    MaximumSegmentLength {
        /// Version-1 bound.
        expected: u64,
        /// Supplied bound.
        observed: u64,
    },
    /// The immutable record-count bound disagrees with version 1.
    MaximumRecordCount {
        /// Version-1 bound.
        expected: u32,
        /// Supplied bound.
        observed: u32,
    },
    /// The record-checksum algorithm is unsupported.
    RecordChecksumAlgorithm {
        /// Supported algorithm.
        expected: u8,
        /// Supplied algorithm.
        observed: u8,
    },
    /// The segment-digest algorithm is unsupported.
    SegmentDigestAlgorithm {
        /// Supported algorithm.
        expected: u8,
        /// Supplied algorithm.
        observed: u8,
    },
    /// The trailing reserved field is nonzero.
    ReservedBytes {
        /// Byte offset of the field.
        offset: u16,
        /// Required zero bytes.
        expected: [u8; 14],
        /// Supplied bytes.
        observed: [u8; 14],
    },
}
