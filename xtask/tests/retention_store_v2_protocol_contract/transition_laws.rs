//! This module owns canonical initialization transition-ledger admission laws.

use xtask::{DurabilityCrashPoint, DurabilityCrashSequence};

const TRANSITIONS: &str = include_str!("../../../conformance/segment-store/v2/transitions.tsv");
const HEADER: &str = "keep.segment-store.transitions/v2\n\
    crash_id\tphase\toperation\tpre_state\tinterrupted_class\t\
    post_state\trecovery_posture\n";

/// The operation column in initialization crash-point order, `KEEP-CRASH-081` onward.
const OPERATIONS: [&str; 11] = [
    "open-and-lock-writer-file",
    "write-marker-stage",
    "sync-marker-stage",
    "sync-root-after-marker-stage",
    "admit-reader-fence",
    "admit-directory",
    "link-marker",
    "sync-root-after-marker",
    "remove-marker-stage",
    "sync-root-after-cleanup",
    "discard-incomplete-marker-stage",
];
const FIRST_ORDINAL: usize = 81;

#[derive(Debug, Eq, PartialEq)]
enum LedgerRefusal {
    Encoding,
    Header,
    RowCount { observed: usize },
    Fields { row: usize },
    Coordinate { row: usize },
    Posture { row: usize },
    DirectoryExtent,
}

#[test]
fn initialization_transition_coordinates_are_complete_and_ordered() {
    let points = DurabilityCrashPoint::ALL
        .into_iter()
        .filter(|point| point.sequence() == DurabilityCrashSequence::Initialization)
        .count();
    assert_eq!(OPERATIONS.len(), points);
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
        "locked-root\tabsent-or-incomplete-marker-stage\tcomplete-marker-stage\t\
         discard-incomplete-stage-or-resume-stage-sync",
        "discard-incomplete-stage\tabsent-or-incomplete-marker-stage\t\
         complete-marker-stage\tresume-stage-sync",
        1,
    );
    assert_ne!(misplaced, TRANSITIONS);
    assert_eq!(admit(&misplaced), Err(LedgerRefusal::Posture { row: 1 }));
}

#[test]
fn committed_posture_requires_its_exact_field_value() {
    let altered = TRANSITIONS.replacen(
        "\tadmit-committed-store\n",
        "\tunknown-admit-committed-store\n",
        1,
    );
    assert_eq!(admit(&altered), Err(LedgerRefusal::Posture { row: 9 }));
}

#[test]
fn extra_transition_rows_refuse_before_row_admission() {
    let extra = format!("{TRANSITIONS}foreign\n");
    assert_eq!(admit(&extra), Err(LedgerRefusal::RowCount { observed: 12 }));
}

fn admit(document: &str) -> Result<(), LedgerRefusal> {
    if !document.is_ascii() || document.contains('\r') || !document.ends_with('\n') {
        return Err(LedgerRefusal::Encoding);
    }
    if !document.starts_with(HEADER) {
        return Err(LedgerRefusal::Header);
    }
    let rows: Vec<_> = document.lines().skip(2).collect();
    if rows.len() != OPERATIONS.len() {
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
        .checked_add(FIRST_ORDINAL)
        .ok_or(LedgerRefusal::Coordinate { row: offset })?;
    if id != format!("KEEP-CRASH-{ordinal:03}")
        || DurabilityCrashPoint::from_identifier(id).map(DurabilityCrashPoint::sequence)
            != Some(DurabilityCrashSequence::Initialization)
        || phase != "initialization"
        || Some(&operation) != OPERATIONS.get(offset)
    {
        return Err(LedgerRefusal::Coordinate { row: offset });
    }
    if Some(posture) != expected_posture(operation) {
        return Err(LedgerRefusal::Posture { row: offset });
    }
    if id == "KEEP-CRASH-086" && interrupted != "directory-prefix-length-zero-to-nine" {
        return Err(LedgerRefusal::DirectoryExtent);
    }
    Ok(())
}

fn expected_posture(operation: &str) -> Option<&'static str> {
    match operation {
        "open-and-lock-writer-file" => Some("resume-marker-stage-write"),
        "write-marker-stage" => Some("discard-incomplete-stage-or-resume-stage-sync"),
        "sync-marker-stage"
        | "sync-root-after-marker-stage"
        | "admit-reader-fence"
        | "admit-directory" => Some("resume-stage-sync"),
        "link-marker" => Some("verify-no-clobber-link-and-resume-commit-sync"),
        "sync-root-after-marker" => Some("resume-commit-sync"),
        "remove-marker-stage" => Some("resume-commit-sync-or-admit-committed-store"),
        "sync-root-after-cleanup" => Some("admit-committed-store"),
        "discard-incomplete-marker-stage" => Some("discard-incomplete-stage-or-resume-stage-write"),
        _ => None,
    }
}
