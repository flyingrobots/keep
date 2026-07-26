//! Public-path detector scheduling and span checks.

use std::collections::BTreeSet;

use keep::{ChunkId, ChunkSpan, FastCdc};

use super::harness_failure::HarnessFailure;

pub(super) fn detect(bytes: &[u8], widths: &[usize]) -> Result<Vec<ChunkSpan>, HarnessFailure> {
    if widths.is_empty() || widths.contains(&0) {
        return Err(HarnessFailure::corpus("partition plan is invalid"));
    }
    let mut spans = Vec::new();
    let mut remaining = bytes;
    let mut schedule = widths.iter().cycle();
    let mut detector = FastCdc::new();
    detector.feed(&[], |span| spans.push(span))?;
    while !remaining.is_empty() {
        let width = schedule
            .next()
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("partition plan ended"))?;
        let count = width.min(remaining.len());
        let Some((part, next)) = remaining.split_at_checked(count) else {
            return Err(HarnessFailure::corpus("partition split escaped input"));
        };
        detector.feed(part, |span| spans.push(span))?;
        detector.feed(&[], |span| spans.push(span))?;
        remaining = next;
    }
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok(spans)
}

pub(super) fn boundary_adjacent_widths(
    logical_length: usize,
    boundaries: &[u64],
) -> Result<Vec<usize>, HarnessFailure> {
    let mut points = BTreeSet::from([0_usize, logical_length]);
    for boundary in boundaries {
        let coordinate = usize::try_from(*boundary)
            .map_err(|_source| HarnessFailure::corpus("boundary does not fit usize"))?;
        if coordinate > logical_length {
            return Err(HarnessFailure::corpus("boundary exceeds source length"));
        }
        if let Some(before) = coordinate.checked_sub(1) {
            points.insert(before);
        }
        points.insert(coordinate);
        if let Some(after) = coordinate.checked_add(1)
            && after <= logical_length
        {
            points.insert(after);
        }
    }
    let ordered: Vec<_> = points.into_iter().collect();
    let mut widths = Vec::new();
    for pair in ordered.windows(2) {
        let left = pair
            .first()
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("adjacent point pair is empty"))?;
        let right = pair
            .get(1)
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("adjacent point pair is truncated"))?;
        let width = right
            .checked_sub(left)
            .ok_or_else(|| HarnessFailure::corpus("adjacent points are reversed"))?;
        if width != 0 {
            widths.push(width);
        }
    }
    if logical_length != 0 && widths.is_empty() {
        return Err(HarnessFailure::corpus("boundary-adjacent plan is empty"));
    }
    Ok(widths)
}

pub(super) fn assert_spans_name_exact_bytes(
    bytes: &[u8],
    spans: &[ChunkSpan],
) -> Result<(), HarnessFailure> {
    let mut reconstructed = Vec::with_capacity(bytes.len());
    let mut previous_end = 0_usize;
    for span in spans {
        let start = usize::try_from(span.offset().get())
            .map_err(|_source| HarnessFailure::corpus("span start does not fit usize"))?;
        let end = usize::try_from(span.end().get())
            .map_err(|_source| HarnessFailure::corpus("span end does not fit usize"))?;
        if start != previous_end {
            return Err(HarnessFailure::corpus("chunk spans are not contiguous"));
        }
        let chunk = bytes
            .get(start..end)
            .ok_or_else(|| HarnessFailure::corpus("chunk span escaped input"))?;
        assert_eq!(ChunkId::hash_bytes(chunk)?, span.id());
        assert_eq!(
            usize::try_from(span.length().get())
                .map_err(|_source| HarnessFailure::corpus("chunk length does not fit usize"))?,
            chunk.len()
        );
        reconstructed.extend_from_slice(chunk);
        previous_end = end;
    }
    assert_eq!(previous_end, bytes.len());
    assert_eq!(reconstructed, bytes);
    Ok(())
}
