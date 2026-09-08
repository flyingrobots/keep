//! This boundary module owns store-migration receipt decoding failures.

/// Failure to decode and admit one version-2 migration receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMigrationReceiptDecodeError {
    /// The input was not exactly one complete receipt.
    WrongLength {
        /// Required fixed width.
        expected: usize,
        /// Observed input width.
        observed: usize,
    },
    /// The fixed record magic was not canonical.
    InvalidMagic {
        /// Observed 16 magic bytes.
        observed: [u8; 16],
    },
    /// The format version is unsupported.
    UnsupportedVersion {
        /// Supported version.
        expected: u16,
        /// Observed version.
        observed: u16,
    },
    /// The record-length field was noncanonical.
    InvalidRecordLength {
        /// Required record length.
        expected: u16,
        /// Observed record length.
        observed: u16,
    },
    /// The receipt carried unsupported flags.
    UnsupportedFlags {
        /// Observed flag bits.
        observed: u32,
    },
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// The receipt did not bind the supplied admitted intent.
    IntentDigestMismatch {
        /// Supplied intent digest.
        expected: [u8; 32],
        /// Receipt intent digest.
        observed: [u8; 32],
    },
    /// The receipt did not bind the intent's store identifier.
    StoreIdentifierMismatch {
        /// Supplied intent store identifier.
        expected: [u8; 32],
        /// Receipt store identifier.
        observed: [u8; 32],
    },
    /// The receipt did not bind the supplied admitted marker.
    FormatMarkerDigestMismatch {
        /// Supplied marker digest.
        expected: [u8; 32],
        /// Receipt marker digest.
        observed: [u8; 32],
    },
    /// The initial retention-state digest was not registered.
    InitialRetentionStateDigestMismatch {
        /// Registered empty-state digest.
        expected: [u8; 32],
        /// Receipt digest.
        observed: [u8; 32],
    },
    /// The initial garbage-collection-state digest was not registered.
    InitialGcStateDigestMismatch {
        /// Registered empty-state digest.
        expected: [u8; 32],
        /// Receipt digest.
        observed: [u8; 32],
    },
    /// The empty recovery-disposition-set digest was not registered.
    EmptyDispositionSetDigestMismatch {
        /// Registered empty-set digest.
        expected: [u8; 32],
        /// Receipt digest.
        observed: [u8; 32],
    },
    /// The synchronization mask carried unknown bits.
    UnsupportedSynchronizationBits {
        /// Complete supported bit set.
        supported: u64,
        /// Observed mask.
        observed: u64,
    },
    /// The synchronization mask omitted one or more mandatory bits.
    IncompleteSynchronizationMask {
        /// Complete required bit set.
        required: u64,
        /// Observed mask.
        observed: u64,
    },
}
