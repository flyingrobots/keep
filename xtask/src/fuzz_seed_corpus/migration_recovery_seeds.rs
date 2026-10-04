//! Migration recovery seeds encode valid prefixes and contradictory transition evidence.

use super::{FuzzSeedError, Seed, prefixed};

pub(super) fn seeds(
    marker: &[u8],
    intent: &[u8],
    receipt: &[u8],
) -> Result<Vec<Seed>, FuzzSeedError> {
    let records = [intent, intent, marker, marker, receipt, receipt];
    let mut seeds = Vec::new();
    for (name, presence, namespace) in [
        ("recovery-version-one", 0, 0),
        ("recovery-intent-stage", 1, 0),
        ("recovery-durable-intent", 2, 0),
        ("recovery-partial-namespace", 2, 7),
        ("recovery-full-namespace", 2, 127),
        ("recovery-marker-stage", 6, 127),
        ("recovery-marker", 10, 127),
        ("recovery-receipt-stage", 26, 127),
        ("recovery-complete", 42, 127),
        ("recovery-effect-before-intent", 0, 1),
        ("recovery-stage-after-effect", 3, 1),
        ("recovery-namespace-hole", 2, 5),
        ("recovery-receipt-before-marker", 18, 127),
    ] {
        seeds.push(seed(name, presence, namespace, intent, records)?);
    }
    let mut corrupt = intent.to_vec();
    let checksum = corrupt
        .last_mut()
        .ok_or_else(|| FuzzSeedError::violation("empty intent fixture"))?;
    *checksum ^= 1;
    seeds.push(seed(
        "recovery-corrupt-intent",
        2,
        0,
        intent,
        [intent, &corrupt, marker, marker, receipt, receipt],
    )?);
    Ok(seeds)
}

fn seed(
    name: &'static str,
    presence: u8,
    namespace: u8,
    expected: &[u8],
    records: [&[u8]; 6],
) -> Result<Seed, FuzzSeedError> {
    if expected.len() != 256 || records.iter().any(|record| record.len() > 513) {
        return Err(FuzzSeedError::violation(
            "recovery seed exceeds its record framing bounds",
        ));
    }
    let mut payload = expected.to_vec();
    payload.extend_from_slice(&[presence, namespace]);
    for record in records {
        let length = u16::try_from(record.len())
            .map_err(|_| FuzzSeedError::violation("recovery seed record exceeds framing bound"))?;
        payload.extend_from_slice(&length.to_le_bytes());
        payload.extend_from_slice(record);
    }
    Seed::new("migration_format", name, prefixed(3, &payload)?)
}
