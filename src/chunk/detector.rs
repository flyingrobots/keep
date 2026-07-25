//! Fixed-profile deterministic streaming boundary detection.

use std::mem;

use super::error::ChunkingError;
use super::gear_table::GEAR_TABLE;
use super::hasher::ChunkHasher;
use super::{ChunkLength, ChunkOffset, ChunkSpan};

const MINIMUM: u32 = 16_384;
const TARGET: u32 = 65_536;
const MAXIMUM: u32 = 262_144;
const SEED: u64 = 0;
const SHORT_MASK: u64 = 0x0000_d907_0753_7000;
const LONG_MASK: u64 = 0x0000_d903_1353_0000;

/// Streaming detector for the registered `fastcdc-64k-v1` profile.
///
/// `FastCdc` borrows each feed slice and emits identified spans through a
/// caller callback. Keep retains no candidate bytes and performs no heap
/// allocation, I/O, or blocking wait. The fixed state is independent of total
/// stream length and remains below [`RETAINED_STATE_LIMIT_BYTES`](Self::RETAINED_STATE_LIMIT_BYTES).
///
/// The callback may allocate or perform I/O; those effects belong to the
/// caller. Emitted [`ChunkId`](super::ChunkId) values verify the bytes observed
/// by this detector, but do not prove storage, retention, or membership in a
/// validated layout.
///
/// Call [`finish`](Self::finish) exactly once to declare EOF and emit a final
/// runt. Dropping the detector does not imply EOF or durability.
///
/// # Examples
///
/// ```
/// use keep::{ChunkSpan, FastCdc};
///
/// let bytes = b"example bytes";
/// let mut spans = Vec::<ChunkSpan>::new();
/// let mut detector = FastCdc::new();
/// detector.feed(bytes, |span| spans.push(span))?;
/// if let Some(final_span) = detector.finish() {
///     spans.push(final_span);
/// }
/// assert_eq!(spans.len(), 1);
/// let span = spans.first().ok_or("missing final chunk")?;
/// assert_eq!(span.length().get(), 13);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use = "a FastCdc detector has no complete result until finish is called"]
pub struct FastCdc {
    chunk_hasher: ChunkHasher,
    chunk_start: ChunkOffset,
    accepted: ChunkOffset,
    candidate_length: u32,
    fingerprint: u64,
}

impl FastCdc {
    /// Exact minimum non-final chunk length for `fastcdc-64k-v1`.
    pub const MINIMUM_CHUNK_LENGTH: ChunkLength = ChunkLength::from_validated(MINIMUM);
    /// Exact target transition coordinate for `fastcdc-64k-v1`.
    pub const TARGET_CHUNK_LENGTH: ChunkLength = ChunkLength::from_validated(TARGET);
    /// Exact hard maximum chunk length for `fastcdc-64k-v1`.
    pub const MAXIMUM_CHUNK_LENGTH: ChunkLength = ChunkLength::from_validated(MAXIMUM);
    /// Maximum retained detector state, excluding caller-owned input and sink.
    pub const RETAINED_STATE_LIMIT_BYTES: usize = 4_096;

    /// Starts a detector at stream offset zero.
    pub fn new() -> Self {
        Self {
            chunk_hasher: ChunkHasher::new(),
            chunk_start: ChunkOffset::ZERO,
            accepted: ChunkOffset::ZERO,
            candidate_length: 0,
            fingerprint: SEED,
        }
    }

    /// Synchronously incorporates the next source bytes.
    ///
    /// Empty feeds are lawful and are not EOF. Every nonempty byte is accepted
    /// exactly once unless this method returns an error. For each completed
    /// non-final chunk, `emit` receives its identity and half-open stream span.
    ///
    /// Keep does not allocate or perform I/O. The method invokes `emit`
    /// synchronously before it returns; callback memory, I/O, and failure policy
    /// remain the caller's responsibility.
    ///
    /// # Errors
    ///
    /// Returns [`ChunkingError`] before accepting the byte that would overflow
    /// a typed coordinate or expose an impossible compiled-profile invariant.
    pub fn feed<F>(&mut self, bytes: &[u8], mut emit: F) -> Result<(), ChunkingError>
    where
        F: FnMut(ChunkSpan),
    {
        for byte in bytes.iter().copied() {
            if let Some(span) = self.accept_byte(byte)? {
                emit(span);
            }
        }
        Ok(())
    }

