//! Validated logical blob identity.

use std::fmt;
use std::io::Read;

use super::hasher::{BlobHashError, BlobHasher, BlobReadError};
use super::length::BlobLength;

/// A validated identity for one exact finite logical byte sequence.
///
/// Version 1 commits to the exact ADR-0001 preimage using BLAKE3-256. The
/// identity is independent of paths, chunks, layouts, representations,
/// encryption, physical locations, and retention.
///
/// Parsing a `BlobId` proves only that the coordinate is canonical and
/// supported. It does not prove that matching content is present or retained.
///
/// # Examples
///
/// ```
/// use keep::BlobId;
///
/// let observed = BlobId::hash_bytes(b"Keep exact bytes.\n")?;
/// let expected: BlobId = concat!(
///     "keep:blob:v1:blake3-256:18:",
///     "af75d70e4993121254ac71f16c5edd02410a36f94d795e4d6064ed3122b7967d"
/// )
/// .parse()?;
/// assert_eq!(observed, expected);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlobId {
    logical_length: BlobLength,
    digest: [u8; 32],
}

impl BlobId {
    /// Constructs a `BlobId` from parts a caller has already handled
    /// correctly for its own path.
    ///
    /// # Preconditions
    ///
    /// This performs no validation of its own, and callers reach it through
    /// two distinct paths with two distinct guarantees:
    ///
    /// - [`BlobHasher`] calls this with a digest it just computed, so
    ///   `digest` genuinely is the ADR-0001 preimage output for
    ///   `logical_length`.
    /// - A boundary adapter calls this with `logical_length` and `digest`
    ///   decoded from a structurally canonical representation. Per
    ///   ADR-0001, parsing proves only that the representation is
    ///   canonical and supported; it does NOT prove that `digest` was
    ///   produced by hashing `logical_length` bytes of any real content.
    ///   Content verification requires independently hashing candidate
    ///   bytes and comparing the result.
    ///
    /// Do not call this with `logical_length`/`digest` pairs that came from
    /// neither path.
    pub(crate) const fn from_validated_parts(logical_length: BlobLength, digest: [u8; 32]) -> Self {
        Self {
            logical_length,
            digest,
        }
    }

    /// Calculates the identity of `bytes` without storing them.
    ///
    /// This operation reads the provided slice once and does not allocate.
    ///
    /// # Errors
    ///
    /// Returns [`BlobHashError`] if the slice length cannot be represented by
    /// the version-1 logical length or would overflow the identity counter.
    pub fn hash_bytes(bytes: &[u8]) -> Result<Self, BlobHashError> {
        let mut hasher = BlobHasher::new();
        hasher.update(bytes)?;
        Ok(hasher.finish())
    }

    /// Calculates an identity by synchronously reading until EOF.
    ///
    /// This blocking operation uses a fixed 8 KiB buffer. It neither stores nor
    /// allocates memory proportional to the input size. Reads returning
    /// [`Interrupted`](std::io::ErrorKind::Interrupted) are retried.
    ///
    /// ```
    /// use std::io::Cursor;
    /// use keep::BlobId;
    ///
    /// let mut source = Cursor::new(b"Keep exact bytes.\n");
    /// let streamed = BlobId::hash_reader(&mut source)?;
    /// assert_eq!(streamed, BlobId::hash_bytes(b"Keep exact bytes.\n")?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`BlobReadError`] for source I/O failures, invalid `Read`
    /// behavior, or logical length overflow.
    pub fn hash_reader<R>(reader: &mut R) -> Result<Self, BlobReadError>
    where
        R: Read + ?Sized,
    {
        BlobHasher::hash_reader(reader)
    }

    /// Returns the exact logical byte length committed by this identity.
    #[must_use]
    pub const fn logical_length(self) -> BlobLength {
        self.logical_length
    }

    /// Returns whether this identity names an empty byte sequence.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.logical_length.is_empty()
    }

    /// Returns the raw ADR-0001 digest for encoding by a boundary adapter.
    ///
    /// This exposes physical digest bytes with no framing. Callers outside
    /// the canonical binary and text codecs MUST NOT treat this as a stable
    /// public representation; use [`BlobId::encode_binary`] or the `Display`
    /// impl instead.
    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

impl fmt::Debug for BlobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlobId")
            .field("logical_length", &self.logical_length)
            .field("digest", &self.digest)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::BlobId;
    use crate::blob::length::BlobLength;

    #[test]
    fn debug_does_not_depend_on_the_text_codec() {
        let subject = BlobId::from_validated_parts(BlobLength::new(3), [7_u8; 32]);
        let observed = format!("{subject:?}");
        assert_eq!(
            observed,
            "BlobId { logical_length: BlobLength(3), digest: [7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, \
             7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7] }"
        );
        assert_ne!(observed, subject.to_string());
    }
}
