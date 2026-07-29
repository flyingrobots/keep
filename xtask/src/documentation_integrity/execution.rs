//! This module owns bounded execution of admitted documentation tools.

mod corpus_guard;
mod refusal_check;

use std::env;
use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::bounded_process::{self, ProcessOutput};
use crate::repository_file::{RepositoryProcessDirectory, RepositoryRoot};

use super::corpus::SourceCorpus;
use super::error::DocumentationError;
use super::tool::DocumentationTool;
use corpus_guard::CorpusGuardedRunner;

const TOOL_DEADLINE: Duration = Duration::from_mins(2);

trait ToolRunner {
    fn capture(
        &mut self,
        tool: DocumentationTool,
        arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError>;
}

struct ExternalToolRunner<'a> {
    process_directory: &'a RepositoryProcessDirectory,
}

pub(super) fn run(
    process_directory: &RepositoryProcessDirectory,
    repository_root: &RepositoryRoot,
    markdown: &SourceCorpus,
    workflows: &SourceCorpus,
) -> Result<(), DocumentationError> {
    let corpora = [markdown, workflows];
    let external = ExternalToolRunner { process_directory };
    let mut runner = CorpusGuardedRunner::new(external, repository_root, &corpora);
    run_with(&mut runner, markdown.paths(), workflows.paths())
}

/// Executes both named malformed-input scenarios through the production runner.
pub(super) fn check_refusals() -> Result<(), DocumentationError> {
    refusal_check::check()
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
    combine_checks(lint, links)?;
    admit_version(runner, DocumentationTool::Actionlint)?;
    run_check(runner, DocumentationTool::Actionlint, workflows)
}

fn combine_checks(
    first: Result<(), DocumentationError>,
    second: Result<(), DocumentationError>,
) -> Result<(), DocumentationError> {
    match (first, second) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(first), Err(second)) => Err(DocumentationError::CheckFailures {
            first: Box::new(first),
            second: Box::new(second),
        }),
    }
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
        let path = env::var_os("PATH").ok_or(DocumentationError::EnvironmentUnavailable("PATH"))?;
        let mut command = documentation_command(tool, arguments, &path);
        bounded_process::capture_with(
            tool.program(),
            &mut command,
            Some(TOOL_DEADLINE),
            |command| self.process_directory.spawn(command),
        )
        .map_err(|source| {
            if source.is_not_found() {
                DocumentationError::ToolUnavailable {
                    program: tool.program(),
                    install_version: tool.install_version(),
                    source,
                }
            } else {
                DocumentationError::Process(source)
            }
        })
    }
}

fn documentation_command(tool: DocumentationTool, arguments: &[String], path: &OsStr) -> Command {
    let mut command = Command::new(tool.program());
    command
        .args(arguments)
        .stdin(Stdio::null())
        .env_clear()
        .env("PATH", path)
        .env("LC_ALL", "C");
    command
}

#[cfg(test)]
#[path = "execution/tests.rs"]
mod tests;
