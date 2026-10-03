//! This module owns canonical store-migration record fuzz seeds.

use super::filesystem::RepositoryFiles;
use super::segment_store_v2_fixture;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed, prefixed};

const FORMAT_MARKER_FIXTURE: &str = "format-marker.hex";
const MIGRATION_INTENT_FIXTURE: &str = "migration-intent.hex";
const MIGRATION_RECEIPT_FIXTURE: &str = "migration-receipt.hex";

pub(super) const FIXTURES: [(u8, &str); 3] = [
    (0, FORMAT_MARKER_FIXTURE),
    (1, MIGRATION_INTENT_FIXTURE),
    (2, MIGRATION_RECEIPT_FIXTURE),
];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let [
        (marker_selector, marker_fixture),
        (intent_selector, intent_fixture),
        (receipt_selector, receipt_fixture),
    ] = FIXTURES;
    let marker = segment_store_v2_fixture::read_hex(files, marker_fixture)?;
    let intent = segment_store_v2_fixture::read_hex(files, intent_fixture)?;
    let receipt = segment_store_v2_fixture::read_hex(files, receipt_fixture)?;
    let mut seeds = vec![
        Seed::new(
            "migration_format",
            "format-marker",
            prefixed(marker_selector, &marker)?,
        )?,
        Seed::new(
            "migration_format",
            "migration-intent",
            prefixed(intent_selector, &intent)?,
        )?,
        Seed::new(
            "migration_format",
            "migration-receipt",
            receipt_seed(receipt_selector, &marker, &intent, &receipt)?,
        )?,
    ];
    seeds.extend(malformed_seeds(&marker, &intent, &receipt)?);
    seeds.extend(super::migration_recovery_seeds::seeds(
        &marker, &intent, &receipt,
    )?);
    Ok(seeds)
}

fn receipt_seed(
    selector: u8,
    marker: &[u8],
    intent: &[u8],
    receipt: &[u8],
) -> Result<Vec<u8>, FuzzSeedError> {
    let payload_bytes = marker
        .len()
        .checked_add(intent.len())
        .and_then(|length| length.checked_add(receipt.len()))
        .ok_or_else(|| FuzzSeedError::violation("migration receipt seed length overflow"))?;
    let framed_bytes = payload_bytes
        .checked_add(1)
        .ok_or_else(|| FuzzSeedError::violation("migration receipt seed length overflow"))?;
    if framed_bytes > MAX_SEED_BYTES {
        return Err(FuzzSeedError::violation(
            "migration receipt seed exceeds the input bound",
        ));
    }
    let mut payload = Vec::with_capacity(payload_bytes);
    payload.extend_from_slice(marker);
    payload.extend_from_slice(intent);
    payload.extend_from_slice(receipt);
    prefixed(selector, &payload)
}

fn malformed_seeds(
    marker: &[u8],
    intent: &[u8],
    receipt: &[u8],
) -> Result<Vec<Seed>, FuzzSeedError> {
    let mut bad_marker = marker.to_vec();
    *bad_marker
        .get_mut(17)
        .ok_or_else(|| FuzzSeedError::violation("marker lacks version"))? = 3;
    let mut bad_intent = intent.to_vec();
    *bad_intent
        .get_mut(23)
        .ok_or_else(|| FuzzSeedError::violation("intent lacks flags"))? = 1;
    let mut bad_receipt = receipt.to_vec();
    *bad_receipt
        .last_mut()
        .ok_or_else(|| FuzzSeedError::violation("receipt lacks checksum"))? ^= 1;
    Ok(vec![
        Seed::new(
            "migration_format",
            "unsupported-marker-version",
            prefixed(0, &bad_marker)?,
        )?,
        Seed::new(
            "migration_format",
            "unsupported-intent-flag",
            prefixed(1, &bad_intent)?,
        )?,
        Seed::new(
            "migration_format",
            "corrupt-receipt-checksum",
            receipt_seed(2, marker, intent, &bad_receipt)?,
        )?,
    ])
}

#[cfg(test)]
mod tests {
    use super::{FuzzSeedError, MAX_SEED_BYTES, receipt_seed};

    #[test]
    fn receipt_seed_frames_dependencies_before_the_receipt() -> Result<(), FuzzSeedError> {
        let seed = receipt_seed(2, b"marker", b"intent", b"receipt")?;
        assert_eq!(seed, b"\x02markerintentreceipt");
        Ok(())
    }

    #[test]
    fn receipt_seed_refuses_before_allocating_above_the_seed_bound() -> Result<(), FuzzSeedError> {
        let oversized_marker = vec![0; MAX_SEED_BYTES];
        let Err(FuzzSeedError::Violation(message)) = receipt_seed(2, &oversized_marker, &[], &[])
        else {
            return Err(FuzzSeedError::violation(
                "oversized migration receipt seed was admitted",
            ));
        };
        assert_eq!(message, "migration receipt seed exceeds the input bound");
        Ok(())
    }
}
