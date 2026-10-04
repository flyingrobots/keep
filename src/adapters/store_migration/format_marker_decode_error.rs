//! This boundary module owns store-format marker decoding failures.

/// Failure to decode and admit one version-2 store-format marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreFormatMarkerDecodeError {
    /// The input was not exactly one complete marker.
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
    /// The fixed record length field was noncanonical.
    InvalidRecordLength {
        /// Required record length.
        expected: u16,
        /// Observed record length.
        observed: u16,
    },
    /// The marker carried unsupported flags.
    UnsupportedFlags {
        /// Observed flag bits.
        observed: u32,
    },
    /// Reserved bytes were nonzero.
    NonZeroReserved {
        /// Observed reserved field.
        observed: u32,
    },
    /// The checksum did not match the exact prefix.
    ChecksumMismatch {
        /// Computed canonical checksum.
        expected: [u8; 32],
        /// Checksum stored in the record.
        observed: [u8; 32],
    },
    /// The format-definition digest was not the registered version-2 value.
    DefinitionDigestMismatch {
        /// Registered version-2 definition digest.
        expected: [u8; 32],
        /// Observed definition digest.
        observed: [u8; 32],
    },
    /// The maximum namespace count was noncanonical.
    InvalidMaximumNamespaceCount {
        /// Required namespace bound.
        expected: u32,
        /// Observed namespace bound.
        observed: u32,
    },
}
