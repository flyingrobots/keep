//! Public-path conformance tests for deterministic streaming CDC.

#[path = "boundary_corpus.rs"]
mod boundary_corpus;
#[path = "detector_support.rs"]
mod detector_support;
#[path = "harness_failure.rs"]
mod harness_failure;
#[path = "mutation_corpus.rs"]
mod mutation_corpus;
#[path = "source_corpus.rs"]
mod source_corpus;

use std::collections::BTreeMap;
use std::mem::{size_of, size_of_val};

use boundary_corpus::expected_boundaries;
use detector_support::{assert_spans_name_exact_bytes, boundary_adjacent_widths, detect};
use harness_failure::HarnessFailure;
use keep::{ChunkHashError, ChunkSpan, FastCdc};
use mutation_corpus::add_mutations;
use source_corpus::primitive_sources;

type TestResult = Result<(), HarnessFailure>;

#[test]
fn every_golden_source_has_exact_boundaries_and_chunk_identities() -> TestResult {
    let sources = all_sources()?;
    let expected = expected_boundaries()?;
    if sources.keys().ne(expected.keys()) {
        return Err(HarnessFailure::corpus(
            "source and boundary case sets differ",
        ));
    }
    for (name, bytes) in &sources {
        let spans = detect(bytes, &[usize::MAX])?;
        let observed: Vec<_> = spans.iter().map(|span| span.end().get()).collect();
        assert_eq!(
            &observed,
            expected
                .get(name)
                .ok_or_else(|| HarnessFailure::corpus("expected boundaries are absent"))?,
            "boundaries moved for {name}"
        );
        assert_profile_bounds(bytes, &spans)?;
        assert_spans_name_exact_bytes(bytes, &spans)?;
    }
    Ok(())
}

#[test]
fn feed_partitioning_cannot_move_boundaries_or_identities() -> TestResult {
    let sources = all_sources()?;
    let expected_boundaries = expected_boundaries()?;
    let plans: [&[usize]; 3] = [
        &[8_192],
        &[1, 7, 64, 4_093, 65_536],
        &[262_143, 2, 16_384, 3],
    ];
    for (name, bytes) in &sources {
        let expected = detect(bytes, &[usize::MAX])?;
        for plan in plans {
            assert_eq!(
                detect(bytes, plan)?,
                expected,
                "partition plan {plan:?} moved {name}"
            );
        }
        if !bytes.is_empty() {
            let boundaries = expected_boundaries
                .get(name)
                .ok_or_else(|| HarnessFailure::corpus("boundary case is absent"))?;
            let adjacent = boundary_adjacent_widths(bytes.len(), boundaries)?;
            assert_eq!(
                detect(bytes, &adjacent)?,
                expected,
                "boundary-adjacent partitions moved {name}"
            );
        }
    }
    Ok(())
}

#[test]
fn a_long_one_byte_feed_schedule_preserves_probe_carry() -> TestResult {
    let sources = all_sources()?;
    for name in ["probe-byte-carry", "target-long-transition"] {
        let bytes = sources
            .get(name)
            .ok_or_else(|| HarnessFailure::corpus("one-byte witness is absent"))?;
        assert_eq!(
            detect(bytes, &[1])?,
            detect(bytes, &[usize::MAX])?,
            "one-byte feeds moved {name}"
        );
    }
    Ok(())
}

#[test]
fn generated_properties_preserve_partition_reconstruction_and_bounds() -> TestResult {
    let lengths = [
        0, 1, 13, 16_383, 16_384, 16_385, 65_535, 65_536, 65_537, 262_143, 262_144, 262_145,
        600_000,
    ];
    let seeds = [1_u64, 0x9e37_79b9_7f4a_7c15, u64::MAX];
    let plans: [&[usize]; 4] = [
        &[1],
        &[3, 5, 11],
        &[16_383, 2, 65_535],
        &[7, 257, 8_191, 262_144],
    ];
    for seed in seeds {
        for length in lengths {
            let bytes = generated_bytes(seed, length)?;
            let expected = detect(&bytes, &[usize::MAX])?;
            assert_profile_bounds(&bytes, &expected)?;
            assert_spans_name_exact_bytes(&bytes, &expected)?;
            assert_eq!(detect(&bytes, &[usize::MAX])?, expected);
            for plan in plans {
                assert_eq!(
                    detect(&bytes, plan)?,
                    expected,
                    "property failed for seed {seed:#x}, length {length}, plan {plan:?}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn retained_working_state_is_constant_and_bounded() -> TestResult {
    let sources = all_sources()?;
    let bytes = sources
        .get("zeros-long")
        .ok_or_else(|| HarnessFailure::corpus("memory witness is absent"))?;
    let mut detector = FastCdc::new();
    let before = size_of_val(&detector);
    let mut emitted = 0_usize;
    let mut counter_overflowed = false;
    detector.feed(bytes, |_span| match emitted.checked_add(1) {
        Some(next) => emitted = next,
        None => counter_overflowed = true,
    })?;
    let after = size_of_val(&detector);
    assert!(!counter_overflowed);
    assert_eq!(before, after);
    assert_eq!(before, size_of::<FastCdc>());
    assert!(before <= FastCdc::RETAINED_STATE_LIMIT_BYTES);
    assert_eq!(emitted, 4);
    assert!(detector.finish().is_none());
    Ok(())
}

#[test]
fn chunk_identity_refuses_the_only_invalid_length_edge() {
    assert_eq!(keep::ChunkId::hash_bytes(&[]), Err(ChunkHashError::Empty));
}

fn all_sources() -> Result<BTreeMap<&'static str, Vec<u8>>, HarnessFailure> {
    let mut sources = primitive_sources()?;
    add_mutations(&mut sources)?;
    Ok(sources)
}

fn assert_profile_bounds(bytes: &[u8], spans: &[ChunkSpan]) -> TestResult {
    if bytes.is_empty() {
        assert!(spans.is_empty());
        return Ok(());
    }
    let (final_span, non_final) = spans
        .split_last()
        .ok_or_else(|| HarnessFailure::corpus("nonempty input emitted no chunks"))?;
    for span in non_final {
        assert!(span.length() >= FastCdc::MINIMUM_CHUNK_LENGTH);
        assert!(span.length() <= FastCdc::MAXIMUM_CHUNK_LENGTH);
    }
    assert!(final_span.length().get() > 0);
    assert!(final_span.length() <= FastCdc::MAXIMUM_CHUNK_LENGTH);
    Ok(())
}

fn generated_bytes(mut state: u64, length: usize) -> Result<Vec<u8>, HarnessFailure> {
    let mut bytes = Vec::with_capacity(length);
    for _ in 0..length {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        bytes.push(
            state
                .to_le_bytes()
                .first()
                .copied()
                .ok_or_else(|| HarnessFailure::corpus("generated byte is absent"))?,
        );
    }
    Ok(bytes)
}
