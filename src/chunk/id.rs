//! Validated physical chunk identity.

use std::fmt;

use super::error::ChunkHashError;
use super::hasher::ChunkHasher;
use super::length::ChunkLength;

/// A validated identity for one exact, finite, nonempty chunk.
///
/// Version 1 commits to exact chunk bytes, their checked `u32` length, and a
/// `KEEP:CHUNK:DATA` domain distinct from [`crate::BlobId`]. It is independent
/// of the CDC profile, blob, layout, representation, and physical location.
///
/// Calculating a `ChunkId` commits to only the bytes supplied to that
/// calculation. It does not compare against an independently supplied
/// identity or prove that the bytes form a lawful boundary, belong to a
/// blob, or remain stored.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChunkId {
    length: ChunkLength,
    digest: [u8; 32],
}

impl ChunkId {
    pub(super) const fn from_validated_parts(length: ChunkLength, digest: [u8; 32]) -> Self {
        Self { length, digest }
    }

    /// Calculates the canonical identity of one nonempty chunk.
    ///
    /// This operation reads `bytes` once, performs no I/O, and allocates no
    /// heap memory.
    ///
    /// # Errors
    ///
    /// Returns [`ChunkHashError::Empty`] for an empty slice or
    /// [`ChunkHashError::InputLengthOutOfRange`] when its length does not fit
    /// the version-1 `u32` length field.
    pub fn hash_bytes(bytes: &[u8]) -> Result<Self, ChunkHashError> {
        if bytes.is_empty() {
            return Err(ChunkHashError::Empty);
        }
        let value = u32::try_from(bytes.len()).map_err(|_source| {
            ChunkHashError::InputLengthOutOfRange {
                observed: bytes.len(),
            }
        })?;
        let length = ChunkLength::from_validated(value);
        let mut hasher = ChunkHasher::new();
        hasher.update(bytes);
        Ok(hasher.finish(length))
    }

    /// Returns the exact number of bytes committed by this identity.
    #[must_use]
    pub const fn length(self) -> ChunkLength {
        self.length
    }
}

impl fmt::Debug for ChunkId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChunkId")
            .field("length", &self.length)
            .field("digest", &self.digest)
            .finish()
    }
}

#[cfg(test)]
#[path = "id_tests.rs"]
mod tests;
