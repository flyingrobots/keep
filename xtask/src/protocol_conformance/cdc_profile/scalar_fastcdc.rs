//! This module owns the independent scalar and streaming `FastCDC` oracle.

use std::mem;

use super::{GearTable, LONG_MASK, MAXIMUM, MINIMUM, SEED, SHORT_MASK, TARGET};
use crate::protocol_conformance::ConformanceError;

pub(super) fn reference_boundaries(
    source: &[u8],
    gear: &GearTable,
) -> Result<Vec<usize>, ConformanceError> {
    let mut ends = Vec::new();
    let mut start = 0_usize;
    while start < source.len() {
        let remaining = source
            .len()
            .checked_sub(start)
            .ok_or_else(|| ConformanceError::violation("CDC remaining length underflow"))?;
        if remaining <= MINIMUM {
            start = start
                .checked_add(remaining)
                .ok_or_else(|| ConformanceError::violation("CDC final boundary overflow"))?;
            ends.push(start);
            continue;
        }
        let limit = remaining.min(MAXIMUM);
        let center = TARGET.min(limit);
        let cut = scan_cut(source, gear, start, center, limit)?;
        start = start
            .checked_add(cut)
            .ok_or_else(|| ConformanceError::violation("CDC boundary overflow"))?;
        ends.push(start);
    }
    Ok(ends)
}

fn scan_cut(
    source: &[u8],
    gear: &GearTable,
    start: usize,
    center: usize,
    limit: usize,
) -> Result<usize, ConformanceError> {
    let mut fingerprint = SEED;
    for position in MINIMUM..center {
        fingerprint = update(fingerprint, source_byte(source, start, position)?, gear)?;
        if fingerprint & SHORT_MASK == 0 {
            return Ok(position);
        }
    }
    for position in center..limit {
        fingerprint = update(fingerprint, source_byte(source, start, position)?, gear)?;
        if fingerprint & LONG_MASK == 0 {
            return Ok(position);
        }
    }
    Ok(limit)
}

fn source_byte(source: &[u8], start: usize, position: usize) -> Result<u8, ConformanceError> {
    let index = start
        .checked_add(position)
        .ok_or_else(|| ConformanceError::violation("CDC probe offset overflow"))?;
    source
        .get(index)
        .copied()
        .ok_or_else(|| ConformanceError::violation("CDC probe is outside its source"))
}

fn update(fingerprint: u64, byte: u8, gear: &GearTable) -> Result<u64, ConformanceError> {
    let value = gear
        .get(usize::from(byte))
        .copied()
        .ok_or_else(|| ConformanceError::violation("Gear byte index is absent"))?;
    Ok(fingerprint.wrapping_shl(1).wrapping_add(value))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StreamingSnapshot {
    current: Vec<u8>,
    completed: Vec<Vec<u8>>,
    fingerprint: u64,
}

pub(super) struct StreamingChunker<'a> {
    gear: &'a GearTable,
    current: Vec<u8>,
    completed: Vec<Vec<u8>>,
    fingerprint: u64,
}

impl<'a> StreamingChunker<'a> {
    pub(super) const fn new(gear: &'a GearTable) -> Self {
        Self {
            gear,
            current: Vec::new(),
            completed: Vec::new(),
            fingerprint: SEED,
        }
    }

    pub(super) fn feed(&mut self, part: &[u8]) -> Result<(), ConformanceError> {
        for value in part {
            self.feed_byte(*value)?;
        }
        Ok(())
    }

    fn feed_byte(&mut self, value: u8) -> Result<(), ConformanceError> {
        let position = self.current.len();
        if position < MINIMUM {
            self.current.push(value);
            return Ok(());
        }
        self.fingerprint = update(self.fingerprint, value, self.gear)?;
        let mask = if position < TARGET {
            SHORT_MASK
        } else {
            LONG_MASK
        };
        if self.fingerprint & mask == 0 {
            self.emit()?;
            self.current.push(value);
            return Ok(());
        }
        self.current.push(value);
        if self.current.len() == MAXIMUM {
            self.emit()?;
        }
        Ok(())
    }

