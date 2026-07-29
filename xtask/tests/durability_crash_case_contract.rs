//! Canonical deterministic crash-matrix coordinate laws.

#![cfg(feature = "repository-tasks")]

use std::error::Error;

use xtask::{
    DurabilityCrashCase, DurabilityCrashCaseError, DurabilityCrashOccurrence, DurabilityCrashPoint,
    DurabilityCrashPosition,
};

#[test]
fn every_crash_point_has_exactly_three_ordered_process_death_cases() -> Result<(), Box<dyn Error>> {
    let cases: Vec<_> = DurabilityCrashCase::all().collect();
    let expected = DurabilityCrashPoint::ALL
        .len()
        .checked_mul(DurabilityCrashPosition::ALL.len())
        .ok_or("crash-matrix case count overflow")?;

    assert_eq!(cases.len(), expected);
    for (point_index, point) in DurabilityCrashPoint::ALL.into_iter().enumerate() {
        for (position_index, position) in DurabilityCrashPosition::ALL.into_iter().enumerate() {
            let index = point_index
                .checked_mul(DurabilityCrashPosition::ALL.len())
                .and_then(|base| base.checked_add(position_index))
                .ok_or("crash-matrix index overflow")?;
            let case = cases.get(index).ok_or("missing canonical crash case")?;
            assert_eq!(case.point(), point);
            assert_eq!(case.position(), position);
            assert_eq!(
                case.occurrence(),
                point
                    .occurrence_counted()
                    .then_some(DurabilityCrashOccurrence::FIRST)
            );
        }
    }
    Ok(())
}

#[test]
fn occurrence_coordinates_exist_only_for_record_append() -> Result<(), Box<dyn Error>> {
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
