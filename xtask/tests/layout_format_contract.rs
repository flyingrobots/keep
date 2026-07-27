//! Written-contract regression evidence for the flat chunk layout format.

const SPECIFICATION: &str = include_str!("../../docs/formats/flat-chunk-layout-v1/README.md");
const INVALID_LAYOUT_ID_BINARY: &str =
    include_str!("../../conformance/layout/v1/invalid-layout-id-binary.tsv");
const INVALID_LAYOUT_ID_TEXT: &str =
    include_str!("../../conformance/layout/v1/invalid-layout-id-text.tsv");
const MUTATIONS: &str = include_str!("../../conformance/layout/v1/mutations.tsv");

#[test]
fn verified_reconstruction_proves_the_bound_storage_profile() {
    assert!(SPECIFICATION.contains("replayed the"));
    assert!(SPECIFICATION.contains("registered profile's boundary detector"));
    assert!(
        MUTATIONS
            .lines()
            .any(|line| line.starts_with("profile-boundary-mismatch\t"))
    );
}

#[test]
fn mutation_outcomes_follow_first_failure_precedence() {
    assert_eq!(
        expected_mutation_outcome("inserted-duplicate-flags-field"),
        Some("layout.wrong-header-length")
    );
    assert_eq!(
        expected_mutation_outcome("entry-order-swap"),
        Some("layout.gap")
    );
}

#[test]
fn layout_id_coordinates_have_field_complete_refusal_vectors() {
    assert!(INVALID_LAYOUT_ID_BINARY.starts_with(
        "keep.flat-chunk-layout-id.invalid-binary/v1\n\
         case\tbase_case\toperation\toffset\tspan_length\tparameter\t\
         expected_outcome\n"
    ));
    assert!(INVALID_LAYOUT_ID_TEXT.starts_with(
        "keep.flat-chunk-layout-id.invalid-text/v1\n\
         case\tinput_hex\texpected_outcome\n"
    ));
    for case_name in [
        "wrong-identity-magic",
        "unsupported-identity-version",
        "unsupported-layout-codec",
        "plan-length-out-of-bounds",
        "plan-length-not-congruent",
        "plan-length-mismatch",
        "digest-mismatch",
        "truncated-coordinate",
        "trailing-byte",
    ] {
        assert!(contains_case(INVALID_LAYOUT_ID_BINARY, case_name));
    }
    for case_name in [
        "input-too-long",
        "empty-input",
        "wrong-scheme",
        "wrong-kind",
        "malformed-version",
        "unsupported-version",
        "unsupported-codec",
        "unsupported-algorithm",
        "leading-zero-plan-length",
        "signed-plan-length",
        "plan-length-overflow",
        "plan-length-out-of-bounds",
        "plan-length-not-congruent",
        "short-digest",
        "uppercase-digest",
        "nonhex-digest",
        "trailing-field",
        "leading-space",
        "trailing-newline",
    ] {
        assert!(contains_case(INVALID_LAYOUT_ID_TEXT, case_name));
    }
    assert!(
        INVALID_LAYOUT_ID_BINARY
            .lines()
            .skip(2)
            .all(|line| line.split('\t').count() == 7)
    );
    assert!(
        INVALID_LAYOUT_ID_TEXT
            .lines()
            .skip(2)
            .all(|line| line.split('\t').count() == 3)
    );
}

fn contains_case(table: &str, case_name: &str) -> bool {
    table.lines().skip(2).any(|line| {
        line.split('\t')
            .next()
            .is_some_and(|field| field == case_name)
    })
}

fn expected_mutation_outcome(case_name: &str) -> Option<&'static str> {
    MUTATIONS.lines().skip(2).find_map(|line| {
        let mut fields = line.split('\t');
        if fields.next()? != case_name {
            return None;
        }
        fields.nth(7)
    })
}
