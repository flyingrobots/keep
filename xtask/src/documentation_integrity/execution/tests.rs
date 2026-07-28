use std::collections::VecDeque;

use crate::bounded_process::ProcessOutput;

use super::{DocumentationError, DocumentationTool, ToolRunner};

struct RecordingRunner {
    calls: Vec<(DocumentationTool, Vec<String>)>,
    outputs: VecDeque<ProcessOutput>,
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
