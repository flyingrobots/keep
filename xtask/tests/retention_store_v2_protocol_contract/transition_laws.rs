//! Stable migration crash-transition ledger laws.

use keep::StoreMigrationPhase;

const TRANSITIONS: &str = include_str!("../../../conformance/segment-store/v2/transitions.tsv");

/// The `operation` column in `StoreMigrationPhase::ALL` order.
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

#[test]
fn migration_transition_ledger_is_complete_and_stable() -> Result<(), String> {
    assert!(TRANSITIONS.starts_with(
        "keep.segment-store.transitions/v2\n\
         crash_id\tphase\toperation\tpre_state\tinterrupted_class\t\
         post_state\trecovery_posture\n"
    ));
    assert_eq!(OPERATIONS.len(), StoreMigrationPhase::ALL.len());

    let mut row_count = 0usize;
    for (offset, row) in TRANSITIONS.lines().skip(2).enumerate() {
        let ordinal = offset
            .checked_add(53)
            .ok_or("transition ordinal overflow")?;
        let expected_id = format!("KEEP-CRASH-{ordinal:03}");
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.first(), Some(&expected_id.as_str()));
        assert_eq!(fields.get(1), Some(&"migration"), "{expected_id}");
        assert_eq!(fields.get(2), OPERATIONS.get(offset), "{expected_id}");
        assert_eq!(
            fields.len(),
            7,
            "transition {expected_id} must have seven fields"
        );
        assert!(
            fields.iter().all(|field| !field.is_empty()),
            "transition {expected_id} has an empty field"
        );
        row_count = row_count
            .checked_add(1)
            .ok_or("transition count overflow")?;
    }
    assert_eq!(row_count, 21);

    // Every stage write may leave an incomplete pre-effect stage, and only
    // those rows may plan a discard; the last two rows admit completion.
    for (ordinal, row) in TRANSITIONS.lines().skip(2).enumerate() {
        let discards = row.contains("discard-incomplete-stage");
        assert_eq!(discards, matches!(ordinal, 0 | 9 | 15), "{row}");
        let completes = row.ends_with("admit-complete-migration");
        assert_eq!(completes, ordinal >= 19, "{row}");
    }
    assert!(
        TRANSITIONS.contains("directory-prefix-length-zero-to-six"),
        "KEEP-CRASH-060 must name one case per admitted prefix length"
    );
    Ok(())
}
