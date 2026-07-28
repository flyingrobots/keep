//! Stable crash-transition ledger laws.

use super::{SPECIFICATION, TRANSITIONS};

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
    assert_eq!(row_count, 30);
    for exact_transition in [
        "KEEP-CRASH-009\tsegment\tlink-sealed-stage\t\
         durable-sealed-stage\t\
         valid-sealed-stage-or-valid-orphan-or-ambiguity\t\
         linked-segment-orphan\tverify-no-clobber-link-and-digest",
        "KEEP-CRASH-017\tcatalog\tlink-generation\t\
         durable-catalog-stage\t\
         valid-catalog-stage-or-valid-orphan-or-ambiguity\t\
         linked-catalog-orphan\tverify-no-clobber-link-and-digest",
        "KEEP-CRASH-025\thead\treplace-current-head\t\
         durable-next-head\t\
         valid-next-head-or-published-generation-or-ambiguity\t\
         replaced-current-head\tverify-one-atomic-head",
        "KEEP-CRASH-026\thead\tsync-root-directory\t\
         replaced-current-head\tpublished-generation-or-ambiguity\t\
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
    ] {
        assert!(
            TRANSITIONS
                .lines()
                .skip(2)
                .any(|row| row == exact_transition),
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
