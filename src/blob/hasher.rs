//! One-pass logical byte identity calculation.

use std::error::Error;
use std::fmt;
use std::io::{self, ErrorKind, Read};

use super::id::BlobId;
use super::length::BlobLength;

const DATA_MAGIC: &[u8; 16] = b"KEEP:BLOB:DATA\0\0";
const IDENTITY_VERSION_BYTES: [u8; 2] = 1_u16.to_be_bytes();
const HASH_ALGORITHM: [u8; 1] = [1];
const READER_BUFFER_BYTES: usize = 8_192;

/// Incremental version-1 `BlobId` calculator.
///
/// Each update is incorporated exactly once. Finishing consumes the calculator
/// and appends the checked logical length required by ADR-0001.
#[must_use = "a BlobHasher has no result until finish is called"]
pub struct BlobHasher {
    state: blake3::Hasher,
    logical_length: BlobLength,
}

impl BlobHasher {
    /// Starts a version-1 identity calculation.
    pub fn new() -> Self {
        let mut state = blake3::Hasher::new();
        state.update(DATA_MAGIC);
        state.update(&IDENTITY_VERSION_BYTES);
        state.update(&HASH_ALGORITHM);
        Self {
            state,
            logical_length: BlobLength::ZERO,
        }
    }

    /// Incorporates the next exact logical bytes.
    ///
    /// Empty updates are lawful. If length accounting fails, neither the hash
    /// state nor accumulated length changes.
    ///
    /// # Errors
    ///
    /// Returns [`BlobHashError::InputLengthOutOfRange`] when the platform slice
    /// length does not fit in `u64`, or [`BlobHashError::LogicalLengthOverflow`]
    /// when the total would exceed the version-1 maximum.
    pub fn update(&mut self, bytes: &[u8]) -> Result<(), BlobHashError> {
        let incoming_value =
            u64::try_from(bytes.len()).map_err(|_source| BlobHashError::InputLengthOutOfRange {
                observed: bytes.len(),
            })?;
        let incoming = BlobLength::new(incoming_value);
        let next = self.logical_length.checked_add(incoming).ok_or(
            BlobHashError::LogicalLengthOverflow {
                accumulated: self.logical_length,
                incoming,
            },
        )?;
        self.state.update(bytes);
        self.logical_length = next;
        Ok(())
    }

    /// Finishes the calculation and returns the exact logical identity.
    #[must_use]
    pub fn finish(mut self) -> BlobId {
        self.state.update(&self.logical_length.get().to_be_bytes());
        let digest = *self.state.finalize().as_bytes();
        BlobId::from_validated_parts(self.logical_length, digest)
    }

    pub(super) fn hash_reader<R>(reader: &mut R) -> Result<BlobId, BlobReadError>
    where
        R: Read + ?Sized,
    {
        let mut hasher = Self::new();
        let mut buffer = [0_u8; READER_BUFFER_BYTES];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return Ok(hasher.finish()),
                Ok(observed) => {
                    let bytes = buffer
                        .get(..observed)
                        .ok_or(BlobReadError::InvalidReadCount {
                            maximum: buffer.len(),
                            observed,
                        })?;
                    hasher.update(bytes).map_err(BlobReadError::Hash)?;
                }
                Err(source) if source.kind() == ErrorKind::Interrupted => {}
                Err(source) => return Err(BlobReadError::Read { source }),
            }
        }
    }
}

impl Default for BlobHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Failure while incrementally calculating a logical identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobHashError {
    /// One input slice cannot be represented by the version-1 length field.
    InputLengthOutOfRange {
        /// Platform slice length that could not be represented.
        observed: usize,
    },
    /// Incorporating an input slice would exceed `u64::MAX` logical bytes.
    LogicalLengthOverflow {
        /// Length successfully incorporated before the refused update.
        accumulated: BlobLength,
        /// Length of the refused update.
        incoming: BlobLength,
    },
}

impl fmt::Display for BlobHashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLengthOutOfRange { observed } => {
                write!(
                    formatter,
                    "input length {observed} cannot be represented by BlobId v1"
                )
            }
            Self::LogicalLengthOverflow {
                accumulated,
                incoming,
            } => write!(
                formatter,
                "logical length overflow after {accumulated} bytes with {incoming} incoming bytes"
            ),
        }
    }
}

impl Error for BlobHashError {}

/// Failure while calculating an identity from a blocking byte reader.
#[derive(Debug)]
pub enum BlobReadError {
    /// The byte source returned an I/O error other than interruption.
    Read {
        /// Original source error.
        source: io::Error,
    },
    /// A broken `Read` implementation reported more bytes than its buffer.
    InvalidReadCount {
        /// Supplied buffer capacity.
        maximum: usize,
        /// Count reported by the reader.
        observed: usize,
    },
    /// Checked logical length accounting failed.
    Hash(BlobHashError),
}

impl fmt::Display for BlobReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => write!(formatter, "failed to read blob bytes: {source}"),
            Self::InvalidReadCount { maximum, observed } => write!(
                formatter,
                "reader reported {observed} bytes for a buffer of {maximum} bytes"
            ),
            Self::Hash(source) => source.fmt(formatter),
        }
    }
}

impl Error for BlobReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::Hash(source) => Some(source),
            Self::InvalidReadCount { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlobHashError, BlobHasher};
    use crate::blob::length::BlobLength;

    #[test]
    fn length_overflow_refuses_before_mutating_identity_state() {
        let mut subject = BlobHasher::new();
        subject.logical_length = BlobLength::new(u64::MAX);
        let state_before = subject.state.clone().finalize();

        assert_eq!(
            subject.update(&[0]),
            Err(BlobHashError::LogicalLengthOverflow {
                accumulated: BlobLength::new(u64::MAX),
                incoming: BlobLength::new(1),
            })
        );
        assert_eq!(subject.logical_length, BlobLength::new(u64::MAX));
        assert_eq!(subject.state.finalize(), state_before);
    }
}
