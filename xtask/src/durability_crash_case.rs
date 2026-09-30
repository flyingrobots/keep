//! This module owns validated coordinates in the deterministic crash matrix.

use crate::{
    DurabilityCrashCaseError, DurabilityCrashOccurrence, DurabilityCrashPoint,
    DurabilityCrashPosition, DurabilityCrashSequence,
};

/// One validated process-death coordinate in the durability crash matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurabilityCrashCase {
    point: DurabilityCrashPoint,
    position: DurabilityCrashPosition,
    occurrence: Option<DurabilityCrashOccurrence>,
}

impl DurabilityCrashCase {
    /// Creates a coordinate after validating occurrence ownership.
    ///
    /// # Errors
    ///
    /// Returns [`DurabilityCrashCaseError::MissingOccurrence`] when a repeated
    /// transition lacks an occurrence, or
    /// [`DurabilityCrashCaseError::UnexpectedOccurrence`] when a non-repeated
    /// transition receives one.
    pub const fn new(
        point: DurabilityCrashPoint,
        position: DurabilityCrashPosition,
        occurrence: Option<DurabilityCrashOccurrence>,
    ) -> Result<Self, DurabilityCrashCaseError> {
        match (point.occurrence_counted(), occurrence) {
            (true, None) => Err(DurabilityCrashCaseError::MissingOccurrence { point }),
            (false, Some(observed)) => {
                Err(DurabilityCrashCaseError::UnexpectedOccurrence { point, observed })
            }
            (_, occurrence) => Ok(Self {
                point,
                position,
                occurrence,
            }),
        }
    }

    /// Returns every canonical case in point-major, position-minor order.
    ///
    /// A boundary with more than one `during` occurrence contributes one
    /// `during` case per occurrence, in occurrence order, between its
    /// `before` and `after` cases.
    pub fn all() -> impl Iterator<Item = Self> {
        DurabilityCrashPoint::ALL.into_iter().flat_map(|point| {
            DurabilityCrashPosition::ALL
                .into_iter()
                .flat_map(move |position| Self::canonical_at(point, position))
        })
    }

    /// Returns every canonical case whose boundary belongs to `sequence`.
    pub fn in_sequence(sequence: DurabilityCrashSequence) -> impl Iterator<Item = Self> {
        Self::all().filter(move |case| case.point().sequence() == sequence)
    }

    fn canonical_at(
        point: DurabilityCrashPoint,
        position: DurabilityCrashPosition,
    ) -> impl Iterator<Item = Self> {
        let occurrences = if position == DurabilityCrashPosition::During {
            point.during_occurrences()
        } else {
            1
        };
        (0..occurrences).map(move |ordinal| {
            let mut case = Self::canonical(point, position);
            if point.occurrence_counted() {
                case.occurrence = Some(DurabilityCrashOccurrence::new(ordinal));
            }
            case
        })
    }

    /// Returns the transition targeted by this coordinate.
    #[must_use]
    pub const fn point(self) -> DurabilityCrashPoint {
        self.point
    }

    /// Returns the process-death position targeted by this coordinate.
    #[must_use]
    pub const fn position(self) -> DurabilityCrashPosition {
        self.position
    }

    /// Returns the occurrence coordinate when the transition repeats.
    #[must_use]
    pub const fn occurrence(self) -> Option<DurabilityCrashOccurrence> {
        self.occurrence
    }

    const fn canonical(point: DurabilityCrashPoint, position: DurabilityCrashPosition) -> Self {
        Self {
            point,
            position,
            occurrence: if point.occurrence_counted() {
                Some(DurabilityCrashOccurrence::FIRST)
            } else {
                None
            },
        }
    }
}
