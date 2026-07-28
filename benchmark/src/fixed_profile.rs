//! Fixed-size shift-sensitive comparison baseline.

use crate::ProfileError;

const CHUNK_BYTES: usize = 65_536;

#[allow(
    clippy::redundant_pub_crate,
    reason = "the sibling profile dispatcher is the only consumer"
)]
pub(super) fn partition(source: &[u8]) -> Result<Vec<usize>, ProfileError> {
    let capacity = source
        .len()
        .checked_div(CHUNK_BYTES)
        .and_then(|chunks| chunks.checked_add(1))
        .ok_or(ProfileError::CoordinateOverflow {
            current: source.len(),
            incoming: CHUNK_BYTES,
        })?;
    let mut ends = Vec::new();
    ends.try_reserve_exact(capacity)
        .map_err(|source| ProfileError::Allocation {
            target: "fixed-boundaries",
            source,
        })?;
    let mut current = 0_usize;
    while current < source.len() {
        current = current
            .checked_add(CHUNK_BYTES)
            .ok_or(ProfileError::CoordinateOverflow {
                current,
                incoming: CHUNK_BYTES,
            })?
            .min(source.len());
        ends.push(current);
    }
    Ok(ends)
}
