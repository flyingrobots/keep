//! Written-policy regression evidence for bounded repository processes.

const BOUNDED_PROCESS: &str = include_str!("../src/bounded_process.rs");
const BOUNDED_PROCESS_CAPTURE: &str = include_str!("../src/bounded_process/capture.rs");
const BOUNDED_PROCESS_CAPTURE_LIMIT: &str = include_str!("../src/bounded_process/capture_limit.rs");
const BOUNDED_PROCESS_ERROR: &str = include_str!("../src/bounded_process/error.rs");
const BOUNDED_PROCESS_INPUT: &str = include_str!("../src/bounded_process/input.rs");
const BOUNDED_PROCESS_INTERRUPT: &str = include_str!("../src/bounded_process/interrupt.rs");
const BOUNDED_PROCESS_GROUP: &str = include_str!("../src/bounded_process/process_group.rs");
const BOUNDED_PROCESS_GROUP_TESTS: &str =
    include_str!("../src/bounded_process/process_group/tests.rs");
const BOUNDED_PROCESS_READER: &str = include_str!("../src/bounded_process/reader.rs");
const BOUNDED_PROCESS_TESTS: &str = include_str!("../src/bounded_process/tests.rs");
const CONFORMANCE_B3SUM: &str = include_str!("../src/protocol_conformance/external_digest.rs");
const EXTERNAL_DIGEST: &str = include_str!("../src/external_digest.rs");
const GOLDEN_B3SUM: &str = include_str!("../src/golden_file_worldline/b3sum_oracle.rs");
const REPOSITORY_FIXTURE: &str = include_str!("../src/repository_fixture.rs");
const GIT_INVENTORY_ERROR: &str = include_str!("../src/git_inventory/error.rs");
const GIT_PATH_STREAM: &str = include_str!("../src/git_inventory/path_stream.rs");
const GIT_PROCESS: &str = include_str!("../src/git_inventory/process.rs");
const SOURCE_PURE_RUST_TESTS: &str = include_str!("../src/source_structure/pure_rust_tests.rs");

#[test]
fn sanitized_git_fixture_has_one_process_authority() {
    let definitions = REPOSITORY_FIXTURE.matches("fn run_git(").count()
        + SOURCE_PURE_RUST_TESTS.matches("fn run_git(").count();
    assert_eq!(definitions, 1);
}

#[test]
fn external_digest_has_one_process_authority() {
    let process_authorities = CONFORMANCE_B3SUM.matches("Command::new(").count()
        + EXTERNAL_DIGEST.matches("Command::new(").count()
        + GOLDEN_B3SUM.matches("Command::new(").count();
    assert_eq!(process_authorities, 1);
    assert!(EXTERNAL_DIGEST.contains("capture_with_input_limits("));
    assert!(!CONFORMANCE_B3SUM.contains("Command::new("));
    assert!(!GOLDEN_B3SUM.contains("Command::new("));
    assert!(!GOLDEN_B3SUM.contains(".wait()"));
    assert!(!GOLDEN_B3SUM.contains(".write_all("));
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
fn repository_git_fixtures_use_the_bounded_process_layer() {
    assert!(REPOSITORY_FIXTURE.contains("bounded_process::status("));
    assert!(REPOSITORY_FIXTURE.contains("GIT_FIXTURE_DEADLINE"));
    assert!(!REPOSITORY_FIXTURE.contains(".output()"));
}

#[test]
fn repository_git_fixtures_clear_the_ambient_environment() {
    assert!(REPOSITORY_FIXTURE.contains(".env_clear()"));
    assert!(REPOSITORY_FIXTURE.contains("env::var_os(\"PATH\")"));
    assert!(REPOSITORY_FIXTURE.contains(".env(\"PATH\""));
    assert!(REPOSITORY_FIXTURE.contains(".env(\"LC_ALL\", \"C\")"));
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
fn bounded_process_capture_documents_every_exported_contract() -> Result<(), String> {
    require_docs(
        BOUNDED_PROCESS,
        &["pub(crate) struct ProcessOutput", "pub(crate) fn status("],
    )?;
    require_docs(
        BOUNDED_PROCESS_CAPTURE,
        &[
            "pub(crate) fn capture(",
            "pub(crate) fn capture_with(",
            "pub(crate) fn capture_with_input_limits(",
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
    )
}

#[test]
fn bounded_process_support_documents_every_exported_contract() -> Result<(), String> {
    require_docs(BOUNDED_PROCESS_INPUT, &["pub(super) fn write_input("])?;
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
    )
}

#[test]
fn repository_process_adapters_document_every_exported_contract() -> Result<(), String> {
    require_docs(
        EXTERNAL_DIGEST,
        &[
            "pub(crate) enum ExternalDigestError",
            "    Environment {",
            "    Process {",
            "    DiagnosticEncoding {",
            "    Failed {",
            "    UnexpectedDiagnostic,",
            "    Width {",
            "pub(crate) fn b3sum(",
        ],
    )?;
    require_docs(REPOSITORY_FIXTURE, &["pub(crate) fn run_git("])?;
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
    require_docs(GIT_PROCESS, &["pub(crate) fn paths_with("])
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
