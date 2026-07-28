//! Structural laws for the immutable-segment writer implementation.

const STAGED_SEGMENT: &str = include_str!("../../src/adapters/staged_segment.rs");

#[test]
fn duplicate_admission_uses_a_membership_index_instead_of_a_linear_identity_vector() {
    assert!(
        STAGED_SEGMENT.contains("HashSet<SegmentRecordIdentity>"),
        "writer duplicate admission must use sublinear membership lookup"
    );
    assert!(
        !STAGED_SEGMENT.contains("Vec<SegmentRecordIdentity>"),
        "writer duplicate admission must not scan a linear identity vector"
    );
}
