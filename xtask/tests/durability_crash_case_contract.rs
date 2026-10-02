//! Canonical deterministic crash-matrix coordinate laws.

#![cfg(feature = "repository-tasks")]

use std::error::Error;

use xtask::{
    DurabilityCrashCase, DurabilityCrashCaseError, DurabilityCrashOccurrence, DurabilityCrashPoint,
    DurabilityCrashPosition,
};

#[test]
fn occurrence_coordinates_exist_only_for_counted_boundaries() -> Result<(), Box<dyn Error>> {
    let occurrence = DurabilityCrashOccurrence::new(7);

    let counted = DurabilityCrashCase::new(
        DurabilityCrashPoint::AppendSegmentRecord,
        DurabilityCrashPosition::During,
        Some(occurrence),
    )?;
    assert_eq!(counted.occurrence(), Some(occurrence));

    let missing = DurabilityCrashCase::new(
        DurabilityCrashPoint::AppendSegmentRecord,
        DurabilityCrashPosition::During,
        None,
    );
    assert_eq!(
        missing,
        Err(DurabilityCrashCaseError::MissingOccurrence {
            point: DurabilityCrashPoint::AppendSegmentRecord,
        })
    );

    let unexpected = DurabilityCrashCase::new(
        DurabilityCrashPoint::WriteSegmentHeader,
        DurabilityCrashPosition::During,
        Some(occurrence),
    );
    assert_eq!(
        unexpected,
        Err(DurabilityCrashCaseError::UnexpectedOccurrence {
            point: DurabilityCrashPoint::WriteSegmentHeader,
            observed: occurrence,
        })
    );
    Ok(())
}

#[test]
fn identifiers_and_positions_round_trip_without_aliases() {
    for point in DurabilityCrashPoint::ALL {
        assert_eq!(
            DurabilityCrashPoint::from_identifier(point.identifier()),
            Some(point)
        );
    }
    assert_eq!(
        DurabilityCrashPoint::from_identifier("KEEP-CRASH-000"),
        None
    );

    for position in DurabilityCrashPosition::ALL {
        assert_eq!(
            DurabilityCrashPosition::from_identifier(position.identifier()),
            Some(position)
        );
    }
    assert_eq!(DurabilityCrashPosition::from_identifier("between"), None);
}

#[test]
fn namespace_occurrences_are_admitted_only_within_the_protocol_fixed_range()
-> Result<(), Box<dyn Error>> {
    let point = DurabilityCrashPoint::MigrationAdmitNamespacePrefix;
    for position in DurabilityCrashPosition::ALL {
        let exclusive_limit = if position == DurabilityCrashPosition::During {
            DurabilityCrashPoint::NAMESPACE_PREFIX_DIRECTORIES
        } else {
            1
        };
        for ordinal in 0..exclusive_limit {
            let occurrence = DurabilityCrashOccurrence::new(ordinal);
            let case = DurabilityCrashCase::new(point, position, Some(occurrence))?;
            assert_eq!(case.occurrence(), Some(occurrence));
        }
        for ordinal in [exclusive_limit, u32::MAX] {
            let observed = DurabilityCrashOccurrence::new(ordinal);
            assert_eq!(
                DurabilityCrashCase::new(point, position, Some(observed)),
                Err(DurabilityCrashCaseError::OccurrenceOutOfRange {
                    point,
                    observed,
                    exclusive_limit,
                })
            );
        }
    }
    Ok(())
}
