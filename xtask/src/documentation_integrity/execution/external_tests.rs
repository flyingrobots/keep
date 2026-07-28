//! This module owns pinned-tool malformed-input refusal evidence.

use std::fs;

use crate::test_directory::TestDirectory;

use super::{DocumentationError, DocumentationTool, ExternalToolRunner};

#[test]
#[ignore = "requires pinned documentation tools installed by the documentation CI job"]
fn broken_internal_fragment_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("broken-fragment")?;
    fs::write(
        directory.path().join("source.md"),
        "# Source\n\n[Missing](target.md#missing-heading)\n",
    )?;
    fs::write(directory.path().join("target.md"), "# Present heading\n")?;
    let refusal = {
        let mut runner = ExternalToolRunner {
            repository_root: directory.path(),
        };
        super::admit_version(&mut runner, DocumentationTool::Markdownlint)?;
        super::admit_version(&mut runner, DocumentationTool::Lychee)?;
        super::run_check(
            &mut runner,
            DocumentationTool::Markdownlint,
            &[String::from("source.md"), String::from("target.md")],
        )?;
        super::run_check(
            &mut runner,
            DocumentationTool::Lychee,
            &[String::from("source.md"), String::from("target.md")],
        )
    };
    assert!(matches!(
        refusal,
        Err(DocumentationError::ToolFailed {
            program: "lychee",
            code: Some(2),
            ref stdout,
            ref stderr,
        }) if format!("{stdout}\n{stderr}").contains("Cannot find fragment")
    ));
    directory.close()?;
    Ok(())
}

#[test]
#[ignore = "requires pinned documentation tools installed by the documentation CI job"]
fn invalid_workflow_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("invalid-workflow")?;
    fs::create_dir_all(directory.path().join(".github/workflows"))?;
    fs::write(
        directory.path().join(".github/workflows/invalid.yml"),
        "name: Invalid\non: [push\n",
    )?;
    let refusal = {
        let mut runner = ExternalToolRunner {
            repository_root: directory.path(),
        };
        super::admit_version(&mut runner, DocumentationTool::Actionlint)?;
        super::run_check(
            &mut runner,
            DocumentationTool::Actionlint,
            &[String::from(".github/workflows/invalid.yml")],
        )
    };
    assert!(matches!(
        refusal,
        Err(DocumentationError::ToolFailed {
            program: "actionlint",
            code: Some(1),
            ref stdout,
            ref stderr,
        }) if format!("{stdout}\n{stderr}").contains("could not parse as YAML")
    ));
    directory.close()?;
    Ok(())
}