    fn emit(&mut self) -> Result<(), ConformanceError> {
        if self.current.is_empty() {
            return Err(ConformanceError::violation(
                "streaming oracle cannot emit an empty chunk",
            ));
        }
        self.completed.push(mem::take(&mut self.current));
        self.fingerprint = SEED;
        Ok(())
    }

    /// Flushes a non-empty trailing chunk and finalizes result access.
    ///
    /// Repeated calls are idempotent while no additional bytes are fed.
    pub(super) fn finish(&mut self) -> Result<(), ConformanceError> {
        if !self.current.is_empty() {
            self.emit()?;
        }
        Ok(())
    }

    /// Returns cumulative end offsets for the finalized chunks.
    ///
    /// Call [`Self::finish`] first. A non-empty unfinished tail is refused, and
    /// cumulative-length overflow returns [`ConformanceError`]. The returned
    /// vector allocates storage proportional to the completed chunk count.
    pub(super) fn boundaries(&self) -> Result<Vec<usize>, ConformanceError> {
        self.require_finished()?;
        let mut total = 0_usize;
        self.completed
            .iter()
            .map(|chunk| {
                total = total
                    .checked_add(chunk.len())
                    .ok_or_else(|| ConformanceError::violation("streaming boundary overflow"))?;
                Ok(total)
            })
            .collect()
    }

    /// Materializes the exact finalized byte stream.
    ///
    /// Call [`Self::finish`] first. A non-empty unfinished tail is refused.
    /// Success allocates the complete reconstructed source.
    pub(super) fn reconstruct(&self) -> Result<Vec<u8>, ConformanceError> {
        self.require_finished()?;
        Ok(self.completed.concat())
    }

    fn require_finished(&self) -> Result<(), ConformanceError> {
        if self.current.is_empty() {
            Ok(())
        } else {
            Err(ConformanceError::violation(
                "streaming results require a finished stream",
            ))
        }
    }

    pub(super) fn snapshot(&self) -> StreamingSnapshot {
        StreamingSnapshot {
            current: self.current.clone(),
            completed: self.completed.clone(),
            fingerprint: self.fingerprint,
        }
    }
}

pub(super) fn probe_fingerprint(
    source: &[u8],
    position: usize,
    gear: &GearTable,
) -> Result<u64, ConformanceError> {
    let mut fingerprint = SEED;
    let end = position
        .checked_add(1)
        .ok_or_else(|| ConformanceError::violation("probe position overflow"))?;
    for offset in MINIMUM..end {
        let value = source
            .get(offset)
            .copied()
            .ok_or_else(|| ConformanceError::violation("probe position is outside its source"))?;
        fingerprint = update(fingerprint, value, gear)?;
    }
    Ok(fingerprint)
}

#[cfg(test)]
mod tests {
    use super::{ConformanceError, GearTable, MINIMUM, StreamingChunker, reference_boundaries};

    #[test]
    fn matching_probe_byte_starts_the_next_streaming_chunk() {
        let gear: GearTable = [0; 256];
        let source = vec![0; MINIMUM + 2];
        assert!(matches!(
            reference_boundaries(&source, &gear),
            Ok(ref boundaries) if boundaries == &[MINIMUM, MINIMUM + 2]
        ));
        let mut chunker = StreamingChunker::new(&gear);
        let before = chunker.snapshot();
        assert!(chunker.feed(&[]).is_ok());
        assert_eq!(chunker.snapshot(), before);
        assert!(chunker.feed(&source).is_ok());
        assert!(chunker.finish().is_ok());
        assert!(matches!(
            chunker.boundaries(),
            Ok(ref boundaries) if boundaries == &[MINIMUM, MINIMUM + 2]
        ));
        assert!(matches!(
            chunker.reconstruct(),
            Ok(ref reconstructed) if reconstructed == &source
        ));
    }

    #[test]
    fn unfinished_nonempty_stream_refuses_result_access() {
        let gear: GearTable = [0; 256];
        let mut chunker = StreamingChunker::new(&gear);
        assert!(chunker.feed(b"unfinished").is_ok());
        assert!(matches!(
            chunker.boundaries(),
            Err(ConformanceError::Violation(ref message))
                if message == "streaming results require a finished stream"
        ));
        assert!(matches!(
            chunker.reconstruct(),
            Err(ConformanceError::Violation(ref message))
                if message == "streaming results require a finished stream"
        ));
    }
}
