//! Exact nearest-rank percentile selection for integer durations.

use crate::MeasurementError;

#[derive(Clone, Copy)]
pub(super) enum Percentile {
    P50,
    P95,
    P99,
}

impl Percentile {
    const fn numerator(self) -> usize {
        match self {
            Self::P50 => 50,
            Self::P95 => 95,
            Self::P99 => 99,
        }
    }
}

pub(super) fn nearest_rank(
    samples: &mut [u128],
    percentile: Percentile,
) -> Result<u128, MeasurementError> {
    if samples.is_empty() {
        return Err(MeasurementError::InvalidSampleCount {
            minimum: 1,
            maximum: 1_000,
            observed: 0,
        });
    }
    samples.sort_unstable();
    let scaled = samples
        .len()
        .checked_mul(percentile.numerator())
        .ok_or_else(|| MeasurementError::MetricArithmetic {
            metric: "percentile-rank",
            current: usize_to_u128(samples.len()),
            incoming: usize_to_u128(percentile.numerator()),
        })?;
    let rank = scaled
        .checked_add(99)
        .and_then(|ceiling| ceiling.checked_div(100))
        .and_then(|one_based| one_based.checked_sub(1))
        .ok_or_else(|| MeasurementError::MetricArithmetic {
            metric: "percentile-rank",
            current: usize_to_u128(scaled),
            incoming: 100,
        })?;
    samples
        .get(rank)
        .copied()
        .ok_or_else(|| MeasurementError::MetricArithmetic {
            metric: "percentile-coordinate",
            current: usize_to_u128(rank),
            incoming: usize_to_u128(samples.len()),
        })
}

fn usize_to_u128(value: usize) -> u128 {
    u128::try_from(value).unwrap_or(u128::MAX)
}
