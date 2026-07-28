//! Validation and identity admission for benchmark chunk boundaries.

use std::ops::Range;

use keep::ChunkId;

use crate::{ChunkPartition, ProfileError};

impl ChunkPartition {
    pub(super) fn from_ends(source: &[u8], ends: Vec<usize>) -> Result<Self, ProfileError> {
        validate_ends(source.len(), &ends)?;
        let mut identities = Vec::new();
        identities
            .try_reserve_exact(ends.len())
            .map_err(|source| ProfileError::Allocation {
                target: "chunk-identities",
                source,
            })?;
        for range in ranges(&ends) {
            let bytes = source
                .get(range.clone())
                .ok_or(ProfileError::FinalBoundaryMismatch {
                    expected: source.len(),
                    observed: range.end,
                })?;
            let identity =
                ChunkId::hash_bytes(bytes).map_err(|source| ProfileError::ChunkIdentity {
                    start: range.start,
                    end: range.end,
                    source,
                })?;
            identities.push(identity);
        }
        Ok(Self {
            logical_bytes: source.len(),
            ends,
            identities,
        })
    }

    /// Returns exact input bytes consumed by this partition.
    #[must_use]
    pub const fn logical_bytes(&self) -> usize {
        self.logical_bytes
    }

    /// Returns ordered exclusive chunk end coordinates.
    #[must_use]
    pub fn ends(&self) -> &[usize] {
        &self.ends
    }

    /// Returns ordered exact chunk identities.
    #[must_use]
    pub fn identities(&self) -> &[ChunkId] {
        &self.identities
    }

    /// Counts distinct exact chunk identities in this partition.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::MetricOverflow`] if the bounded count cannot
    /// fit the metric coordinate.
    pub fn unique_chunk_count(&self) -> Result<usize, ProfileError> {
        let count = self
            .identities
            .iter()
            .enumerate()
            .filter(|(index, identity)| is_first(self.identities(), *index, identity))
            .try_fold(0_u64, |count, _identity| {
                count.checked_add(1).ok_or(ProfileError::MetricOverflow {
                    metric: "unique-chunk-count",
                    current: count,
                    incoming: 1,
                })
            })?;
        usize::try_from(count).map_err(|_source| ProfileError::MetricOverflow {
            metric: "unique-chunk-count",
            current: count,
            incoming: 0,
        })
    }

    /// Counts distinct chunk identities shared with another partition.
    ///
    /// Repeated occurrences count once because physical deduplication is keyed
    /// by exact [`ChunkId`].
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::MetricOverflow`] if the bounded count cannot
    /// fit the metric coordinate.
    pub fn reused_unique_chunk_count(&self, other: &Self) -> Result<usize, ProfileError> {
        let count = self
            .identities
            .iter()
            .enumerate()
            .filter(|(index, identity)| {
                is_first(self.identities(), *index, identity) && other.identities.contains(identity)
            })
            .try_fold(0_u64, |count, _identity| {
                count.checked_add(1).ok_or(ProfileError::MetricOverflow {
                    metric: "reused-unique-chunk-count",
                    current: count,
                    incoming: 1,
                })
            })?;
        usize::try_from(count).map_err(|_source| ProfileError::MetricOverflow {
            metric: "reused-unique-chunk-count",
            current: count,
            incoming: 0,
        })
    }

    /// Sums bytes named by distinct chunk identities.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::MetricOverflow`] if the bounded byte count
    /// cannot fit the metric coordinate.
    pub fn unique_materialized_bytes(&self) -> Result<u64, ProfileError> {
        self.identities
            .iter()
            .enumerate()
            .filter(|(index, identity)| is_first(self.identities(), *index, identity))
            .try_fold(0_u64, |total, (_index, identity)| {
                let incoming = u64::from(identity.length().get());
                total
                    .checked_add(incoming)
                    .ok_or(ProfileError::MetricOverflow {
                        metric: "unique-materialized-bytes",
                        current: total,
                        incoming,
                    })
            })
    }

    /// Iterates exact half-open chunk ranges in order.
    pub fn ranges(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        ranges(&self.ends)
    }
}

fn validate_ends(source_length: usize, ends: &[usize]) -> Result<(), ProfileError> {
    let mut previous = 0_usize;
    for observed in ends.iter().copied() {
        if observed <= previous {
            return Err(ProfileError::NonIncreasingBoundary { previous, observed });
        }
        previous = observed;
    }
    if previous != source_length {
        return Err(ProfileError::FinalBoundaryMismatch {
            expected: source_length,
            observed: previous,
        });
    }
    Ok(())
}

fn ranges(ends: &[usize]) -> impl Iterator<Item = Range<usize>> + '_ {
    ends.iter().copied().scan(0_usize, |start, end| {
        let range = *start..end;
        *start = end;
        Some(range)
    })
}

fn is_first(identities: &[ChunkId], index: usize, identity: &ChunkId) -> bool {
    identities
        .iter()
        .take(index)
        .all(|observed| observed != identity)
}
