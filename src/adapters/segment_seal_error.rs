//! Typed segment-seal admission failures.

use super::SegmentDigest;

/// A segment seal failed exact structural or cryptographic admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentSealError {
    /// The seal does not occupy the exact fixed-width frame.
    WrongLength {
        /// Required encoded byte count.
        expected: usize,
        /// Supplied encoded byte count.
        observed: usize,
    },
    /// The format discriminator does not identify a segment seal.
    InvalidMagic {
        /// Required format discriminator.
        expected: [u8; 16],
        /// Decoded format discriminator.
        observed: [u8; 16],
    },
    /// The seal version is not supported.
    UnsupportedVersion {
        /// Supported format version.
        expected: u16,
        /// Decoded format version.
        observed: u16,
    },
    /// The seal sets flags that version 1 does not define.
    UnknownFlags {
        /// Required version-1 flag value.
        expected: u16,
        /// Decoded flag value.
        observed: u16,
    },
    /// The embedded seal length is not the canonical fixed width.
    SealLength {
        /// Required embedded seal length.
        expected: u16,
        /// Decoded embedded seal length.
        observed: u16,
    },
    /// A reserved 16-bit field is nonzero.
    ReservedU16 {
        /// Required reserved value.
        expected: u16,
        /// Decoded reserved value.
        observed: u16,
    },
    /// The declared record count exceeds the format bound.
    RecordCountOutOfBounds {
        /// Largest admitted record count.
        maximum: u32,
        /// Decoded record count.
        observed: u32,
    },
    /// A reserved 32-bit field is nonzero.
    ReservedU32 {
        /// Required reserved value.
        expected: u32,
        /// Decoded reserved value.
        observed: u32,
    },
    /// The host slice length cannot be represented by the format.
    PrefixLengthHostWidth {
        /// Host-width pre-seal byte count.
        observed: usize,
    },
    /// The declared pre-seal length differs from the supplied prefix.
    BytesBeforeSeal {
        /// Supplied prefix byte count.
        expected: u64,
        /// Declared pre-seal byte count.
        observed: u64,
    },
    /// Deriving complete-segment lengths overflowed.
    LengthArithmetic {
        /// Pre-seal byte count at which derivation failed.
        bytes_before_seal: u64,
    },
    /// The complete segment length exceeds the format bound.
    SegmentLengthOutOfBounds {
        /// Largest admitted complete segment byte count.
        maximum: u64,
        /// Derived or decoded complete segment byte count.
        observed: u64,
    },
    /// The declared complete segment length is inconsistent.
    SegmentLength {
        /// Length derived from the supplied prefix and fixed-width seal.
        expected: u64,
        /// Declared complete segment byte count.
        observed: u64,
    },
    /// The declared record byte count is inconsistent.
    RecordBytes {
        /// Byte count derived by excluding the header from the prefix.
        expected: u64,
        /// Declared concatenated record byte count.
        observed: u64,
    },
    /// The seal checksum algorithm is not supported.
    SealChecksumAlgorithm {
        /// Supported checksum algorithm identifier.
        expected: u8,
        /// Decoded checksum algorithm identifier.
        observed: u8,
    },
    /// The physical segment digest algorithm is not supported.
    SegmentDigestAlgorithm {
        /// Supported digest algorithm identifier.
        expected: u8,
        /// Decoded digest algorithm identifier.
        observed: u8,
    },
    /// Reserved algorithm-extension bytes are nonzero.
    ReservedBytes {
        /// Required reserved byte sequence.
        expected: [u8; 6],
        /// Decoded reserved byte sequence.
        observed: [u8; 6],
    },
    /// Framing the digest preimage overflowed the host width.
    DigestLengthArithmetic {
        /// Prefix length at which preimage framing failed.
        prefix_length: usize,
    },
    /// The physical digest does not name the supplied segment bytes.
    SegmentDigestMismatch {
        /// Digest recomputed from the supplied prefix and canonical metadata.
        expected: SegmentDigest,
        /// Digest carried by the seal.
        observed: SegmentDigest,
    },
    /// The seal checksum does not cover the admitted seal fields.
    SealChecksumMismatch {
        /// Checksum recomputed from the canonical seal fields.
        expected: [u8; 32],
        /// Checksum carried by the seal.
        observed: [u8; 32],
    },
}
