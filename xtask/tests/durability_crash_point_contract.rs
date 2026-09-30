//! Executable identity contract for deterministic crash injection.

#![cfg(feature = "repository-tasks")]

use xtask::{DurabilityCrashPoint, DurabilityCrashSequence};

use DurabilityCrashSequence::{Gc, Migration};

use expected::EXPECTED;

#[path = "durability_crash_point_contract/expected.rs"]
mod expected;

#[test]
fn crash_boundaries_have_one_contiguous_stable_vocabulary() {
    let actual =
        DurabilityCrashPoint::ALL.map(|point| (point, point.identifier(), point.sequence()));

    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn only_record_append_and_namespace_prefix_select_an_occurrence() {
    let occurrence_counted: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.occurrence_counted())
        .collect();

    assert_eq!(
        occurrence_counted,
        [
            DurabilityCrashPoint::AppendSegmentRecord,
            DurabilityCrashPoint::MigrationAdmitNamespacePrefix
        ]
    );
}

#[test]
fn the_namespace_prefix_runs_one_during_case_per_directory() {
    for point in DurabilityCrashPoint::ALL {
        let expected = if point == DurabilityCrashPoint::MigrationAdmitNamespacePrefix {
            DurabilityCrashPoint::NAMESPACE_PREFIX_DIRECTORIES
        } else {
            1
        };
        assert_eq!(
            point.during_occurrences(),
            expected,
            "{}",
            point.identifier()
        );
    }
    assert_eq!(DurabilityCrashPoint::NAMESPACE_PREFIX_DIRECTORIES, 6);
}

#[test]
fn migration_boundaries_follow_the_twenty_one_phases_in_order() {
    let migration: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.sequence() == Migration)
        .collect();

    assert_eq!(migration, DurabilityCrashPoint::MIGRATION);
    assert_eq!(migration.len(), keep::StoreMigrationPhase::ALL.len());
    assert_eq!(
        DurabilityCrashPoint::MIGRATION.map(DurabilityCrashPoint::identifier),
        std::array::from_fn::<_, 21, _>(|index| {
            let ordinal = 53 + index;
            let identifier = format!("KEEP-CRASH-{ordinal:03}");
            DurabilityCrashPoint::from_identifier(&identifier)
                .map_or("missing", DurabilityCrashPoint::identifier)
        })
    );
}

#[test]
fn gc_boundaries_follow_the_fourteen_phases_in_order() {
    let gc: Vec<_> = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.sequence() == Gc)
        .collect();

    assert_eq!(gc, DurabilityCrashPoint::GC);
    assert_eq!(gc.len(), keep::GcExecutionPhase::ALL.len());
    assert_eq!(
        DurabilityCrashPoint::GC.map(DurabilityCrashPoint::identifier),
        std::array::from_fn::<_, 14, _>(|index| {
            let ordinal = 74 + index;
            let identifier = format!("KEEP-CRASH-{ordinal:03}");
            DurabilityCrashPoint::from_identifier(&identifier)
                .map_or("missing", DurabilityCrashPoint::identifier)
        })
    );
}

#[test]
fn sequences_round_trip_their_command_line_identifiers() {
    for sequence in DurabilityCrashSequence::ALL {
        assert_eq!(
            DurabilityCrashSequence::from_identifier(sequence.identifier()),
            Some(sequence)
        );
    }
    assert_eq!(
        DurabilityCrashSequence::from_identifier("publication"),
        None
    );
}
