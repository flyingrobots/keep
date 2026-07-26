#![no_main]

use keep::{ChunkId, ChunkSpan, ChunkingError, FastCdc};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let expected = require_detection(bytes, &[usize::MAX], "whole");
    let bytewise = require_detection(bytes, &[1], "bytewise");
    assert_eq!(bytewise, expected);

    let widths: Vec<_> = bytes
        .iter()
        .copied()
        .map(|value| usize::from(value).saturating_add(1))
        .collect();
    if !widths.is_empty() {
        let irregular = require_detection(bytes, &widths, "irregular");
        assert_eq!(irregular, expected);
    }
    for span in expected {
        let start = require_usize(span.offset().get(), "span start");
        let end = require_usize(span.end().get(), "span end");
        let chunk = require_span(bytes, start, end);
        assert_eq!(ChunkId::hash_bytes(chunk), Ok(span.id()));
    }
});

fn require_detection(bytes: &[u8], widths: &[usize], schedule: &str) -> Vec<ChunkSpan> {
    let result = detect(bytes, widths);
    assert!(
        result.is_ok(),
        "{schedule} detection refused a finite fuzz input: {:?}",
        result.as_ref().err()
    );
    match result {
        Ok(spans) => spans,
        Err(_error) => std::process::abort(),
    }
}

fn detect(bytes: &[u8], widths: &[usize]) -> Result<Vec<ChunkSpan>, FuzzFailure> {
    if widths.is_empty() || widths.contains(&0) {
        return Err(FuzzFailure::InvalidSchedule);
    }
    let mut detector = FastCdc::new();
    let mut spans = Vec::new();
    let mut remaining = bytes;
    let mut schedule = widths.iter().cycle();
    while !remaining.is_empty() {
        let width = schedule.next().copied().ok_or(FuzzFailure::ScheduleEnded)?;
        let count = width.min(remaining.len());
        let (part, next) = remaining
            .split_at_checked(count)
            .ok_or(FuzzFailure::PartitionEscapedInput)?;
        detector.feed(part, |span| spans.push(span))?;
        remaining = next;
    }
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok(spans)
}

fn require_usize(value: u64, coordinate: &str) -> usize {
    let result = usize::try_from(value);
    assert!(result.is_ok(), "{coordinate} {value} does not fit usize");
    match result {
        Ok(converted) => converted,
        Err(_error) => std::process::abort(),
    }
}

fn require_span(bytes: &[u8], start: usize, end: usize) -> &[u8] {
    let result = bytes.get(start..end);
    assert!(
        result.is_some(),
        "emitted span {start}..{end} escaped {} input bytes",
        bytes.len()
    );
    result.map_or_else(|| -> &[u8] { std::process::abort() }, |span| span)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FuzzFailure {
    InvalidSchedule,
    ScheduleEnded,
    PartitionEscapedInput,
    Chunking(ChunkingError),
}

impl From<ChunkingError> for FuzzFailure {
    fn from(source: ChunkingError) -> Self {
        Self::Chunking(source)
    }
}
