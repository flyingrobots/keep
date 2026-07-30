//! Bounded and explicit recovery laws.

use super::SPECIFICATION;

#[test]
fn recovery_inventory_is_bounded_before_names_are_retained() {
    for required in [
        "`MAX_RECOVERY_INVENTORY_ENTRY_COUNT` | `2,097,152`",
        "recovery counts entries",
        "before retaining or sorting their names",
        "observed-at-least",
        "`2,097,153`",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing bounded recovery-inventory law: {required}"
        );
    }
}

#[test]
fn recovery_admits_only_the_protocol_created_stage_pool_duplicate() {
    for required in [
        "The sole admissible duplicate digest",
        "one fixed staging name and its exact\n\
         digest-derived pool name",
        "complete byte-for-byte verification",
        "Any third name",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing staging/pool duplicate law: {required}"
        );
    }
}

#[test]
fn durable_stage_recovery_can_complete_only_immutable_pool_publication() {
    for required in [
        "## Complete a durable stage",
        "reverifies and resynchronizes the complete staged\n\
         artifact",
        "reuses `KEEP-CRASH-008`–`012`",
        "catalog completion reuses\n\
         `KEEP-CRASH-016`–`020`",
        "returns a\n\
         valid-orphan receipt",
        "never creates or finalizes a publication head",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing durable-stage completion law: {required}"
        );
    }
}

#[test]
fn next_head_finalization_requires_one_exact_transition_and_durable_receipt() {
    for required in [
        "## Leftover next head",
        "complete transitive catalog view",
        "`plan_recovery_next_head_finalization` refuses a mismatched\n\
         snapshot",
        "generation one over an uninitialized root",
        "expected exact successor",
        "`execute_recovery_next_head_finalization` revalidates durable current state",
        "synchronized and reverified before it atomically replaces `HEAD`",
        "requires `head.next` to be absent and skips replacement",
        "`FilesystemRecoveryNextHeadFinalizer` binds this port",
        "`RecoveryNextHeadFinalizationReceipt`",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing next-head finalization law: {required}"
        );
    }
}

#[test]
fn truncated_stage_discard_fingerprint_has_one_preimage() {
    assert!(
        SPECIFICATION.contains("framed_blake3_v1(ASCII(\"KEEP:RECOVERY:STAGE\\0\"), stage_bytes)"),
        "truncated-stage discard fingerprint lacks its exact preimage"
    );
}

#[test]
fn discard_fingerprints_refuse_oversized_evidence_before_hashing() {
    for required in [
        "`current.seg` uses `MAX_SEGMENT_LENGTH`",
        "`current.cat` uses `MAX_CATALOG_LENGTH`",
        "`head.next` uses `PUBLICATION_HEAD_LENGTH`",
        "reads at most the selected limit plus one byte",
        "before any discard fingerprint is admitted",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing discard input bound: {required}"
        );
    }
}

#[test]
fn discard_receipts_follow_the_actual_stage_parent_directory() {
    for required in [
        "`current.seg` and `current.cat` select `staging`",
        "`KEEP-CRASH-027` and `KEEP-CRASH-028`",
        "`head.next` selects the store root",
        "`head.next` selects the store root, using `KEEP-CRASH-029` and\n  `KEEP-CRASH-030`",
        "Only synchronization of the\n\
         selected parent directory",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing stage-parent discard law: {required}"
        );
    }
}
