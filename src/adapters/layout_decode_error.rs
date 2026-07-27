//! Bounded canonical layout decoding failures.

use std::collections::TryReserveError;
use std::num::TryFromIntError;

use crate::{
    BlobIdBinaryParseError, LayoutIdMismatch, LayoutValidationError, StorageProfileAdmissionError,
};

/// Failure to decode, validate, or admit a canonical flat-layout record.
#[derive(Debug)]
pub enum LayoutDecodeError {
    /// Input ended before the complete fixed header.
    TruncatedHeader {
        /// Required fixed header bytes.
        expected: usize,
        /// Available input bytes.
        observed: usize,
    },
    /// The record magic did not identify a flat-layout record.
    InvalidMagic {
        /// Exact unsupported 16-byte magic.
        observed: [u8; 16],
    },
    /// The layout format version is unsupported.
    UnsupportedFormatVersion {
        /// Supported format version.
        expected: u16,
        /// Decoded unsupported version.
        observed: u16,
    },
    /// The layout codec is unsupported.
    UnsupportedCodec {
        /// Supported codec coordinate.
        expected: u16,
        /// Decoded unsupported codec.
        observed: u16,
    },
    /// One or more mandatory-to-understand flags are unknown.
    UnknownFlags {
        /// Supported zero flag word.
        expected: u32,
        /// Decoded nonzero flag word.
        observed: u32,
    },
    /// The fixed header length is not canonical.
    WrongHeaderLength {
        /// Canonical header length.
        expected: u16,
        /// Decoded noncanonical length.
        observed: u16,
    },
    /// The fixed entry length is not canonical.
    WrongEntryLength {
        /// Canonical entry length.
        expected: u16,
        /// Decoded noncanonical length.
        observed: u16,
    },
    /// The record-checksum algorithm is unsupported.
    UnsupportedChecksumAlgorithm {
        /// Supported algorithm coordinate.
        expected: u8,
        /// Decoded unsupported coordinate.
        observed: u8,
    },
    /// The chunk-hash algorithm is unsupported.
    UnsupportedChunkHashAlgorithm {
        /// Supported algorithm coordinate.
        expected: u8,
        /// Decoded unsupported coordinate.
        observed: u8,
    },
    /// The chunk identity version is unsupported.
    UnsupportedChunkIdentityVersion {
        /// Supported identity version.
        expected: u16,
        /// Decoded unsupported version.
        observed: u16,
    },
    /// A reserved header byte was nonzero.
    NonzeroReserved {
        /// Absolute byte offset of the first nonzero reserved byte.
        offset: usize,
        /// Required zero byte.
        expected: u8,
        /// Decoded nonzero byte.
        observed: u8,
    },
    /// The declared entry count exceeds the immutable protocol maximum.
    EntryCountLimitExceeded {
        /// Immutable version-1 maximum.
        maximum: u32,
        /// Declared entry count.
        observed: u32,
    },
    /// The declared entry count exceeds the caller's configured cap.
    ConfiguredEntryLimitExceeded {
        /// Caller-selected maximum.
        maximum: u32,
        /// Declared entry count.
        observed: u32,
    },
    /// The declared record length exceeds the immutable protocol maximum.
    RecordLengthLimitExceeded {
        /// Immutable version-1 maximum.
        maximum: u64,
        /// Declared record length.
        observed: u64,
    },
    /// Checked record-length arithmetic failed for the declared entry count.
    RecordLengthArithmetic {
        /// Entry count that could not produce a canonical record length.
        entry_count: u32,
    },
    /// The declared record length disagrees with framing and entry count.
    RecordLengthMismatch {
        /// Length calculated from the declared entry count.
        expected: u64,
        /// Declared record length.
        observed: u64,
    },
    /// The entry count disagrees with an otherwise exact declared length.
    EntryCountLengthMismatch {
        /// Declared entry count.
        entry_count: u32,
        /// Length calculated from that count.
        expected: u64,
        /// Exact declared and actual record length.
        observed: u64,
    },
    /// Input ended before the exact declared record length.
    TruncatedRecord {
        /// Declared record length.
        expected: u64,
        /// Available input bytes.
        observed: usize,
    },
    /// Bytes remain after the exact declared record length.
    TrailingData {
        /// Declared record length.
        expected: u64,
        /// Available input bytes.
        observed: usize,
    },
    /// A bounded wire length does not fit the host index width.
    HostRecordLengthOutOfRange {
        /// Bounded wire length.
        observed: u64,
        /// Original integer conversion failure.
        source: TryFromIntError,
    },
    /// The typed record checksum did not match the encoded record.
    ChecksumMismatch {
        /// Checksum calculated from the header and entries.
        expected: [u8; 32],
        /// Checksum carried by the record.
        observed: [u8; 32],
    },
    /// The embedded target `BlobId` coordinate is noncanonical or unsupported.
    BlobId {
        /// Exact nested binary-coordinate failure.
        source: BlobIdBinaryParseError,
    },
    /// The storage-profile identity version is unsupported.
    UnsupportedStorageProfileVersion {
        /// Supported identity version.
        expected: u16,
        /// Decoded unsupported version.
        observed: u16,
    },
    /// The storage-profile hash algorithm is unsupported.
    UnsupportedStorageProfileAlgorithm {
        /// Supported algorithm coordinate.
        expected: u8,
        /// Decoded unsupported coordinate.
        observed: u8,
    },
    /// The canonical storage-profile identity is not registered locally.
    StorageProfile {
        /// Exact registry-admission failure.
        source: StorageProfileAdmissionError,
    },
    /// The bounded entry count does not fit the host index width.
    EntryCountHostWidth {
        /// Bounded wire entry count.
        observed: u32,
        /// Original integer conversion failure.
        source: TryFromIntError,
    },
    /// Allocating the bounded entry collection failed.
    Allocation {
        /// Exact bounded entry capacity requested.
        requested: usize,
        /// Original allocation failure.
        source: TryReserveError,
    },
    /// An entry carried the forbidden zero chunk length.
    ZeroChunkLength {
        /// Zero-based entry index.
        index: u32,
    },
    /// Decoded semantic fields violated a layout structural law.
    Validation {
        /// Exact semantic validation failure.
        source: LayoutValidationError,
    },
    /// The complete calculated layout identity differed from policy.
    LayoutIdentity {
        /// Exact coordinate mismatch.
        source: LayoutIdMismatch,
    },
}
