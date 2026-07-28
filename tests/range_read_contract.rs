//! Living contract for the public exact byte-range read surface.

const ROOT_README: &str = include_str!("../README.md");
const BLOB_MODULE: &str = include_str!("../src/blob/mod.rs");
const CAPABILITIES: &str = include_str!("../conformance/golden-file-worldline/v1/capabilities.tsv");
const ARCHITECTURE: &str = include_str!("../docs/architecture/reference-store/README.md");
const ARCHITECTURE_RATIONALE: &str =
    include_str!("../docs/architecture/reference-store/rationale.md");
const FORMAT_SPEC: &str = include_str!("../docs/formats/flat-chunk-layout-v1/README.md");
const FORMAT_RATIONALE: &str = include_str!("../docs/formats/flat-chunk-layout-v1/rationale.md");
const RANGE_READ_API: &str = include_str!("../src/reference/range_read.rs");
const RANGE_READ_RECEIPT: &str = include_str!("../src/reference/range_read_receipt.rs");
const RANGE_PLAN_TEST_ENTRYPOINT: &str = include_str!("range_plan.rs");
const RANGE_READ_TEST_ENTRYPOINT: &str = include_str!("range_read.rs");
const RANGE_READ_FAILURE_TEST_ENTRYPOINT: &str = include_str!("range_read_failures.rs");

#[test]
fn range_fixture_modules_are_visible_only_within_their_test_crates() {
    assert!(RANGE_PLAN_TEST_ENTRYPOINT.contains("\npub(crate) mod support;"));
    assert!(RANGE_READ_TEST_ENTRYPOINT.contains("\npub(crate) mod support;"));
    assert!(RANGE_READ_FAILURE_TEST_ENTRYPOINT.contains("\npub(crate) mod support;"));
}

#[test]
fn the_range_receipt_is_consequential_without_overstating_verification() {
    assert!(RANGE_READ_RECEIPT.contains("#[must_use ="));
    assert!(!RANGE_READ_RECEIPT.contains("Returns the verified canonical layout identity used."));
    assert!(!RANGE_READ_RECEIPT.contains("unselected profile"));
    assert!(RANGE_READ_RECEIPT.contains("storage-profile"));
    assert!(RANGE_READ_RECEIPT.contains("boundaries were verified"));
    assert!(
        RANGE_READ_RECEIPT.contains("Returns the canonical identity of the admitted layout used.")
    );
}

#[test]
fn logical_range_coordinates_have_a_named_domain_owner() {
    assert!(
        BLOB_MODULE.contains("This module owns logical blob identity and byte-range coordinates.")
    );
}

#[test]
fn exact_range_reads_are_current_in_contract_rationale_and_roadmap() {
    assert!(ROOT_README.contains("authenticated exact byte-range reads"));
    assert!(ARCHITECTURE.contains("## Exact byte-range reads"));
    assert!(ARCHITECTURE.contains("It does not prove the"));
    assert!(ARCHITECTURE.contains("complete `BlobId`, any unrequested chunk"));
    assert!(
        ARCHITECTURE_RATIONALE.contains("## Why range reads authenticate selected chunks only")
    );
    assert!(FORMAT_SPEC.contains(
        "| `KEEP-LAYOUT-017` | Exact range planning selects only the minimal ordered overlap | Range-plan and instrumented chunk-lookup laws | Implemented in #11 |"
    ));
    assert!(FORMAT_RATIONALE.contains("Exact range planning and reads are"));
    assert!(FORMAT_RATIONALE.contains("implemented in issue #11."));
    assert!(CAPABILITIES.contains("keep.range.minimal-overlap/v1\trequired\tM2\t11\t"));
    for contract in [
        ARCHITECTURE,
        ARCHITECTURE_RATIONALE,
        FORMAT_SPEC,
        RANGE_READ_API,
        RANGE_READ_RECEIPT,
    ] {
        assert!(!contract.contains("unselected profile"));
        assert!(!contract.contains("unselected storage-profile"));
    }
}
