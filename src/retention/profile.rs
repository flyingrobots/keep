//! This module owns the closed registered retention realization-profile set.

use super::RetentionProfileAdmissionError;

const PROFILE_DIGEST: [u8; 32] = [
    0xdb, 0x1c, 0x1c, 0x1a, 0x50, 0x61, 0x3e, 0xf1, 0x1f, 0x7c, 0x0e, 0xe0, 0x88, 0x2e, 0x37, 0xb6,
    0xd2, 0x4e, 0x2d, 0xb2, 0xca, 0x57, 0x78, 0x3d, 0x01, 0x19, 0x7b, 0xa5, 0x1b, 0x61, 0xce, 0x59,
];

/// One deterministic retention realization profile implemented by Keep.
///
/// The type has private representation so future registered profiles remain
/// an additive registry change rather than an exhaustive-enum break.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegisteredRetentionProfile {
    identity: u32,
    version: u32,
    digest: [u8; 32],
}

impl RegisteredRetentionProfile {
    /// The single-canonical-witness version-1 profile.
    pub const SINGLE_CANONICAL_WITNESS_V1: Self = Self {
        identity: 1,
        version: 1,
        digest: PROFILE_DIGEST,
    };

    /// Admits an exact registered profile coordinate.
    ///
    /// # Errors
    ///
    /// Returns a typed coordinate or definition-digest mismatch.
    pub fn admit(
        identity: u32,
        version: u32,
        digest: [u8; 32],
    ) -> Result<Self, RetentionProfileAdmissionError> {
        let expected = Self::SINGLE_CANONICAL_WITNESS_V1;
        if identity != expected.identity || version != expected.version {
            return Err(RetentionProfileAdmissionError::UnsupportedCoordinate {
                expected_identity: expected.identity,
                expected_version: expected.version,
                observed_identity: identity,
                observed_version: version,
            });
        }
        if digest != expected.digest {
            return Err(RetentionProfileAdmissionError::DefinitionDigestMismatch {
                expected: expected.digest,
                observed: digest,
            });
        }
        Ok(expected)
    }

    /// Returns the registered integer identity.
    #[must_use]
    pub const fn identity(self) -> u32 {
        self.identity
    }

    /// Returns the registered profile version.
    #[must_use]
    pub const fn version(self) -> u32 {
        self.version
    }

    /// Returns the exact registered definition digest.
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
