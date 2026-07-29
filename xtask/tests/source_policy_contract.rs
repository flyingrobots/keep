//! Written-policy regression evidence for the executable source-size law.

const RUST_STANDARDS: &str = include_str!("../../docs/Rust Standards.md");
const BOUNDED_PROCESS: &str = include_str!("../src/bounded_process.rs");
const BOUNDED_PROCESS_CAPTURE: &str = include_str!("../src/bounded_process/capture.rs");
const BOUNDED_PROCESS_CAPTURE_LIMIT: &str = include_str!("../src/bounded_process/capture_limit.rs");
const BOUNDED_PROCESS_ERROR: &str = include_str!("../src/bounded_process/error.rs");
const BOUNDED_PROCESS_INTERRUPT: &str = include_str!("../src/bounded_process/interrupt.rs");
const BOUNDED_PROCESS_GROUP: &str = include_str!("../src/bounded_process/process_group.rs");
const BOUNDED_PROCESS_GROUP_TESTS: &str =
    include_str!("../src/bounded_process/process_group/tests.rs");
const BOUNDED_PROCESS_READER: &str = include_str!("../src/bounded_process/reader.rs");
const BOUNDED_PROCESS_TESTS: &str = include_str!("../src/bounded_process/tests.rs");
const DOCUMENTATION_TEST_REPOSITORY: &str =
    include_str!("../src/documentation_integrity/corpus/test_repository.rs");
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
fn git_inventory_uses_the_deadline_bounded_process_layer() {
    assert!(GIT_PROCESS.contains("const GIT_DEADLINE: Duration"));
    assert!(GIT_PROCESS.contains("bounded_process::capture_with_limits("));
}

#[test]
fn captured_process_keeps_the_group_leader_until_reader_collection_finishes()
-> Result<(), &'static str> {
    let (_, after_signature) = BOUNDED_PROCESS_CAPTURE
        .split_once("    fn finish(")
        .ok_or("captured process must retain its finish boundary")?;
    let (body, _) = after_signature
        .split_once("\n    fn cleanup_readers")
        .ok_or("captured process finish boundary must remain inspectable")?;
    let wait = body
        .find("wait_for_child")
        .ok_or("captured process must reap its child")?;

    for operation in [
        "self.stdout.receive",
        "self.stderr.receive",
        "self.stdout.join",
        "self.stderr.join",
    ] {
        let position = body
            .find(operation)
            .ok_or("captured process must collect and join both output streams")?;
        assert!(position < wait, "child wait precedes {operation}");
    }
    assert!(
        !body
            .get(wait..)
            .unwrap_or_default()
            .contains("cleanup_process"),
        "cleanup may group-kill after the child ownership lifetime ends"
    );
    Ok(())
}

#[test]
fn descendant_cleanup_uses_disconnect_evidence_instead_of_elapsed_time() {
    assert!(!BOUNDED_PROCESS_GROUP_TESTS.contains("descendant_survived_cleanup"));
    assert!(!BOUNDED_PROCESS_GROUP_TESTS.contains("Duration::from_millis(500)"));
    assert!(BOUNDED_PROCESS_GROUP_TESTS.contains("require_descendant_disconnect"));
}

#[test]
fn documentation_git_fixtures_use_the_bounded_process_layer() {
    assert!(DOCUMENTATION_TEST_REPOSITORY.contains("bounded_process::status("));
    assert!(DOCUMENTATION_TEST_REPOSITORY.contains("GIT_FIXTURE_DEADLINE"));
    assert!(!DOCUMENTATION_TEST_REPOSITORY.contains(".output()"));
}

#[test]
fn process_fixtures_do_not_write_to_rust_stdout() {
    assert!(!BOUNDED_PROCESS_TESTS.contains("io::stdout()"));
}

#[test]
fn process_fixtures_isolate_git_from_host_configuration() {
    assert_eq!(
        BOUNDED_PROCESS_TESTS
            .matches("Command::new(\"git\")")
            .count(),
        1
    );
    assert!(BOUNDED_PROCESS_TESTS.contains(".env_clear()"));
    assert!(BOUNDED_PROCESS_TESTS.contains(".env(\"PATH\""));
    assert!(BOUNDED_PROCESS_TESTS.contains(".env(\"GIT_CONFIG_NOSYSTEM\", \"1\")"));
    assert!(BOUNDED_PROCESS_TESTS.contains(".env(\"GIT_CONFIG_GLOBAL\""));
    assert!(BOUNDED_PROCESS_TESTS.contains("--template="));
    assert!(!BOUNDED_PROCESS_TESTS.contains("template.display()"));
}

#[test]
fn repository_process_boundaries_document_every_exported_contract() -> Result<(), String> {
    require_docs(
        BOUNDED_PROCESS,
        &["pub(crate) struct ProcessOutput", "pub(crate) fn status("],
    )?;
    require_docs(
        BOUNDED_PROCESS_CAPTURE,
        &[
            "pub(crate) fn capture(",
            "pub(crate) fn capture_with(",
            "pub(crate) fn capture_with_limits(",
        ],
    )?;
    require_docs(
        BOUNDED_PROCESS_CAPTURE_LIMIT,
        &[
            "pub(crate) struct CaptureLimits",
            "    pub(crate) const fn new(",
        ],
    )?;
    require_docs(
        BOUNDED_PROCESS_ERROR,
        &[
            "pub(crate) enum ProcessError",
            "    Additional {",
            "    Cleanup {",
            "    Io {",
            "    Interrupted {",
            "    MissingStream {",
            "    OutputLimit {",
            "    ReaderPanic {",
            "    Timeout {",
            "    pub(crate) fn is_not_found(",
        ],
    )?;
    require_docs(
        BOUNDED_PROCESS_INTERRUPT,
        &[
            "pub(super) struct InterruptGuard",
            "    pub(super) fn begin(",
            "    pub(super) fn refusal(",
        ],
    )?;
    require_docs(
        BOUNDED_PROCESS_GROUP,
        &[
            "pub(super) struct ProcessGroup",
            "    pub(super) fn for_child(",
            "    pub(super) fn terminate(",
        ],
    )?;
    require_docs(
        BOUNDED_PROCESS_READER,
        &[
            "pub(super) struct ReaderWorker",
            "    pub(super) fn start(",
            "    pub(super) fn receive(",
            "    pub(super) fn join(",
        ],
    )?;
    require_docs(
        GIT_INVENTORY_ERROR,
        &[
            "pub(crate) enum GitOutputUnit",
            "    Bytes,",
            "    Items,",
            "pub(crate) enum GitInventoryError",
            "    DuplicatePath(",
            "    EmptyPath {",
            "    Failed {",
            "    DiagnosticEncoding {",
            "    OutputBound {",
            "    OutputFraming {",
            "    Process {",
            "    Run {",
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
            .rev()
            .find(|line| !line.trim_start().starts_with("#["))
            .is_some_and(|line| line.trim_start().starts_with("///"));
        if !documented {
            return Err(format!("missing rustdoc for `{declaration}`"));
        }
    }
    Ok(())
}
