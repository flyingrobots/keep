//! This boundary module owns the semantic GC retirement intent.

use super::{GcCandidate, GcRetirementIntentCoordinates, GcRetirementIntentError};

/// One validated retirement intent: its coordinates and its canonical,
/// duplicate-free, digest-ordered candidate set.
///
/// The value proves nothing about the store. It is the exact statement a
/// GC executor writes to `gc/intent` before the first unlink, so that
/// recovery can classify every later state against it.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcRetirementIntent {
    coordinates: GcRetirementIntentCoordinates,
    candidates: Vec<GcCandidate>,
}

impl GcRetirementIntent {
    /// Maximum candidate count in one intent.
    pub const MAXIMUM_CANDIDATE_COUNT: u32 = 65_536;

    /// Admits a retirement intent over a canonical candidate set.
    ///
    /// # Errors
    ///
    /// Returns [`GcRetirementIntentError`] when the set is empty, exceeds
    /// the maximum, repeats a segment, or is not in canonical
    /// segment-digest order.
    pub fn new(
        coordinates: GcRetirementIntentCoordinates,
        candidates: Vec<GcCandidate>,
    ) -> Result<Self, GcRetirementIntentError> {
        if candidates.is_empty() {
            return Err(GcRetirementIntentError::NoCandidates);
        }
        let observed = u32::try_from(candidates.len()).map_err(|_| {
            GcRetirementIntentError::CandidateCountExceeded {
                maximum: Self::MAXIMUM_CANDIDATE_COUNT,
                observed: u32::MAX,
            }
        })?;
        if observed > Self::MAXIMUM_CANDIDATE_COUNT {
            return Err(GcRetirementIntentError::CandidateCountExceeded {
                maximum: Self::MAXIMUM_CANDIDATE_COUNT,
                observed,
            });
        }
        require_canonical_order(&candidates)?;
        Ok(Self {
            coordinates,
            candidates,
        })
    }

    /// Returns every bound coordinate besides the candidates.
    pub const fn coordinates(&self) -> &GcRetirementIntentCoordinates {
        &self.coordinates
    }

    /// Returns the canonical candidate set.
    pub fn candidates(&self) -> &[GcCandidate] {
        &self.candidates
    }

    /// Returns the exact candidate count.
    #[must_use]
    pub fn candidate_count(&self) -> u32 {
        // The constructor bounded the length by `MAXIMUM_CANDIDATE_COUNT`.
        u32::try_from(self.candidates.len()).unwrap_or(Self::MAXIMUM_CANDIDATE_COUNT)
    }
}

fn require_canonical_order(candidates: &[GcCandidate]) -> Result<(), GcRetirementIntentError> {
    for (position, pair) in candidates.windows(2).enumerate() {
        let (Some(prior), Some(observed)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let index = u32::try_from(position)
            .ok()
            .and_then(|position| position.checked_add(1))
            .unwrap_or(u32::MAX);
        match observed.segment_digest().cmp(&prior.segment_digest()) {
            std::cmp::Ordering::Greater => {}
            std::cmp::Ordering::Equal => {
                return Err(GcRetirementIntentError::DuplicateCandidate { index });
            }
            std::cmp::Ordering::Less => {
                return Err(GcRetirementIntentError::NonCanonicalCandidateOrder { index });
            }
        }
    }
    Ok(())
}
