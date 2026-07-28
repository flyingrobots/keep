//! Deterministic range-request generation.

use keep::{ByteLength, ByteOffset, ByteRange};

use crate::scenario_ingest_operation::to_u64;
use crate::{Scenario, ScenarioError};

const RANGE_COUNT: usize = 32;
const SEQUENTIAL_LENGTH: u64 = 32_768;
const RANDOM_LENGTH: u64 = 4_096;

pub(super) fn ranges(
    scenario: Scenario,
    source_length: usize,
) -> Result<Box<[ByteRange]>, ScenarioError> {
    let mut ranges = Vec::new();
    ranges
        .try_reserve_exact(RANGE_COUNT)
        .map_err(|source| ScenarioError::Allocation {
            target: "range-requests",
            source,
        })?;
    match scenario {
        Scenario::SequentialRangeReads => sequential_ranges(scenario, &mut ranges)?,
        Scenario::RandomRangeReads => random_ranges(scenario, source_length, &mut ranges)?,
        _ => {
            return Err(ScenarioError::CorpusRangeUnavailable {
                target: "range-scenario",
                available: source_length,
            });
        }
    }
    Ok(ranges.into_boxed_slice())
}

fn sequential_ranges(scenario: Scenario, ranges: &mut Vec<ByteRange>) -> Result<(), ScenarioError> {
    let mut offset = 0_u64;
    for _index in 0..RANGE_COUNT {
        ranges.push(byte_range(scenario, offset, SEQUENTIAL_LENGTH)?);
        offset = offset
            .checked_add(SEQUENTIAL_LENGTH)
            .ok_or(ScenarioError::MetricOverflow {
                metric: "sequential-range-offset",
                current: offset,
                incoming: SEQUENTIAL_LENGTH,
            })?;
    }
    Ok(())
}

fn random_ranges(
    scenario: Scenario,
    source_length: usize,
    ranges: &mut Vec<ByteRange>,
) -> Result<(), ScenarioError> {
    let source_length = to_u64(source_length, "random-range-source-length")?;
    let coordinate_count = source_length
        .checked_sub(RANDOM_LENGTH)
        .and_then(|maximum| maximum.checked_add(1))
        .ok_or(ScenarioError::MetricOverflow {
            metric: "random-range-coordinate-count",
            current: source_length,
            incoming: RANDOM_LENGTH,
        })?;
    let mut state = 0x1319_8a2e_0370_7344_u64;
    for _index in 0..RANGE_COUNT {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        let offset = state
            .checked_rem(coordinate_count)
            .ok_or(ScenarioError::MetricOverflow {
                metric: "random-range-offset",
                current: state,
                incoming: coordinate_count,
            })?;
        ranges.push(byte_range(scenario, offset, RANDOM_LENGTH)?);
    }
    Ok(())
}

fn byte_range(scenario: Scenario, offset: u64, length: u64) -> Result<ByteRange, ScenarioError> {
    ByteRange::new(ByteOffset::new(offset), ByteLength::new(length)).map_err(|source| {
        ScenarioError::ByteRange {
            scenario,
            source: Box::new(source),
        }
    })
}
