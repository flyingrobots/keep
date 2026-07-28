//! This module owns deterministic CDC partition-schedule cycling.

use crate::protocol_conformance::ConformanceError;
use crate::protocol_conformance::cdc_profile::scalar_fastcdc::StreamingChunker;

pub(super) fn feed_sizes(
    chunker: &mut StreamingChunker<'_>,
    source: &[u8],
    sizes: &[usize],
) -> Result<(), ConformanceError> {
    feed_sizes_with(chunker, source, sizes, "partition schedule", |_| Ok(()))
}

pub(super) fn feed_sizes_with(
    chunker: &mut StreamingChunker<'_>,
    source: &[u8],
    sizes: &[usize],
    schedule: &str,
    mut before_partition: impl FnMut(&mut StreamingChunker<'_>) -> Result<(), ConformanceError>,
) -> Result<(), ConformanceError> {
    if sizes.is_empty() || sizes.contains(&0) {
        return Err(ConformanceError::violation(format!(
            "{schedule} must contain positive sizes"
        )));
    }
    let mut offset = 0_usize;
    let mut index = 0_usize;
    while offset < source.len() {
        let size = *sizes
            .get(index)
            .ok_or_else(|| ConformanceError::violation(format!("{schedule} size is absent")))?;
        let end = partition_end(offset, size, source.len(), schedule)?;
        before_partition(chunker)?;
        chunker.feed(source.get(offset..end).ok_or_else(|| {
            ConformanceError::violation(format!("{schedule} moved outside its source"))
        })?)?;
        offset = end;
        let next = index
            .checked_add(1)
            .ok_or_else(|| ConformanceError::violation(format!("{schedule} overflow")))?;
        index = if next == sizes.len() { 0 } else { next };
    }
    Ok(())
}

pub(super) fn partition_end(
    offset: usize,
    size: usize,
    source_length: usize,
    schedule: &str,
) -> Result<usize, ConformanceError> {
    offset
        .checked_add(size)
        .map(|end| end.min(source_length))
        .ok_or_else(|| ConformanceError::violation(format!("{schedule} offset overflow")))
}
