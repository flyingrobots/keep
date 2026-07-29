use std::collections::{BTreeMap, VecDeque};
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::bounded_process::ProcessOutput;
use crate::documentation_integrity::corpus::{SourceCorpus, test_repository::run_git};
use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

use super::corpus_guard::CorpusGuardedRunner;
use super::{
    DirectoryToolRunner, DocumentationError, DocumentationTool, ToolRunner, documentation_command,
};

struct RecordingRunner {
    calls: Vec<(DocumentationTool, Vec<String>)>,
    outputs: VecDeque<ProcessOutput>,
}

struct ReplacingRunner {
    repository: PathBuf,
}

#[test]
fn documentation_tools_receive_only_reviewed_environment() {
    let path = OsString::from("/reviewed/tools");
    let command = documentation_command(DocumentationTool::Markdownlint, &[], &path);
    let observed = command
        .get_envs()
        .map(|(name, value)| (name.to_owned(), value.map(OsString::from)))
        .collect::<BTreeMap<_, _>>();
    let expected = BTreeMap::from([
        (OsString::from("LC_ALL"), Some(OsString::from("C"))),
        (OsString::from("PATH"), Some(path)),
    ]);

    assert_eq!(observed, expected);
    assert_eq!(
        DocumentationError::EnvironmentUnavailable("PATH").to_string(),
        "documentation tool environment variable `PATH` is unavailable"
    );
}

#[test]
fn admitted_tools_run_with_exact_arguments_and_silent_success() {
    let mut runner = RecordingRunner::new([
        version(DocumentationTool::Markdownlint),
        version(DocumentationTool::Lychee),
        success(),
        success(),
        version(DocumentationTool::Actionlint),
        success(),
    ]);
    let markdown = [String::from("README.md")];
    let workflows = [String::from(".github/workflows/ci.yml")];

    assert!(super::run_with(&mut runner, &markdown, &workflows).is_ok());
    assert_eq!(runner.calls.len(), 6);
    assert_eq!(
        runner.calls.get(2),
        Some(&(
            DocumentationTool::Markdownlint,
            vec![
                String::from("--no-globs"),
                String::from("--"),
                String::from("README.md")
            ]
        ))
    );
    assert_eq!(
        runner.calls.get(5),
        Some(&(
            DocumentationTool::Actionlint,
            vec![
                String::from("-shellcheck="),
                String::from("-pyflakes="),
                String::from(".github/workflows/ci.yml")
            ]
        ))
    );
}

#[test]
fn link_check_runs_after_markdownlint_returns_nonzero() {
    let mut runner = RecordingRunner::new([
        version(DocumentationTool::Markdownlint),
        version(DocumentationTool::Lychee),
        failure(b"lint", b""),
        success(),
    ]);

    let result = super::run_with(&mut runner, &[String::from("README.md")], &[]);

    assert!(matches!(
        result,
        Err(DocumentationError::ToolFailed {
            program: "markdownlint-cli2",
            code: Some(1),
            ref stdout,
            ref stderr,
        }) if stdout == "lint" && stderr.is_empty()
    ));
    assert_eq!(runner.calls.len(), 4);
    assert_eq!(
        runner.calls.get(3).map(|call| call.0),
        Some(DocumentationTool::Lychee)
    );
}

#[test]
fn simultaneous_markdown_failures_are_both_reported() -> Result<(), &'static str> {
    let mut runner = RecordingRunner::new([
        version(DocumentationTool::Markdownlint),
        version(DocumentationTool::Lychee),
        failure(b"lint refusal", b""),
        failure(b"link refusal", b""),
    ]);

    let result = super::run_with(&mut runner, &[String::from("README.md")], &[]);
    let diagnostic = result
        .err()
        .ok_or("both Markdown checks unexpectedly succeeded")?
        .to_string();

    assert!(diagnostic.contains("lint refusal"));
    assert!(diagnostic.contains("link refusal"));
    Ok(())
}

