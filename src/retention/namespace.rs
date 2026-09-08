//! This module owns admission and identity of opaque retention namespace bytes.

use std::num::NonZeroU8;

use super::{RetentionNamespaceDigest, RetentionNamespaceError};

const DIGEST_DOMAIN: &[u8] = b"keep.retention-namespace/v1\0";

/// One validated opaque retention authority namespace.
///
/// Every nonempty byte string through 255 bytes is canonical as-is. Admission
/// performs no Unicode, path, case, or application-level interpretation.
///
/// Constructing from a borrowed slice allocates one owned copy. Constructing
/// from a `Vec<u8>` consumes it; boxed-slice conversion may discard excess
/// capacity.
#[must_use]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionNamespace {
    bytes: Box<[u8]>,
    length: NonZeroU8,
}

impl RetentionNamespace {
    /// Maximum admitted namespace length in bytes.
    pub const MAXIMUM_BYTE_LENGTH: u8 = u8::MAX;

    /// Returns the exact opaque namespace bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Derives the canonical physical namespace-directory identity.
    ///
    /// The digest binds the domain, the fixed-width big-endian byte length,
    /// and the exact namespace bytes. This operation does not allocate.
    pub fn digest(&self) -> RetentionNamespaceDigest {
        let length = u16::from(self.length.get()).to_be_bytes();
        let mut hasher = blake3::Hasher::new();
        hasher.update(DIGEST_DOMAIN);
        hasher.update(&length);
        hasher.update(&self.bytes);
        RetentionNamespaceDigest::from_hash(*hasher.finalize().as_bytes())
    }

    fn admit_length(observed: usize) -> Result<NonZeroU8, RetentionNamespaceError> {
        let length = u8::try_from(observed).map_err(|_| RetentionNamespaceError::TooLong {
            maximum: Self::MAXIMUM_BYTE_LENGTH,
            observed,
        })?;
        NonZeroU8::new(length).ok_or(RetentionNamespaceError::Empty)
    }
}

impl TryFrom<Vec<u8>> for RetentionNamespace {
    type Error = RetentionNamespaceError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let length = Self::admit_length(bytes.len())?;
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
            length,
        })
    }
}

impl TryFrom<&[u8]> for RetentionNamespace {
    type Error = RetentionNamespaceError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let length = Self::admit_length(bytes.len())?;
        Ok(Self {
            bytes: Box::from(bytes),
            length,
        })
    }
}
