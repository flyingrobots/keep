//! Benchmark-only Keep `FastCDC` candidate boundary calculation.

use crate::{ChunkingProfile, ProfileError};

#[path = "../../src/chunk/gear_table.rs"]
mod canonical_gear;

struct Parameters {
    minimum: usize,
    target: usize,
    maximum: usize,
    short_mask: u64,
    long_mask: u64,
}

#[allow(
    clippy::redundant_pub_crate,
    reason = "the sibling profile dispatcher is the only consumer"
)]
pub(super) fn partition(
    profile: ChunkingProfile,
    source: &[u8],
) -> Result<Vec<usize>, ProfileError> {
    let parameters = parameters(profile);
    let capacity = source
        .len()
        .checked_div(parameters.minimum)
        .and_then(|chunks| chunks.checked_add(1))
        .ok_or(ProfileError::CoordinateOverflow {
            current: source.len(),
            incoming: parameters.minimum,
        })?;
    let mut ends = Vec::new();
    ends.try_reserve_exact(capacity)
        .map_err(|source| ProfileError::Allocation {
            target: "keep-fastcdc-boundaries",
            source,
        })?;
    scan(source, &parameters, &mut ends)?;
    Ok(ends)
}

fn scan(source: &[u8], parameters: &Parameters, ends: &mut Vec<usize>) -> Result<(), ProfileError> {
    let mut index = 0_usize;
    let mut candidate_length = 0_usize;
    let mut fingerprint = 0_u64;
    while index < source.len() {
        let byte = source
            .get(index)
            .copied()
            .ok_or(ProfileError::FinalBoundaryMismatch {
                expected: source.len(),
                observed: index,
            })?;
        let next_length =
            candidate_length
                .checked_add(1)
                .ok_or(ProfileError::CoordinateOverflow {
                    current: candidate_length,
                    incoming: 1,
                })?;
        if candidate_length < parameters.minimum {
            candidate_length = next_length;
            index = next(index)?;
            continue;
        }
        let gear = canonical_gear::GEAR_TABLE
            .get(usize::from(byte))
            .copied()
            .ok_or(ProfileError::MissingGearEntry { byte })?;
        let next_fingerprint = fingerprint.wrapping_shl(1).wrapping_add(gear);
        let mask = if candidate_length < parameters.target {
            parameters.short_mask
        } else {
            parameters.long_mask
        };
        if next_fingerprint & mask == 0 {
            ends.push(index);
            candidate_length = 1;
            fingerprint = 0;
            index = next(index)?;
            continue;
        }
        candidate_length = next_length;
        fingerprint = next_fingerprint;
        index = next(index)?;
        if candidate_length == parameters.maximum {
            ends.push(index);
            candidate_length = 0;
            fingerprint = 0;
        }
    }
    if ends.last().copied().unwrap_or(0) != source.len() {
        ends.push(source.len());
    }
    Ok(())
}

const fn parameters(profile: ChunkingProfile) -> Parameters {
    match profile {
        ChunkingProfile::KeepFastCdcSmall => Parameters {
            minimum: 4_096,
            target: 16_384,
            maximum: 65_536,
            short_mask: 0x0000_d903_0353_7000,
            long_mask: 0x0000_d901_0353_0000,
        },
        ChunkingProfile::KeepFastCdcRegistered => Parameters {
            minimum: 16_384,
            target: 65_536,
            maximum: 262_144,
            short_mask: 0x0000_d907_0753_7000,
            long_mask: 0x0000_d903_1353_0000,
        },
        ChunkingProfile::KeepFastCdcLarge => Parameters {
            minimum: 65_536,
            target: 262_144,
            maximum: 1_048_576,
            short_mask: 0x0000_d917_4753_7000,
            long_mask: 0x0000_d903_0353_7000,
        },
        ChunkingProfile::Fixed64KiB | ChunkingProfile::GitCasDefault => Parameters {
            minimum: 1,
            target: 1,
            maximum: 1,
            short_mask: 1,
            long_mask: 1,
        },
    }
}

fn next(current: usize) -> Result<usize, ProfileError> {
    current
        .checked_add(1)
        .ok_or(ProfileError::CoordinateOverflow {
            current,
            incoming: 1,
        })
}