#[test]
fn unreviewed_version_stops_before_tool_execution() {
    let mut runner = RecordingRunner::new([ProcessOutput {
        code: Some(0),
        succeeded: true,
        stdout: b"markdownlint-cli2 v999.0.0\n".to_vec(),
        stderr: Vec::new(),
    }]);

    let result = super::run_with(&mut runner, &[], &[]);

    assert!(matches!(
        result,
        Err(DocumentationError::VersionMismatch {
            program: "markdownlint-cli2",
            expected: "markdownlint-cli2 v0.23.2 (markdownlint v0.41.1)",
            ref observed,
        }) if observed == "markdownlint-cli2 v999.0.0"
    ));
    assert_eq!(runner.calls.len(), 1);
}

#[test]
fn corpus_guard_executes_against_the_admitted_source_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-transient-source")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet", "--template="])?;
    fs::create_dir(root.join("docs"))?;
    fs::write(root.join("docs/selected.md"), "# Original\n")?;
    fs::write(
        root.join(".markdownlint-cli2.yaml"),
        "config:\n  MD013: false\n",
    )?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    let corpus = SourceCorpus::markdown(&repository_root, &process_directory)?;
    let corpora = [&corpus];
    let replacing = ReplacingRunner {
        repository: root.to_owned(),
    };
    let mut runner =
        CorpusGuardedRunner::new(replacing, &process_directory, &repository_root, &corpora)?;

    let output = runner.capture(DocumentationTool::Markdownlint, &[])?;

    assert_eq!(output.stdout, b"# Original\n");
    runner.close()?;
    drop(corpus);
    directory.close()?;
    Ok(())
}

impl RecordingRunner {
    fn new(outputs: impl IntoIterator<Item = ProcessOutput>) -> Self {
        Self {
            calls: Vec::new(),
            outputs: outputs.into_iter().collect(),
        }
    }
}

impl ToolRunner for RecordingRunner {
    fn capture(
        &mut self,
        tool: DocumentationTool,
        arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError> {
        self.calls.push((tool, arguments.to_vec()));
        self.outputs
            .pop_front()
            .ok_or(DocumentationError::RepositoryContract {
                path: "test runner",
                requirement: "one output exists for every expected call",
            })
    }
}

impl DirectoryToolRunner for ReplacingRunner {
    fn capture_in(
        &mut self,
        execution_root: &RepositoryRoot,
        _process_directory: &crate::repository_file::RepositoryProcessDirectory,
        _tool: DocumentationTool,
        _arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError> {
        let selected = self.repository.join("docs");
        let retained = self.repository.join("retained-docs");
        fs::rename(&selected, &retained)
            .map_err(|source| fixture_io("retain selected directory", source))?;
        fs::create_dir(&selected)
            .map_err(|source| fixture_io("create substitute directory", source))?;
        fs::write(selected.join("selected.md"), "# Substitute\n")
            .map_err(|source| fixture_io("write substitute source", source))?;
        let observed = read_source(execution_root, Path::new("docs/selected.md"))?;
        fs::remove_dir_all(&selected)
            .map_err(|source| fixture_io("remove substitute directory", source))?;
        fs::rename(&retained, &selected)
            .map_err(|source| fixture_io("restore selected directory", source))?;
        Ok(ProcessOutput {
            code: Some(0),
            succeeded: true,
            stdout: observed,
            stderr: Vec::new(),
        })
    }
}

fn read_source(
    repository_root: &RepositoryRoot,
    relative: &Path,
) -> Result<Vec<u8>, DocumentationError> {
    let mut file = repository_root.open_file(relative).map_err(|_| {
        fixture_io(
            "open execution source",
            std::io::Error::other("open failed"),
        )
    })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|source| fixture_io("read execution source", source))?;
    Ok(bytes)
}

fn fixture_io(requirement: &'static str, source: std::io::Error) -> DocumentationError {
    DocumentationError::Inspect {
        corpus: "test",
        path: requirement.to_owned(),
        source,
    }
}

fn version(tool: DocumentationTool) -> ProcessOutput {
    ProcessOutput {
        code: Some(0),
        succeeded: true,
        stdout: format!("{}\n", tool.expected_version()).into_bytes(),
        stderr: Vec::new(),
    }
}

const fn success() -> ProcessOutput {
    ProcessOutput {
        code: Some(0),
        succeeded: true,
        stdout: Vec::new(),
        stderr: Vec::new(),
    }
}

fn failure(stdout: &[u8], stderr: &[u8]) -> ProcessOutput {
    ProcessOutput {
        code: Some(1),
        succeeded: false,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}