    /// Declares EOF and returns the final chunk, including a sub-minimum runt.
    ///
    /// Empty input returns `None`. This consumes the detector, allocates no
    /// memory, performs no I/O, and does not persist the returned identity.
    #[must_use]
    pub fn finish(mut self) -> Option<ChunkSpan> {
        self.emit_current()
    }

    fn accept_byte(&mut self, byte: u8) -> Result<Option<ChunkSpan>, ChunkingError> {
        let next_offset =
            self.accepted
                .checked_increment()
                .ok_or(ChunkingError::StreamLengthOverflow {
                    accepted: self.accepted,
                })?;
        let next_length =
            self.candidate_length
                .checked_add(1)
                .ok_or(ChunkingError::ChunkLengthOverflow {
                    maximum: Self::MAXIMUM_CHUNK_LENGTH,
                })?;
        if next_length > MAXIMUM {
            return Err(ChunkingError::ChunkLengthOverflow {
                maximum: Self::MAXIMUM_CHUNK_LENGTH,
            });
        }
        if self.candidate_length < MINIMUM {
            self.incorporate(byte, next_offset, next_length, self.fingerprint);
            return Ok(None);
        }
        let gear = GEAR_TABLE
            .get(usize::from(byte))
            .copied()
            .ok_or(ChunkingError::MissingGearEntry { byte })?;
        let fingerprint = self.fingerprint.wrapping_shl(1).wrapping_add(gear);
        let mask = if self.candidate_length < TARGET {
            SHORT_MASK
        } else {
            LONG_MASK
        };
        if fingerprint & mask == 0 {
            let span = self.emit_nonempty()?;
            self.incorporate(byte, next_offset, 1, SEED);
            return Ok(Some(span));
        }
        self.incorporate(byte, next_offset, next_length, fingerprint);
        if next_length == MAXIMUM {
            return self.emit_nonempty().map(Some);
        }
        Ok(None)
    }

    fn incorporate(
        &mut self,
        byte: u8,
        next_offset: ChunkOffset,
        next_length: u32,
        fingerprint: u64,
    ) {
        self.chunk_hasher.update(&[byte]);
        self.accepted = next_offset;
        self.candidate_length = next_length;
        self.fingerprint = fingerprint;
    }

    fn emit_nonempty(&mut self) -> Result<ChunkSpan, ChunkingError> {
        self.emit_current().ok_or(ChunkingError::EmptyBoundary {
            offset: self.accepted,
        })
    }

    fn emit_current(&mut self) -> Option<ChunkSpan> {
        let length = ChunkLength::new(self.candidate_length)?;
        let hasher = mem::replace(&mut self.chunk_hasher, ChunkHasher::new());
        let id = hasher.finish(length);
        let span = ChunkSpan::new(self.chunk_start, self.accepted, id);
        self.chunk_start = self.accepted;
        self.candidate_length = 0;
        self.fingerprint = SEED;
        Some(span)
    }
}

impl Default for FastCdc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{FastCdc, MAXIMUM, SEED};
    use crate::{ChunkLength, ChunkOffset, ChunkingError};

    #[test]
    fn stream_overflow_refuses_before_mutating_detector_state() {
        let mut subject = FastCdc::new();
        subject.accepted = ChunkOffset::from_validated(u64::MAX);

        assert_eq!(
            subject.feed(&[7], |_span| {}),
            Err(ChunkingError::StreamLengthOverflow {
                accepted: ChunkOffset::from_validated(u64::MAX),
            })
        );
        assert_eq!(subject.accepted.get(), u64::MAX);
        assert_eq!(subject.candidate_length, 0);
        assert_eq!(subject.fingerprint, SEED);
    }

    #[test]
    fn candidate_overflow_refuses_before_mutating_detector_state() {
        let mut subject = FastCdc::new();
        subject.candidate_length = MAXIMUM;

        assert_eq!(
            subject.feed(&[7], |_span| {}),
            Err(ChunkingError::ChunkLengthOverflow {
                maximum: ChunkLength::from_validated(MAXIMUM),
            })
        );
        assert_eq!(subject.accepted, ChunkOffset::ZERO);
        assert_eq!(subject.candidate_length, MAXIMUM);
        assert_eq!(subject.fingerprint, SEED);
    }
}
