//! Living-documentation contract for the public non-durable reference CAS.

const ROOT_README: &str = include_str!("../README.md");
const FORMAT_SPEC: &str = include_str!("../docs/formats/flat-chunk-layout-v1/README.md");
const FORMAT_RATIONALE: &str = include_str!("../docs/formats/flat-chunk-layout-v1/rationale.md");
const CORPUS_README: &str = include_str!("../conformance/layout/v1/README.md");
const CAPABILITIES: &str = include_str!("../conformance/golden-file-worldline/v1/capabilities.tsv");
const ARCHITECTURE: &str = include_str!("../docs/architecture/reference-store/README.md");
const ARCHITECTURE_RATIONALE: &str =
    include_str!("../docs/architecture/reference-store/rationale.md");
const RECONSTRUCTION_ERROR_DISPLAY: &str =
    include_str!("../src/reference/reconstruction_error_display.rs");

#[test]
fn reconstruction_error_formatters_stay_below_the_hard_function_limit() {
    for function in RECONSTRUCTION_ERROR_DISPLAY.split("\nfn ").skip(1) {
        let body = function
            .split_once("\n}\n\n")
            .map_or(function, |(body, _)| body);
        let name = function.split_once('(').map_or(function, |(name, _)| name);
        assert!(
            body.lines().count() <= 59,
            "{name} exceeds the 60-line hard limit"
        );
    }
}

#[test]
fn public_reference_cas_is_reported_as_implemented_without_a_durability_claim() {
    assert!(
        ROOT_README
            .contains("[non-durable reference CAS](docs/architecture/reference-store/README.md)")
    );
    assert!(!ROOT_README.contains("does **not** expose ingestion, layouts, or physical storage"));
    assert!(!ROOT_README.contains("but ingestion and\nstorage do not"));
    assert!(
        !ROOT_README
            .contains("next format\nboundary without claiming that its implementation exists")
    );
    assert!(
        ARCHITECTURE.contains("Process death may erase every pre-commit and post-commit state")
    );
    assert!(ARCHITECTURE_RATIONALE.contains("## Alternatives considered"));
}

#[test]
fn verified_layout_reconstruction_is_current_in_spec_rationale_and_corpus() {
    assert!(FORMAT_SPEC.contains(
        "| `KEEP-LAYOUT-016` | Verified reconstruction reproduces the declared spans under the bound storage profile | Verification state and profile-boundary mutation | Implemented in #13 |"
    ));
    assert!(
        FORMAT_RATIONALE
            .contains("adapter implements ingestion and verified reconstruction in issue #13.")
    );
    assert!(CORPUS_README.contains("Issue [#13](https://github.com/flyingrobots/keep/issues/13)"));
    assert!(
        CORPUS_README.contains("public verification boundary that compares actual chunk bytes")
    );
}

#[test]
fn shipped_public_read_and_bounded_ingest_capabilities_are_required() {
    assert!(CAPABILITIES.contains("keep.content.exact-public-read/v1\trequired\tM2\t13\t"));
    assert!(CAPABILITIES.contains("keep.ingest.bounded-stream/v1\trequired\tM2\t13\t"));
}
