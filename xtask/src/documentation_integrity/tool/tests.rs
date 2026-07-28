//! This module owns documentation-tool policy regression evidence.

use super::DocumentationTool;
use crate::documentation_integrity::error::DocumentationError;

#[test]
fn every_unreviewed_tool_version_is_refused() {
    for tool in tools() {
        assert!(matches!(
            tool.admit_version("999.0.0"),
            Err(DocumentationError::VersionMismatch {
                program,
                expected,
                ref observed,
            }) if program == tool.program()
                && expected == tool.expected_version()
                && observed == "999.0.0"
        ));
    }
}

#[test]
fn every_reviewed_tool_version_is_admitted_exactly() {
    for tool in tools() {
        assert!(tool.admit_version(tool.expected_version()).is_ok());
        assert!(!tool.install_version().is_empty());
    }
}

#[test]
fn tool_arguments_preserve_the_reviewed_execution_boundary() {
    assert_eq!(
        DocumentationTool::Markdownlint.version_arguments(),
        ["--no-globs", "--version"]
    );
    assert_eq!(
        DocumentationTool::Markdownlint.check_prefix(),
        ["--no-globs", "--"]
    );
    assert_eq!(
        DocumentationTool::Lychee.check_prefix(),
        [
            "--offline",
            "--include-fragments",
            "--no-progress",
            "--format",
            "detailed",
            "--",
        ]
    );
    assert_eq!(
        DocumentationTool::Actionlint.check_prefix(),
        ["-shellcheck=", "-pyflakes="]
    );
}

const fn tools() -> [DocumentationTool; 3] {
    [
        DocumentationTool::Markdownlint,
        DocumentationTool::Lychee,
        DocumentationTool::Actionlint,
    ]
}
