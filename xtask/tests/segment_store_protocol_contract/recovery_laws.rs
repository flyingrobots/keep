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
