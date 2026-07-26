//! Failure-transition tests for the streaming detector.

use super::{FastCdc, MAXIMUM, SEED};
use crate::{ChunkLength, ChunkOffset, ChunkingError};

#[test]
fn stream_overflow_poisoning_preserves_the_original_failure() {
    let mut subject = FastCdc::new();
    subject.accepted = ChunkOffset::from_validated(u64::MAX);
    let expected = ChunkingError::StreamLengthOverflow {
        accepted: ChunkOffset::from_validated(u64::MAX),
        call_bytes_accepted: 0,
    };

    assert_eq!(subject.feed(&[7], |_span| {}), Err(expected));
    assert_eq!(subject.feed(&[8], |_span| {}), Err(expected));
    assert_eq!(subject.finish(), Err(expected));
}

#[test]
fn candidate_overflow_reports_attempted_length_and_poisoning() {
    let mut subject = FastCdc::new();
    subject.candidate_length = MAXIMUM;
    let attempted = u64::from(MAXIMUM).saturating_add(1);
    let expected = ChunkingError::ChunkLengthOverflow {
        maximum: ChunkLength::from_validated(MAXIMUM),
        attempted,
        call_bytes_accepted: 0,
    };

    assert_eq!(subject.feed(&[7], |_span| {}), Err(expected));
    assert_eq!(subject.accepted, ChunkOffset::ZERO);
    assert_eq!(subject.candidate_length, MAXIMUM);
    assert_eq!(subject.fingerprint, SEED);
    assert_eq!(subject.feed(&[], |_span| {}), Err(expected));
    assert_eq!(subject.finish(), Err(expected));
}

#[test]
fn later_failure_reports_partial_progress_after_an_emission() {
    let mut subject = FastCdc::new();
    let existing_length = MAXIMUM - 1;
    let accepted_before_call = u64::MAX - 1;
    let chunk_start = accepted_before_call - u64::from(existing_length);
    let existing_bytes: Vec<_> = (0..existing_length).map(|_| 0_u8).collect();
    subject.chunk_hasher.update(&existing_bytes);
    subject.chunk_start = ChunkOffset::from_validated(chunk_start);
    subject.accepted = ChunkOffset::from_validated(accepted_before_call);
    subject.candidate_length = existing_length;
    let mut emitted = 0_usize;

    let observed = subject.feed(&[7, 8], |_span| {
        emitted = emitted.saturating_add(1);
    });

    assert_eq!(emitted, 1);
    assert_eq!(
        observed,
        Err(ChunkingError::StreamLengthOverflow {
            accepted: ChunkOffset::from_validated(u64::MAX),
            call_bytes_accepted: 1,
        })
    );
}

#[test]
fn successful_feed_keeps_hash_state_consistent_before_callbacks() -> Result<(), ChunkingError> {
    let mut subject = FastCdc::new();
    let bytes: Vec<_> = (0..MAXIMUM).map(|_| 0_u8).collect();
    let mut callbacks = 0_usize;

    subject.feed(&bytes, |_span| {
        callbacks = callbacks.saturating_add(1);
    })?;

    assert_eq!(callbacks, 1);
    assert!(subject.finish()?.is_none());
    Ok(())
}
