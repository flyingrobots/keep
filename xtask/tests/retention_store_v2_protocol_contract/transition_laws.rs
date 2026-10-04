//! This module owns canonical migration transition-ledger admission laws.

use keep::{GcExecutionPhase, StoreMigrationPhase};

const TRANSITIONS: &str = include_str!("../../../conformance/segment-store/v2/transitions.tsv");
const HEADER: &str = "keep.segment-store.transitions/v2\n\
    crash_id\tphase\toperation\tpre_state\tinterrupted_class\t\
    post_state\trecovery_posture\n";

/// The operation column in `StoreMigrationPhase::ALL` order.
const OPERATIONS: [&str; 21] = [
    "write-intent-stage",
    "sync-intent-stage",
    "link-intent",
    "sync-root-after-intent",
    "remove-intent-stage",
    "sync-root-after-intent-cleanup",
    "admit-reader-fence",
    "admit-namespace-prefix",
    "sync-root-after-namespace",
    "write-marker-stage",
    "sync-marker-stage",
    "link-marker",
    "sync-root-after-marker",
    "remove-marker-stage",
    "sync-root-after-marker-cleanup",
    "write-receipt-stage",
    "sync-receipt-stage",
    "link-receipt",
    "sync-root-after-receipt",
    "remove-receipt-stage",
    "sync-root-after-receipt-cleanup",
];

/// The GC operation column in `GcExecutionPhase::ALL` order.
const GC_OPERATIONS: [&str; 14] = [
    "write-intent-stage",
    "sync-intent-stage",
    "link-intent",
    "sync-gc-after-intent",
    "remove-intent-stage",
    "sync-gc-after-intent-cleanup",
    "unlink-candidate",
    "sync-segment-pool",
    "write-receipt-stage",
    "sync-receipt-stage",
    "replace-receipt",
    "sync-gc-after-receipt",
    "remove-intent",
    "sync-gc-after-intent-removal",
];

#[derive(Debug, Eq, PartialEq)]

enum LedgerRefusal {
    Encoding,
    Header,
    RowCount { observed: usize },
    Fields { row: usize },
    Coordinate { row: usize },
    Posture { row: usize },
    NamespaceExtent,
}

#[test]
fn transition_coordinates_are_complete_and_ordered() {
    assert_eq!(OPERATIONS.len(), StoreMigrationPhase::ALL.len());
    assert_eq!(GC_OPERATIONS.len(), GcExecutionPhase::ALL.len());
    assert_eq!(admit(TRANSITIONS), Ok(()));
}

#[test]
fn transition_encoding_rejects_crlf_and_missing_final_newline() -> Result<(), String> {
    let crlf = TRANSITIONS.replacen("resume-stage-sync\n", "resume-stage-sync\r\n", 1);
    assert_eq!(admit(&crlf), Err(LedgerRefusal::Encoding));
    let unterminated = TRANSITIONS
        .strip_suffix('\n')
        .ok_or("fixture has no newline")?;
    assert_eq!(admit(unterminated), Err(LedgerRefusal::Encoding));
    Ok(())
}

#[test]
fn discard_posture_cannot_be_hidden_in_another_column() {
    let misplaced = TRANSITIONS.replacen(
        "admitted-version-one-store\tabsent-or-incomplete-intent-stage\t\
         complete-intent-stage\tdiscard-incomplete-stage-or-resume-stage-sync",
        "discard-incomplete-stage\tabsent-or-incomplete-intent-stage\t\
         complete-intent-stage\tresume-stage-sync",
        1,
    );
    assert_ne!(misplaced, TRANSITIONS);
    assert_eq!(admit(&misplaced), Err(LedgerRefusal::Posture { row: 0 }));
}

#[test]
fn completion_posture_requires_its_exact_field_value() {
    let altered = TRANSITIONS.replacen(
        "\tadmit-complete-migration\n",
        "\tunknown-admit-complete-migration\n",
        1,
    );
    assert_eq!(admit(&altered), Err(LedgerRefusal::Posture { row: 19 }));
}

#[test]
fn extra_transition_rows_refuse_before_row_admission() {
    let extra = format!("{TRANSITIONS}foreign\n");
    assert_eq!(admit(&extra), Err(LedgerRefusal::RowCount { observed: 36 }));
}

