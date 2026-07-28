//! Stable crash-transition ledger laws.

use super::{CONFORMANCE_GUIDE, SPECIFICATION, TRANSITIONS};

const EXACT_TRANSITIONS: &[&str] = &[
    "KEEP-CRASH-008\tsegment\tsync-sealed-stage\t\
     flushed-sealed-stage\ttruncated-tail-or-valid-sealed-stage\t\
     durable-sealed-stage\tcomplete-pool-publication-or-preserve",
    "KEEP-CRASH-009\tsegment\tlink-sealed-stage\t\
     durable-sealed-stage\t\
     valid-sealed-stage-or-valid-orphan-or-ambiguity\t\
     linked-segment-orphan\tverify-no-clobber-link-and-digest",
    "KEEP-CRASH-010\tsegment\tsync-segment-directory\t\
     linked-segment-orphan\tvalid-sealed-stage-or-valid-orphan\t\
     durable-linked-segment-orphan\tpreserve-invisible",
    "KEEP-CRASH-016\tcatalog\tsync-generation\t\
     flushed-catalog-stage\ttruncated-tail-or-valid-stage\t\
     durable-catalog-stage\tcomplete-pool-publication-or-preserve",
    "KEEP-CRASH-017\tcatalog\tlink-generation\t\
     durable-catalog-stage\t\
     valid-catalog-stage-or-valid-orphan-or-ambiguity\t\
     linked-catalog-orphan\tverify-no-clobber-link-and-digest",
    "KEEP-CRASH-018\tcatalog\tsync-catalog-directory\t\
     linked-catalog-orphan\tvalid-catalog-stage-or-valid-orphan\t\
     durable-linked-catalog-orphan\tpreserve-invisible",
    "KEEP-CRASH-025\thead\treplace-current-head\t\
     durable-next-head\t\
     valid-next-head-or-published-generation-or-ambiguity\t\
     replaced-current-head\tverify-one-atomic-head",
    "KEEP-CRASH-026\thead\tsync-root-directory\t\
     replaced-current-head\t\
     valid-next-head-or-published-generation-or-ambiguity\t\
     published-generation-admitted\tverify-complete-reader-snapshot",
    "KEEP-CRASH-027\trecovery\tunlink-truncated-stage\t\
     named-truncated-stage\t\
     named-truncated-stage-or-unlinked-stage\t\
     unlinked-truncated-stage\tverify-requested-stage-fingerprint",
    "KEEP-CRASH-028\trecovery\tsync-staging-after-discard\t\
     unlinked-truncated-stage\t\
     named-truncated-stage-or-discarded-stage\t\
     discarded-truncated-stage\treport-explicit-discard",
    "KEEP-CRASH-029\trecovery\tunlink-next-head\t\
     named-unpublishable-next-head\t\
     named-unpublishable-next-head-or-unlinked-next-head\t\
     unlinked-next-head\tverify-requested-next-head-fingerprint",
    "KEEP-CRASH-030\trecovery\tsync-root-after-next-head-discard\t\
     unlinked-next-head\t\
     named-unpublishable-next-head-or-discarded-next-head\t\
     discarded-next-head\treport-explicit-next-head-discard",
    "KEEP-CRASH-031\tinitialization\testablish-writer-lock\t\
     capability-probed-initialization-root\trecoverable-initialization\t\
     locked-initialization-root\treopen-or-create-and-lock",
    "KEEP-CRASH-032\tinitialization\tcreate-staging-directory\t\
     locked-initialization-root\trecoverable-initialization\t\
     staging-directory-present\tverify-canonical-initialization-set",
    "KEEP-CRASH-033\tinitialization\tcreate-segment-directory\t\
     staging-directory-present\trecoverable-initialization\t\
     segment-directory-present\tverify-canonical-initialization-set",
    "KEEP-CRASH-034\tinitialization\tcreate-catalog-directory\t\
     segment-directory-present\trecoverable-initialization\t\
     complete-unsynchronized-initialization-set\t\
     verify-canonical-initialization-set",
    "KEEP-CRASH-035\tinitialization\tsync-root-after-initialization\t\
     complete-unsynchronized-initialization-set\t\
     recoverable-initialization-or-uninitialized-store\t\
     uninitialized-store-admitted\treport-initialization",
];

#[test]
fn conformance_guide_routes_recovery_and_initialization_transitions() {
    for required in [
        "`KEEP-CRASH-027`–`KEEP-CRASH-030` own explicit recovery discard",
        "`KEEP-CRASH-031`–`KEEP-CRASH-035` own crash-safe initialization",
    ] {
        assert!(
            CONFORMANCE_GUIDE.contains(required),
            "missing transition-range ownership: {required}"
        );
    }
    assert!(!CONFORMANCE_GUIDE.contains("The final four rows"));
}

#[test]
fn durable_transition_ledger_is_complete_and_stable() -> Result<(), String> {
    assert!(TRANSITIONS.starts_with(
        "keep.segment-store.transitions/v1\n\
         crash_id\tphase\toperation\tpre_state\tinterrupted_class\t\
         post_state\trecovery_posture\n"
    ));

    let mut row_count = 0usize;
    for (offset, row) in TRANSITIONS.lines().skip(2).enumerate() {
        let ordinal = offset.checked_add(1).ok_or("transition ordinal overflow")?;
        let expected_id = format!("KEEP-CRASH-{ordinal:03}");
        let mut fields = row.split('\t');
        assert_eq!(fields.next(), Some(expected_id.as_str()));
        assert_eq!(
            fields.count(),
            6,
            "transition {expected_id} must have seven fields"
        );
        row_count = row_count
            .checked_add(1)
            .ok_or("transition count overflow")?;
    }
    assert_eq!(row_count, 35);
    for exact_transition in EXACT_TRANSITIONS {
        assert!(
            TRANSITIONS
                .lines()
                .skip(2)
                .any(|row| row == *exact_transition),
            "imprecise atomic-transition state: {exact_transition}"
        );
    }

    for recovery_class in [
        "reusable-stage",
        "valid-orphan",
        "truncated-tail",
        "corrupt",
        "stale-generation",
        "ambiguity",
    ] {
        assert!(
            format!("{SPECIFICATION}\n{TRANSITIONS}").contains(recovery_class),
            "missing recovery class: {recovery_class}"
        );
    }

    Ok(())
}
