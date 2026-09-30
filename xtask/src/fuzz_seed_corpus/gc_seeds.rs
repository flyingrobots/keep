//! This module owns canonical GC retirement record fuzz seeds.

use super::filesystem::RepositoryFiles;
use super::segment_store_v2_fixture;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed, prefixed};

const GC_INTENT_FIXTURE: &str = "one-candidate-gc-intent.hex";
const GC_RECEIPT_FIXTURE: &str = "one-candidate-gc-receipt.hex";

pub(super) const FIXTURES: [(u8, &str); 2] = [(0, GC_INTENT_FIXTURE), (1, GC_RECEIPT_FIXTURE)];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let [
        (intent_selector, intent_fixture),
        (receipt_selector, receipt_fixture),
    ] = FIXTURES;
    let intent = segment_store_v2_fixture::read_hex(files, intent_fixture)?;
    let receipt = segment_store_v2_fixture::read_hex(files, receipt_fixture)?;
    Ok(vec![
        Seed::new(
            "gc_format",
            "one-candidate-gc-intent",
            prefixed(intent_selector, &intent)?,
        )?,
        Seed::new(
            "gc_format",
            "one-candidate-gc-receipt",
            receipt_seed(receipt_selector, &intent, &receipt)?,
        )?,
    ])
}

/// Frames the receipt seed as selector, big-endian `u32` intent length, the
/// exact intent, then the receipt, matching the target's decoding order.
fn receipt_seed(selector: u8, intent: &[u8], receipt: &[u8]) -> Result<Vec<u8>, FuzzSeedError> {
    let intent_length = u32::try_from(intent.len())
        .map_err(|_| FuzzSeedError::violation("GC intent seed exceeds the u32 length frame"))?;
    let payload_bytes = intent
        .len()
        .checked_add(4)
        .and_then(|length| length.checked_add(receipt.len()))
        .ok_or_else(|| FuzzSeedError::violation("GC receipt seed length overflow"))?;
    let framed_bytes = payload_bytes
        .checked_add(1)
        .ok_or_else(|| FuzzSeedError::violation("GC receipt seed length overflow"))?;
    if framed_bytes > MAX_SEED_BYTES {
        return Err(FuzzSeedError::violation(
            "GC receipt seed exceeds the input bound",
        ));
    }
    let mut payload = Vec::with_capacity(payload_bytes);
    payload.extend_from_slice(&intent_length.to_be_bytes());
    payload.extend_from_slice(intent);
    payload.extend_from_slice(receipt);
    prefixed(selector, &payload)
}

#[cfg(test)]
mod tests {
    use super::{FuzzSeedError, MAX_SEED_BYTES, receipt_seed};

    #[test]
    fn receipt_seed_frames_the_intent_length_before_both_records() -> Result<(), FuzzSeedError> {
        let seed = receipt_seed(1, b"intent", b"receipt")?;
        assert_eq!(seed, b"\x01\x00\x00\x00\x06intentreceipt");
        Ok(())
    }

    #[test]
    fn receipt_seed_refuses_before_allocating_above_the_seed_bound() -> Result<(), FuzzSeedError> {
        let oversized_intent = vec![0; MAX_SEED_BYTES];
        let Err(FuzzSeedError::Violation(message)) = receipt_seed(1, &oversized_intent, &[]) else {
            return Err(FuzzSeedError::violation(
                "oversized GC receipt seed was admitted",
            ));
        };
        assert_eq!(message, "GC receipt seed exceeds the input bound");
        Ok(())
    }
}
