//! Executable identity contract for deterministic crash injection.

#![cfg(feature = "repository-tasks")]

use xtask::{DurabilityCrashPoint, DurabilityCrashSequence};

use DurabilityCrashSequence::{
    Catalog, Gc, Head, Initialization, Migration, RecoveryDiscard, Retention, Segment,
};

const fn expected() -> &'static [(DurabilityCrashPoint, &'static str, DurabilityCrashSequence)] {
    include!("durability_crash_point_contract/expected.rs")
}

#[test]
fn crash_boundaries_have_one_contiguous_stable_vocabulary() {
    let actual =
        DurabilityCrashPoint::ALL.map(|point| (point, point.identifier(), point.sequence()));

    assert_eq!(actual.as_slice(), expected());
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

// Subject: repository-tool runtime, not Keep recovery acceptance evidence.
// Size: small; oracle: the exact sequence names documented in README.md.
// Delete if this CLI is removed or a stronger command-boundary test subsumes it.
#[test]
fn crash_matrix_cli_admits_its_stable_sequence_names() {
    for (name, expected) in [
        ("segment", Segment),
        ("catalog", Catalog),
        ("head", Head),
        ("recovery-discard", RecoveryDiscard),
        ("initialization", Initialization),
        ("retention", Retention),
        ("migration", Migration),
        ("gc", Gc),
    ] {
        assert_eq!(
            DurabilityCrashSequence::from_identifier(name),
            Some(expected),
            "CLI sequence {name} must select {expected:?}"
        );
    }
}

// Subject: repository-tool runtime. Size: small; oracle: exact CLI vocabulary.
// Delete if this CLI is removed or a stronger command-boundary test subsumes it.
#[test]
fn crash_matrix_cli_refuses_names_outside_its_exact_vocabulary() {
    for name in ["", "unknown", "Retention", "retentions", "retention "] {
        assert_eq!(
            DurabilityCrashSequence::from_identifier(name),
            None,
            "unsupported CLI sequence {name:?} must refuse"
        );
    }
}
