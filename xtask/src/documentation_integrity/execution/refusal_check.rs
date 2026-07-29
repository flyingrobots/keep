//! This module owns executable pinned-tool malformed-input refusal evidence.

use std::fs;
use std::io;

use crate::documentation_integrity::DocumentationError;
use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

use super::{DocumentationTool, ExternalToolRunner};

/// Requires exact pinned-tool refusals for every malformed-input scenario.
pub(super) fn check() -> Result<(), DocumentationError> {
    broken_internal_fragment_is_refused()?;
    invalid_workflow_is_refused()
}

fn broken_internal_fragment_is_refused() -> Result<(), DocumentationError> {
    let directory = TestDirectory::create("broken-fragment")
        .map_err(|source| fixture_error("create broken-fragment directory", source))?;
    fs::write(
        directory.path().join("source.md"),
        "# Source\n\n[Missing](target.md#missing-heading)\n",
    )
    .map_err(|source| fixture_error("write broken-fragment source", source))?;
    fs::write(directory.path().join("target.md"), "# Present heading\n")
        .map_err(|source| fixture_error("write broken-fragment target", source))?;
    let repository_root = RepositoryRoot::open(directory.path())
        .map_err(|source| fixture_error("open broken-fragment repository", source))?;
    let process_directory = repository_root
        .process_directory()
        .map_err(|source| fixture_error("open broken-fragment process directory", source))?;
    let refusal = {
        let mut runner = ExternalToolRunner {
            process_directory: &process_directory,
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
    require_refusal(
        refusal,
        "broken internal fragment",
        "lychee",
        Some(2),
        "Cannot find fragment",
    )?;
    directory
        .close()
        .map_err(|source| fixture_error("remove broken-fragment directory", source))?;
    Ok(())
}

fn invalid_workflow_is_refused() -> Result<(), DocumentationError> {
    let directory = TestDirectory::create("invalid-workflow")
        .map_err(|source| fixture_error("create invalid-workflow directory", source))?;
    fs::create_dir_all(directory.path().join(".github/workflows"))
        .map_err(|source| fixture_error("create invalid workflow directory", source))?;
    fs::write(
        directory.path().join(".github/workflows/invalid.yml"),
        "name: Invalid\non: [push\n",
    )
    .map_err(|source| fixture_error("write invalid workflow", source))?;
    let repository_root = RepositoryRoot::open(directory.path())
        .map_err(|source| fixture_error("open invalid-workflow repository", source))?;
    let process_directory = repository_root
        .process_directory()
        .map_err(|source| fixture_error("open invalid-workflow process directory", source))?;
    let refusal = {
        let mut runner = ExternalToolRunner {
            process_directory: &process_directory,
        };
        super::admit_version(&mut runner, DocumentationTool::Actionlint)?;
        super::run_check(
            &mut runner,
            DocumentationTool::Actionlint,
            &[String::from(".github/workflows/invalid.yml")],
        )
    };
    require_refusal(
        refusal,
        "invalid workflow",
        "actionlint",
        Some(1),
        "could not parse as YAML",
    )?;
    directory
        .close()
        .map_err(|source| fixture_error("remove invalid-workflow directory", source))?;
    Ok(())
}

fn require_refusal(
    refusal: Result<(), DocumentationError>,
    scenario: &'static str,
    program: &'static str,
    code: Option<i32>,
    diagnostic: &str,
) -> Result<(), DocumentationError> {
    match refusal {
        Err(DocumentationError::ToolFailed {
            program: observed_program,
            code: observed_code,
            stdout,
            stderr,
        }) if observed_program == program
            && observed_code == code
            && format!("{stdout}\n{stderr}").contains(diagnostic) =>
        {
            Ok(())
        }
        Err(observed) => Err(DocumentationError::RefusalMismatch {
            scenario,
            observed: Some(Box::new(observed)),
        }),
        Ok(()) => Err(DocumentationError::RefusalMismatch {
            scenario,
            observed: None,
        }),
    }
}

const fn fixture_error(action: &'static str, source: io::Error) -> DocumentationError {
    DocumentationError::RefusalFixture { action, source }
}

#[cfg(test)]
mod tests {
    use super::{DocumentationError, require_refusal};

    #[test]
    fn exact_tool_failure_is_executable_refusal_evidence() {
        let refusal = Err(DocumentationError::ToolFailed {
            program: "lychee",
            code: Some(2),
            stdout: String::from("Cannot find fragment"),
            stderr: String::new(),
        });

        assert!(
            require_refusal(
                refusal,
                "broken internal fragment",
                "lychee",
                Some(2),
                "Cannot find fragment",
            )
            .is_ok()
        );
    }

    #[test]
    fn successful_malformed_input_is_a_typed_evidence_failure() {
        assert!(matches!(
            require_refusal(
                Ok(()),
                "broken internal fragment",
                "lychee",
                Some(2),
                "Cannot find fragment",
            ),
            Err(DocumentationError::RefusalMismatch {
                scenario: "broken internal fragment",
                observed: None,
            })
        ));
    }
}
