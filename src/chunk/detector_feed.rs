//! Batched hashing and byte transitions for streaming CDC feeds.

use super::{FastCdc, LONG_MASK, MAXIMUM, MINIMUM, SEED, SHORT_MASK, TARGET};
use crate::chunk::gear_table::GEAR_TABLE;
use crate::{ChunkOffset, ChunkSpan, ChunkingError};

impl FastCdc {
    /// Synchronously incorporates the next source bytes.
    ///
    /// Empty feeds are lawful and are not EOF. For each completed non-final
    /// chunk, `emit` receives its identity and half-open stream span.
    ///
    /// Keep does not allocate or perform I/O. The method invokes `emit`
    /// synchronously before it returns; callback memory, I/O, and failure policy
    /// remain the caller's responsibility.
    ///
    /// If the callback unwinds, discard the detector. Keep cannot observe a
    /// caller panic and therefore cannot record a typed failure transition.
    ///
    /// # Errors
    ///
    /// A failure may occur after a prefix of `bytes` was accepted and callbacks
    /// for that prefix ran. Each variant reports that accepted prefix length.
    /// The detector then becomes failed: later `feed` and
    /// [`finish`](Self::finish) calls return the original error without
    /// accepting or emitting anything. Discard a failed detector; retry is not
    /// meaningful.
    pub fn feed<F>(&mut self, bytes: &[u8], mut emit: F) -> Result<(), ChunkingError>
    where
        F: FnMut(ChunkSpan),
    {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if let Err(error) = self.feed_healthy(bytes, &mut emit) {
            self.failure = Some(error);
            return Err(error);
        }
        Ok(())
    }

    fn feed_healthy<F>(&mut self, bytes: &[u8], emit: &mut F) -> Result<(), ChunkingError>
    where
        F: FnMut(ChunkSpan),
    {
        let mut pending_start = 0_usize;
        for (call_index, byte) in bytes.iter().copied().enumerate() {
            match self.advance_byte(byte, call_index)? {
                ByteTransition::Continue => {}
                ByteTransition::BoundaryBeforeProbe { next_offset } => {
                    self.hash_feed_range(bytes, pending_start, call_index, call_index)?;
                    let span = self.emit_nonempty(call_index)?;
                    let next_index = next_call_index(call_index, bytes.len())?;
                    self.accept_probe(next_offset);
                    self.hash_feed_range(bytes, call_index, next_index, next_index)?;
                    pending_start = next_index;
                    emit(span);
                }
                ByteTransition::BoundaryAfterProbe => {
                    let next_index = next_call_index(call_index, bytes.len())?;
                    self.hash_feed_range(bytes, pending_start, next_index, next_index)?;
                    let span = self.emit_nonempty(next_index)?;
                    pending_start = next_index;
                    emit(span);
                }
            }
        }
        self.hash_feed_range(bytes, pending_start, bytes.len(), bytes.len())
    }

    fn advance_byte(
        &mut self,
        byte: u8,
        call_bytes_accepted: usize,
    ) -> Result<ByteTransition, ChunkingError> {
        let next_offset = self.next_offset(call_bytes_accepted)?;
        let next_length = self.next_candidate_length(call_bytes_accepted)?;
        if self.candidate_length < MINIMUM {
            self.advance_coordinates(next_offset, next_length, self.fingerprint);
            return Ok(ByteTransition::Continue);
        }
        let gear =
            GEAR_TABLE
                .get(usize::from(byte))
                .copied()
                .ok_or(ChunkingError::MissingGearEntry {
                    byte,
                    call_bytes_accepted,
                })?;
        let fingerprint = self.fingerprint.wrapping_shl(1).wrapping_add(gear);
        if fingerprint & self.active_mask() == 0 {
            return Ok(ByteTransition::BoundaryBeforeProbe { next_offset });
        }
        self.advance_coordinates(next_offset, next_length, fingerprint);
        if next_length == MAXIMUM {
            return Ok(ByteTransition::BoundaryAfterProbe);
        }
        Ok(ByteTransition::Continue)
    }

    fn next_offset(&self, call_bytes_accepted: usize) -> Result<ChunkOffset, ChunkingError> {
        self.accepted
            .checked_increment()
            .ok_or(ChunkingError::StreamLengthOverflow {
                accepted: self.accepted,
                call_bytes_accepted,
            })
    }

    fn next_candidate_length(&self, call_bytes_accepted: usize) -> Result<u32, ChunkingError> {
        let attempted = u64::from(self.candidate_length)
            .checked_add(1)
            .ok_or_else(|| overflow(u64::MAX, call_bytes_accepted))?;
        let next =
            u32::try_from(attempted).map_err(|_source| overflow(attempted, call_bytes_accepted))?;
        if next > MAXIMUM {
            return Err(overflow(attempted, call_bytes_accepted));
        }
        Ok(next)
    }

    const fn active_mask(&self) -> u64 {
        if self.candidate_length < TARGET {
            SHORT_MASK
        } else {
            LONG_MASK
        }
    }

    const fn advance_coordinates(
        &mut self,
        next_offset: ChunkOffset,
        next_length: u32,
        fingerprint: u64,
    ) {
        self.accepted = next_offset;
        self.candidate_length = next_length;
        self.fingerprint = fingerprint;
    }

    const fn accept_probe(&mut self, next_offset: ChunkOffset) {
        self.advance_coordinates(next_offset, 1, SEED);
    }

    fn hash_feed_range(
        &mut self,
        bytes: &[u8],
        start: usize,
        end: usize,
        call_bytes_accepted: usize,
    ) -> Result<(), ChunkingError> {
        if start == end {
            return Ok(());
        }
        let range = bytes
            .get(start..end)
            .ok_or(ChunkingError::FeedRangeInvariant {
                start,
                end,
                input_length: bytes.len(),
                call_bytes_accepted,
            })?;
        self.chunk_hasher.update(range);
        Ok(())
    }

    fn emit_nonempty(&mut self, call_bytes_accepted: usize) -> Result<ChunkSpan, ChunkingError> {
        self.emit_current().ok_or(ChunkingError::EmptyBoundary {
            offset: self.accepted,
            call_bytes_accepted,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ByteTransition {
    Continue,
    BoundaryBeforeProbe { next_offset: ChunkOffset },
    BoundaryAfterProbe,
}

fn next_call_index(call_index: usize, input_length: usize) -> Result<usize, ChunkingError> {
    call_index
        .checked_add(1)
        .filter(|next| *next <= input_length)
        .ok_or(ChunkingError::FeedRangeInvariant {
            start: call_index,
            end: usize::MAX,
            input_length,
            call_bytes_accepted: call_index,
        })
}

const fn overflow(attempted: u64, call_bytes_accepted: usize) -> ChunkingError {
    ChunkingError::ChunkLengthOverflow {
        maximum: FastCdc::MAXIMUM_CHUNK_LENGTH,
        attempted,
        call_bytes_accepted,
    }
}
