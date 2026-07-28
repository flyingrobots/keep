//! Written-policy regression evidence for the executable source-size law.

const RUST_STANDARDS: &str = include_str!("../../docs/Rust Standards.md");
const BOUNDED_PROCESS: &str = include_str!("../src/bounded_process.rs");
const BOUNDED_PROCESS_CAPTURE: &str = include_str!("../src/bounded_process/capture.rs");
const BOUNDED_PROCESS_ERROR: &str = include_str!("../src/bounded_process/error.rs");
const BOUNDED_PROCESS_TESTS: &str = include_str!("../src/bounded_process/tests.rs");
const GIT_INVENTORY_ERROR: &str = include_str!("../src/git_inventory/error.rs");
const GIT_PATH_STREAM: &str = include_str!("../src/git_inventory/path_stream.rs");
const GIT_PROCESS: &str = include_str!("../src/git_inventory/process.rs");
const REPOSITORY_FILE: &str = include_str!("../src/repository_file.rs");
const SOURCE_STRUCTURE: &str = include_str!("../src/source_structure.rs");

#[test]
fn written_source_limit_matches_the_executable_law() {
    assert!(SOURCE_STRUCTURE.contains("SOURCE_MODULE_HARD_LIMIT_LINES: u64 = 500"));
    assert!(
        RUST_STANDARDS
            .contains("**Tests:** same 500-line absolute maximum; prefer scenario subdivision")
    );
    assert!(RUST_STANDARDS.contains("Count physical lines for deterministic enforcement."));
    assert!(!RUST_STANDARDS.contains("Count nonblank, non-comment lines for enforcement"));
}

#[test]
fn repository_file_admission_declares_its_unix_scope() {
    assert!(REPOSITORY_FILE.contains("intentionally supported only on Unix hosts"));
    assert!(REPOSITORY_FILE.contains("Unix device and inode identity"));
}

#[test]
fn source_scan_revalidates_repository_identity_after_reading() {
    assert_eq!(
        SOURCE_STRUCTURE
            .matches("verify_source_root(&source_root, repository_root)?;")
            .count(),
        2
    );
}

#[test]
fn process_fixtures_do_not_write_to_rust_stdout() {
    assert!(!BOUNDED_PROCESS_TESTS.contains("io::stdout()"));
}

#[test]
fn repository_process_boundaries_document_every_exported_contract() -> Result<(), String> {
    require_docs(
        BOUNDED_PROCESS,
        &["pub(crate) struct ProcessOutput", "pub(crate) fn status("],
    )?;
    require_docs(BOUNDED_PROCESS_CAPTURE, &["pub(crate) fn capture("])?;
    require_docs(
        BOUNDED_PROCESS_ERROR,
        &[
            "pub(crate) enum ProcessError",
            "    Additional {",
            "    Cleanup {",
            "    Io {",
            "    MissingStream {",
            "    OutputLimit {",
            "    ReaderPanic {",
            "    Timeout {",
            "    pub(crate) fn is_not_found(",
        ],
    )?;
    require_docs(
        GIT_INVENTORY_ERROR,
        &[
            "pub(crate) enum GitOutputUnit",
            "    Bytes,",
            "    Items,",
            "pub(crate) enum GitInventoryError",
            "    Cleanup {",
            "    DuplicatePath(",
            "    EmptyPath {",
            "    Failed {",
            "    DiagnosticEncoding {",
            "    OutputBound {",
            "    OutputFraming {",
            "    Pipe {",
            "    Run {",
            "    Worker {",
        ],
    )?;
    require_docs(
        GIT_PATH_STREAM,
        &[
            "pub(super) fn read_paths(",
            "pub(crate) struct GitPath(",
            "    pub(crate) const fn new(",
            "    pub(crate) fn as_bytes(",
        ],
    )?;
    require_docs(GIT_PROCESS, &["pub(crate) fn paths("])
}

fn require_docs(source: &str, declarations: &[&str]) -> Result<(), String> {
    for declaration in declarations {
        let (before, _) = source
            .split_once(declaration)
            .ok_or_else(|| format!("missing declaration `{declaration}`"))?;
        let documented = before
            .lines()
            .next_back()
            .is_some_and(|line| line.trim_start().starts_with("///"));
        if !documented {
            return Err(format!("missing rustdoc for `{declaration}`"));
        }
    }
    Ok(())
}