fn admit(document: &str) -> Result<(), LedgerRefusal> {
    if !document.is_ascii() || document.contains('\r') || !document.ends_with('\n') {
        return Err(LedgerRefusal::Encoding);
    }
    if !document.starts_with(HEADER) {
        return Err(LedgerRefusal::Header);
    }
    let rows: Vec<_> = document.lines().skip(2).collect();
    if rows.len() != OPERATIONS.iter().chain(GC_OPERATIONS.iter()).count() {
        return Err(LedgerRefusal::RowCount {
            observed: rows.len(),
        });
    }
    for (offset, row) in rows.iter().enumerate() {
        admit_row(offset, row)?;
    }
    Ok(())
}

fn admit_row(offset: usize, row: &str) -> Result<(), LedgerRefusal> {
    let fields: Vec<_> = row.split('\t').collect();
    let [id, phase, operation, pre, interrupted, post, posture]: [&str; 7] = fields
        .try_into()
        .map_err(|_| LedgerRefusal::Fields { row: offset })?;
    if [id, phase, operation, pre, interrupted, post, posture]
        .into_iter()
        .any(str::is_empty)
    {
        return Err(LedgerRefusal::Fields { row: offset });
    }
    let ordinal = offset
        .checked_add(53)
        .ok_or(LedgerRefusal::Coordinate { row: offset })?;
    let (expected_phase, expected_operation) = if offset < OPERATIONS.len() {
        ("migration", OPERATIONS.get(offset))
    } else {
        (
            "gc",
            GC_OPERATIONS.get(offset.saturating_sub(OPERATIONS.len())),
        )
    };
    if id != format!("KEEP-CRASH-{ordinal:03}")
        || phase != expected_phase
        || Some(&operation) != expected_operation
    {
        return Err(LedgerRefusal::Coordinate { row: offset });
    }
    let expected = if phase == "migration" {
        expected_posture(operation)
    } else {
        expected_gc_posture(operation)
    };
    if Some(posture) != expected {
        return Err(LedgerRefusal::Posture { row: offset });
    }
    if id == "KEEP-CRASH-060" && interrupted != "directory-prefix-length-zero-to-six" {
        return Err(LedgerRefusal::NamespaceExtent);
    }
    Ok(())
}

fn expected_posture(operation: &str) -> Option<&'static str> {
    match operation {
        "write-intent-stage" | "write-marker-stage" | "write-receipt-stage" => {
            Some("discard-incomplete-stage-or-resume-stage-sync")
        }
        "sync-intent-stage" | "sync-marker-stage" | "sync-receipt-stage" => {
            Some("resume-stage-sync")
        }
        "link-intent" | "link-marker" | "link-receipt" => {
            Some("verify-no-clobber-link-and-resume-root-sync")
        }
        "sync-root-after-intent"
        | "sync-root-after-marker"
        | "sync-root-after-receipt"
        | "sync-root-after-namespace" => Some("resume-root-sync"),
        "remove-intent-stage"
        | "remove-marker-stage"
        | "sync-root-after-intent-cleanup"
        | "sync-root-after-marker-cleanup" => Some("resume-cleanup-sync"),
        "admit-reader-fence" => Some("resume-namespace-prefix"),
        "admit-namespace-prefix" => Some("resume-namespace-prefix-or-root-sync"),
        "remove-receipt-stage" | "sync-root-after-receipt-cleanup" => {
            Some("admit-complete-migration")
        }
        _ => None,
    }
}

fn expected_gc_posture(operation: &str) -> Option<&'static str> {
    match operation {
        "write-intent-stage" | "write-receipt-stage" => {
            Some("discard-incomplete-stage-or-resume-stage-sync")
        }
        "sync-intent-stage" | "sync-receipt-stage" => Some("resume-stage-sync"),
        "link-intent" => Some("verify-no-clobber-link-and-resume-gc-sync"),
        "sync-gc-after-intent" | "sync-gc-after-receipt" => Some("resume-gc-sync"),
        "remove-intent-stage" | "sync-gc-after-intent-cleanup" => Some("resume-cleanup-sync"),
        "unlink-candidate" => Some("verify-prefix-and-resume-at-first-present-candidate"),
        "sync-segment-pool" => Some("resume-pool-sync"),
        "replace-receipt" => Some("verify-exact-receipt-and-resume-gc-sync"),
        "remove-intent" | "sync-gc-after-intent-removal" => Some("admit-complete-retirement"),
        _ => None,
    }
}
