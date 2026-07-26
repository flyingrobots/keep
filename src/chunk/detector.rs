//! Fixed-profile deterministic streaming boundary detection.

use std::mem;

use super::error::ChunkingError;
use super::hasher::ChunkHasher;
use super::{ChunkLength, ChunkOffset, ChunkSpan};

#[path = "detector_feed.rs"]
mod feed;

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
/// caller. Emitted [`ChunkId`](super::ChunkId) values commit to the bytes
/// observed by this detector. They do not compare against an expected identity
/// or prove storage, retention, or membership in a validated layout.
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
/// if let Some(final_span) = detector.finish()? {
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
    failure: Option<ChunkingError>,
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
            failure: None,
        }
    }

    /// Declares EOF and returns the final chunk, including a sub-minimum runt.
    ///
    /// Empty input returns `Ok(None)`. This consumes the detector, allocates no
    /// memory, performs no I/O, and does not persist the returned identity.
    ///
    /// # Errors
    ///
    /// Returns the original error when a prior [`feed`](Self::feed) failed.
    #[must_use = "finish returns the final identified span or the original detector failure"]
    pub fn finish(mut self) -> Result<Option<ChunkSpan>, ChunkingError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        Ok(self.emit_current())
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
#[path = "detector_tests.rs"]
mod tests;
