//! Written-contract regression evidence for the flat chunk layout format.

const SPECIFICATION: &str = include_str!("../../docs/formats/flat-chunk-layout-v1/README.md");
const RATIONALE: &str = include_str!("../../docs/formats/flat-chunk-layout-v1/rationale.md");
const FORMAT_REGISTRY: &str = include_str!("../../docs/formats/README.md");
const CONFORMANCE_GUIDE: &str = include_str!("../../conformance/layout/v1/README.md");
const LAYOUT_DECODE_ERROR_DISPLAY: &str =
    include_str!("../../src/adapters/layout_decode_error_display.rs");
const LAYOUT_MUTATION_TESTS: &str = include_str!("../../tests/layout_mutations.rs");
const LAYOUT_TEST_HELPER_CALLERS: &str = concat!(
    include_str!("../../tests/layout_decode.rs"),
    include_str!("../../tests/layout_id.rs"),
    include_str!("../../tests/layout_mutations.rs"),
    include_str!("../../tests/layout_mutations/support.rs"),
    include_str!("../../tests/layout_oracle.rs"),
    include_str!("../../tests/layout_oracle/support.rs"),
    include_str!("../../tests/layout_properties.rs"),
    include_str!("../../tests/layout_record.rs"),
);
const INVALID_LAYOUT_ID_BINARY: &str =
    include_str!("../../conformance/layout/v1/invalid-layout-id-binary.tsv");
const INVALID_LAYOUT_ID_TEXT: &str =
    include_str!("../../conformance/layout/v1/invalid-layout-id-text.tsv");
const MUTATIONS: &str = include_str!("../../conformance/layout/v1/mutations.tsv");

#[test]
fn layout_decode_error_formatter_stays_below_the_hard_function_limit() -> Result<(), &'static str> {
    let (_, after_signature) = LAYOUT_DECODE_ERROR_DISPLAY
        .split_once("    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {")
        .ok_or("display implementation must retain its formatter")?;
    let (body, _) = after_signature
        .split_once("\n    }\n}\n\nfn ")
        .ok_or("display formatter must remain a directly inspectable function")?;
    assert!(
        body.lines().count() <= 59,
        "LayoutDecodeError::fmt exceeds the 60-line hard limit"
    );
    Ok(())
}

#[test]
fn layout_mutation_classifiers_name_every_unclassified_variant() {
    assert!(
        !LAYOUT_MUTATION_TESTS.contains("_ => None"),
        "layout mutation classifiers must not hide future error variants"
    );
}

#[test]
fn layout_corpus_plumbing_has_one_shared_owner() {
    for duplicate in [
        "\nfn detect_spans(",
        "\nfn source_bytes(",
        "\nfn record_fixture(",
        "\nfn field(",
        "\nfn field_unchecked(",
        "\nfn fixture(",
        "\nfn layout_field(",
        "\nfn layout_id(",
        "\nfn layout_id_binary(",
        "\nfn require_error",
    ] {
        assert!(
            !LAYOUT_TEST_HELPER_CALLERS.contains(duplicate),
            "layout corpus helper remains duplicated: {duplicate}"
        );
    }
}

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
fn format_registry_reports_flat_layout_as_implemented() {
    assert!(
        FORMAT_REGISTRY
            .contains("Implemented through verified reconstruction in issues #10 and #13"),
        "format registry understates the implemented flat-layout proof boundary"
    );
}

#[test]
fn format_registry_reports_the_segment_store_implementation_boundary() {
    const EXPECTED_ROW: &str = "\
| [Durable Segment Store v1](segment-store-v1/README.md) | \
`keep.segment-store/v1` | Specified in issue #14; segment I/O implemented in \
issue #15; publication and recovery remain in issues #16–#17 | \
[Golden corpus](../../conformance/segment-store/v1/README.md) |";

    assert!(
        FORMAT_REGISTRY.lines().any(|line| line == EXPECTED_ROW),
        "format registry lost the exact durable segment-store implementation boundary"
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
    assert_eq!(
        expected_mutation_outcome("blob-logical-length-mismatch"),
        Some("layout.empty-blob-has-entries")
    );
}

#[test]
fn format_prose_uses_unambiguous_compound_phrases() {
    assert!(!CONFORMANCE_GUIDE.contains("same exact"));
    assert!(!RATIONALE.contains("one byte representation"));
    assert!(!SPECIFICATION.contains("future bounded hierarchical codec"));
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
        "leading-zero-version",
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
