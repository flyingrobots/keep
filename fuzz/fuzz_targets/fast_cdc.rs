#![no_main]

//! This target owns streaming `FastCdc` boundary and identity fuzzing.

use keep::{ChunkSpan, ChunkingError, FastCdc};
use keep_fuzz::validate_spans;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let expected = require_detection(bytes, &[usize::MAX], "whole");
    let bytewise = require_detection(bytes, &[1], "bytewise");
    assert_eq!(bytewise, expected);

    let widths: Vec<_> = bytes.iter().copied().map(schedule_width).collect();
    if !widths.is_empty() {
        let irregular = require_detection(bytes, &widths, "irregular");
        assert_eq!(irregular, expected);
    }
    require_valid_coverage(bytes, &expected);
});

fn schedule_width(value: u8) -> usize {
    usize::from(value)
        .checked_add(1)
        .unwrap_or_else(abort_schedule_width_overflow)
}

fn abort_schedule_width_overflow() -> usize {
    std::process::abort();
}

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

fn require_valid_coverage(bytes: &[u8], spans: &[ChunkSpan]) {
    let result = validate_spans(bytes, spans);
    assert!(
        result.is_ok(),
        "emitted spans violated exact input coverage: {:?}",
        result.as_ref().err()
    );
    if result.is_err() {
        std::process::abort();
    }
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
