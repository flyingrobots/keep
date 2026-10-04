//! This module owns the recovery-stage fingerprint algorithm coordinate.

/// Registered recovery-stage fingerprint algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStageFingerprintAlgorithm {
    /// Version-1 framed BLAKE3-256.
    FramedBlake3V1,
}

impl RecoveryStageFingerprintAlgorithm {
    /// Returns the canonical wire coordinate.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::FramedBlake3V1 => 1,
        }
    }
}
