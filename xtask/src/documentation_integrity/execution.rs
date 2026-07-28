//! This module owns bounded execution of admitted documentation tools.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::bounded_process::{self, ProcessOutput};

use super::error::DocumentationError;
use super::tool::DocumentationTool;

const TOOL_DEADLINE: Duration = Duration::from_mins(2);

trait ToolRunner {
    fn capture(
        &mut self,
        tool: DocumentationTool,
        arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError>;
}

struct ExternalToolRunner<'a> {
    repository_root: &'a Path,
}

pub(super) fn run(
    repository_root: &Path,
    markdown: &[String],
    workflows: &[String],
) -> Result<(), DocumentationError> {
    run_with(
        &mut ExternalToolRunner { repository_root },
        markdown,
        workflows,
    )
}

fn run_with(
    runner: &mut impl ToolRunner,
    markdown: &[String],
    workflows: &[String],
) -> Result<(), DocumentationError> {
    admit_version(runner, DocumentationTool::Markdownlint)?;
    admit_version(runner, DocumentationTool::Lychee)?;
    let lint = run_check(runner, DocumentationTool::Markdownlint, markdown);
    let links = run_check(runner, DocumentationTool::Lychee, markdown);
    lint?;
    links?;
    admit_version(runner, DocumentationTool::Actionlint)?;
    run_check(runner, DocumentationTool::Actionlint, workflows)
}

fn admit_version(
    runner: &mut impl ToolRunner,
    tool: DocumentationTool,
) -> Result<(), DocumentationError> {
    let arguments = text_arguments(tool.version_arguments());
    let output = runner.capture(tool, &arguments)?;
    let stdout = String::from_utf8(output.stdout).map_err(|source| {
        DocumentationError::ToolOutputEncoding {
            program: tool.program(),
            stream: "version stdout",
            source,
        }
    })?;
    let observed = stdout.lines().next().unwrap_or("<missing>");
    let admission = tool.admit_version(observed);
    if output.succeeded {
        admission
    } else {
        admission?;
        Err(failed(tool, output.code, stdout, output.stderr)?)
    }
}

fn run_check(
    runner: &mut impl ToolRunner,
    tool: DocumentationTool,
    paths: &[String],
) -> Result<(), DocumentationError> {
    let mut arguments = text_arguments(tool.check_prefix());
    arguments.extend(paths.iter().cloned());
    let output = runner.capture(tool, &arguments)?;
    if output.succeeded {
        Ok(())
    } else {
        Err(failed(
            tool,
            output.code,
            String::from_utf8(output.stdout).map_err(|source| {
                DocumentationError::ToolOutputEncoding {
                    program: tool.program(),
                    stream: "stdout",
                    source,
                }
            })?,
            output.stderr,
        )?)
    }
}

fn failed(
    tool: DocumentationTool,
    code: Option<i32>,
    stdout: String,
    stderr: Vec<u8>,
) -> Result<DocumentationError, DocumentationError> {
    let stderr =
        String::from_utf8(stderr).map_err(|source| DocumentationError::ToolOutputEncoding {
            program: tool.program(),
            stream: "stderr",
            source,
        })?;
    Ok(DocumentationError::ToolFailed {
        program: tool.program(),
        code,
        stdout,
        stderr,
    })
}

fn text_arguments(arguments: &[&str]) -> Vec<String> {
    arguments.iter().map(ToString::to_string).collect()
}

impl ToolRunner for ExternalToolRunner<'_> {
    fn capture(
        &mut self,
        tool: DocumentationTool,
        arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError> {
        let mut command = Command::new(tool.program());
        command
            .args(arguments)
            .current_dir(self.repository_root)
            .stdin(Stdio::null());
        bounded_process::capture(tool.program(), &mut command, Some(TOOL_DEADLINE)).map_err(
            |source| {
                if source.is_not_found() {
                    DocumentationError::ToolUnavailable {
                        program: tool.program(),
                        install_version: tool.install_version(),
                        source,
                    }
                } else {
                    DocumentationError::Process(source)
                }
            },
        )
    }
}

#[cfg(test)]
#[path = "execution/external_tests.rs"]
mod external_tests;
#[cfg(test)]
#[path = "execution/tests.rs"]
mod tests;
