//! Living-documentation contract for the public non-durable reference CAS.

const ROOT_README: &str = include_str!("../README.md");
const CHANGELOG: &str = include_str!("../CHANGELOG.md");
const FORMAT_SPEC: &str = include_str!("../docs/formats/flat-chunk-layout-v1/README.md");
const FORMAT_RATIONALE: &str = include_str!("../docs/formats/flat-chunk-layout-v1/rationale.md");
const CORPUS_README: &str = include_str!("../conformance/layout/v1/README.md");
const CAPABILITIES: &str = include_str!("../conformance/golden-file-worldline/v1/capabilities.tsv");
const ARCHITECTURE: &str = include_str!("../docs/architecture/reference-store/README.md");
const ARCHITECTURE_RATIONALE: &str =
    include_str!("../docs/architecture/reference-store/rationale.md");
const RECONSTRUCTION_ERROR_DISPLAY: &str =
    include_str!("../src/reference/reconstruction_error_display.rs");
const RECONSTRUCTION_ERROR: &str = include_str!("../src/reference/reconstruction_error.rs");
const REFERENCE_INGESTION: &str = include_str!("../src/reference/ingestion.rs");
const PUBLISHED_BLOB: &str = include_str!("../src/reference/published_blob.rs");
const RECONSTRUCTION_RECEIPT: &str = include_str!("../src/reference/reconstruction_receipt.rs");
const RANGE_READ_RECEIPT: &str = include_str!("../src/reference/range_read_receipt.rs");
const STAGED_BLOB_TESTS: &str = include_str!("../src/reference/staged_blob_tests.rs");
const GOLDEN_TEST_ENTRYPOINT: &str = include_str!("golden_file_worldline.rs");
const STREAMING_TEST_ENTRYPOINT: &str = include_str!("streaming_cas.rs");
const TEST_SUPPORT: &str = include_str!("support/mod.rs");
const TEST_READERS: &str = include_str!("support/byte_readers.rs");
const TEST_WRITERS: &str = include_str!("support/byte_writers.rs");

#[test]
fn publication_changelog_names_required_chunks_without_assuming_deduplication() {
    assert!(!CHANGELOG.contains("required\n  deduplicated chunk"));
    assert!(!CHANGELOG.contains("required deduplicated chunk"));
    assert!(CHANGELOG.contains("lacks a required chunk"));
}

#[test]
fn integration_fixtures_are_visible_only_within_their_test_crate() {
    assert!(GOLDEN_TEST_ENTRYPOINT.contains("\npub(crate) mod support;"));
    assert!(STREAMING_TEST_ENTRYPOINT.contains("\npub(crate) mod layout_mutation_support;"));
    assert!(STREAMING_TEST_ENTRYPOINT.contains("\npub(crate) mod support;"));
    assert!(!TEST_SUPPORT.contains("\npub use byte_"));
    assert!(!TEST_READERS.contains("\npub struct"));
    assert!(!TEST_READERS.contains("\n    pub fn new"));
    assert!(!TEST_WRITERS.contains("\npub struct"));
    assert!(!TEST_WRITERS.contains("\n    pub fn new"));
    assert!(!TEST_WRITERS.contains("\n    pub fn bytes"));
}

#[test]
fn staged_corruption_laws_share_one_publication_fixture() {
    assert!(STAGED_BLOB_TESTS.contains("\nfn publish_fixture("));
    assert_eq!(
        STAGED_BLOB_TESTS
            .match_indices("let published = publish_fixture(&mut store, source)?;")
            .count(),
        4
    );
    assert!(!STAGED_BLOB_TESTS.contains("let mut first_source = Cursor::new(source);"));
}

#[test]
fn consequential_reference_store_receipts_are_must_use() {
    assert!(PUBLISHED_BLOB.contains("#[must_use ="));
    assert!(RECONSTRUCTION_RECEIPT.contains("#[must_use ="));
    assert!(RANGE_READ_RECEIPT.contains("#[must_use ="));
}

#[test]
fn reconstruction_output_accounting_uses_typed_lengths() {
    let overflow_variant = RECONSTRUCTION_ERROR
        .split_once("WrittenLengthOverflow {")
        .and_then(|(_, remainder)| remainder.split_once("\n    },"))
        .map_or(RECONSTRUCTION_ERROR, |(variant, _)| variant);

    assert!(overflow_variant.contains("bytes_written: BlobLength"));
    assert!(!overflow_variant.contains("bytes_written: u64"));
}

#[test]
fn ingestion_read_width_is_bound_to_the_registered_minimum_chunk_length() {
    assert!(
        REFERENCE_INGESTION
            .contains("assert!(read_buffer_bytes!() <= FastCdc::MINIMUM_CHUNK_LENGTH.get());"),
        "the one-boundary feed law must be enforced at compile time"
    );
}

#[test]
fn reconstruction_error_formatters_stay_below_the_hard_function_limit() {
    assert!(
        !RECONSTRUCTION_ERROR_DISPLAY.contains("enum DisplayGroup"),
        "reconstruction diagnostics must not route variants indirectly"
    );
    assert!(
        !RECONSTRUCTION_ERROR_DISPLAY.contains("Err(fmt::Error)"),
        "reconstruction diagnostics must not reject routed variants"
    );
    let lines: Vec<_> = RECONSTRUCTION_ERROR_DISPLAY.lines().collect();
    let mut discovered = false;
    for (start, line) in lines.iter().enumerate() {
        let Some(name) = rust_function_name(line) else {
            continue;
        };
        discovered = true;
        let indentation: String = line
            .chars()
            .take_while(|character| character.is_whitespace())
            .collect();
        let terminator = format!("{indentation}}}");
        let terminator_offset = lines
            .iter()
            .skip(start)
            .skip(1)
            .position(|candidate| *candidate == terminator);
        assert!(
            matches!(terminator_offset, Some(offset) if offset <= 58),
            "{name} exceeds the 60-line hard limit"
        );
    }
    assert!(discovered, "no reconstruction formatter was measured");
}

fn rust_function_name(line: &str) -> Option<&str> {
    let signature = line.trim_start();
    let signature = ["pub ", "pub(crate) ", "pub(super) "]
        .into_iter()
        .find_map(|visibility| signature.strip_prefix(visibility))
        .unwrap_or(signature);
    let signature = signature.strip_prefix("const ").unwrap_or(signature);
    signature
        .strip_prefix("fn ")
        .map(|function| function.split_once('(').map_or(function, |(name, _)| name))
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
    assert!(!RANGE_READ_RECEIPT.contains("Returns the verified canonical layout identity used."));
    assert!(
        RANGE_READ_RECEIPT.contains("Returns the canonical identity of the admitted layout used.")
    );
}

#[test]
fn shipped_public_reads_and_bounded_ingest_capabilities_are_required() {
    assert!(CAPABILITIES.contains("keep.content.exact-public-read/v1\trequired\tM2\t13\t"));
    assert!(CAPABILITIES.contains("keep.ingest.bounded-stream/v1\trequired\tM2\t13\t"));
    assert!(CAPABILITIES.contains("keep.range.minimal-overlap/v1\trequired\tM2\t11\t"));
}
