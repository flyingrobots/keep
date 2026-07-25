#![no_main]

use keep::{ChunkId, ChunkSpan, FastCdc};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let Some(expected) = detect(bytes, &[usize::MAX]) else {
        return;
    };
    let Some(bytewise) = detect(bytes, &[1]) else {
        return;
    };
    assert_eq!(bytewise, expected);

    let widths: Vec<_> = bytes
        .iter()
        .copied()
        .map(|value| usize::from(value).saturating_add(1))
        .collect();
    if !widths.is_empty() {
        let Some(irregular) = detect(bytes, &widths) else {
            return;
        };
        assert_eq!(irregular, expected);
    }
    for span in expected {
        let Ok(start) = usize::try_from(span.offset().get()) else {
            return;
        };
        let Ok(end) = usize::try_from(span.end().get()) else {
            return;
        };
        let Some(chunk) = bytes.get(start..end) else {
            return;
        };
        assert_eq!(ChunkId::hash_bytes(chunk), Ok(span.id()));
    }
});

fn detect(bytes: &[u8], widths: &[usize]) -> Option<Vec<ChunkSpan>> {
    if widths.is_empty() {
        return None;
    }
    let mut detector = FastCdc::new();
    let mut spans = Vec::new();
    let mut remaining = bytes;
    let mut schedule = widths.iter().cycle();
    while !remaining.is_empty() {
        let width = schedule.next().copied()?;
        let count = width.min(remaining.len());
        let (part, next) = remaining.split_at_checked(count)?;
        detector.feed(part, |span| spans.push(span)).ok()?;
        remaining = next;
    }
    if let Some(span) = detector.finish() {
        spans.push(span);
    }
    Some(spans)
}
