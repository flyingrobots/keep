//! Isolated heap-allocation evidence for streaming CDC.

use allocation_counter::{AllocationInfo, measure};
use keep::{ChunkingError, FastCdc};

const MINIMUM_INPUT_BYTES: usize = 16_384;
const REPRESENTATIVE_INPUT_BYTES: usize = 1_048_576;
const LARGE_INPUT_BYTES: usize = 4_194_304;

#[test]
fn detector_peak_heap_allocation_is_independent_of_input_length() -> Result<(), ChunkingError> {
    for length in [
        MINIMUM_INPUT_BYTES,
        REPRESENTATIVE_INPUT_BYTES,
        LARGE_INPUT_BYTES,
    ] {
        let bytes = deterministic_bytes(length);
        let mut emitted = 0_usize;
        let mut feed_result = Ok(());
        let mut finish_result = Ok(None);

        let observed = measure(|| {
            let mut detector = FastCdc::new();
            feed_result = detector.feed(&bytes, |_span| {
                emitted = emitted.saturating_add(1);
            });
            finish_result = detector.finish();
        });

        feed_result?;
        let final_span = finish_result?;
        assert!(emitted > 0 || final_span.is_some());
        assert_eq!(
            observed,
            AllocationInfo::default(),
            "detector allocated while processing {length} bytes"
        );
    }
    Ok(())
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
