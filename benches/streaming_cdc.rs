//! Streaming CDC throughput regression benchmarks.

use divan::counter::BytesCount;
use divan::{Bencher, black_box};
use keep::{ChunkSpan, FastCdc};

const MINIMUM_INPUT_BYTES: usize = 16_384;
const REPRESENTATIVE_INPUT_BYTES: usize = 1_048_576;

fn main() {
    divan::main();
}

#[divan::bench(args = [MINIMUM_INPUT_BYTES, REPRESENTATIVE_INPUT_BYTES])]
fn whole_feed(bencher: Bencher<'_, '_>, input_length: usize) {
    let bytes = deterministic_bytes(input_length);
    bencher
        .counter(BytesCount::new(input_length))
        .bench_local(move || run_whole_feed(black_box(&bytes)));
}

#[divan::bench(args = [MINIMUM_INPUT_BYTES, REPRESENTATIVE_INPUT_BYTES])]
fn bytewise_feed(bencher: Bencher<'_, '_>, input_length: usize) {
    let bytes = deterministic_bytes(input_length);
    bencher
        .counter(BytesCount::new(input_length))
        .bench_local(move || run_bytewise_feed(black_box(&bytes)));
}

fn run_whole_feed(bytes: &[u8]) {
    let mut detector = FastCdc::new();
    let result = detector.feed(black_box(bytes), black_box(consume_span));
    assert!(result.is_ok(), "whole-feed benchmark input was refused");
    let finished = detector.finish();
    assert!(finished.is_ok(), "whole-feed benchmark finish was refused");
    let _ = black_box(finished);
}

fn run_bytewise_feed(bytes: &[u8]) {
    let mut detector = FastCdc::new();
    for byte in bytes {
        let result = detector.feed(std::slice::from_ref(byte), black_box(consume_span));
        assert!(result.is_ok(), "bytewise benchmark input was refused");
    }
    let finished = detector.finish();
    assert!(finished.is_ok(), "bytewise benchmark finish was refused");
    let _ = black_box(finished);
}

const fn consume_span(span: ChunkSpan) {
    black_box(span);
}

fn deterministic_bytes(length: usize) -> Vec<u8> {
    let mut state = 0x0123_4567_89ab_cdef_u64;
    let mut bytes = Vec::with_capacity(length);
    for _ in 0..length {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        let [byte, ..] = state.to_le_bytes();
        bytes.push(byte);
    }
    bytes
}
