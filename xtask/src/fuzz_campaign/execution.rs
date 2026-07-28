//! This module owns all-target cargo-fuzz execution and failure aggregation.

mod error;
#[cfg(test)]
mod tests;

use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub(crate) use error::ExecutionError;
use error::{TargetFailure, TargetFailureReason};

use super::command::{CommandPlan, OutputMode};
use crate::bounded_process::{self, ProcessError, ProcessOutput};

const CARGO_FUZZ_PROCESS: &str = "cargo-fuzz";

trait CommandRunner {
    fn execute(
        &mut self,
        repository_root: &Path,
        plan: &CommandPlan,
    ) -> Result<ProcessOutput, ProcessError>;
}

struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn execute(
        &mut self,
        repository_root: &Path,
        plan: &CommandPlan,
    ) -> Result<ProcessOutput, ProcessError> {
        let mut command = Command::new("cargo");
        command.args(plan.arguments()).current_dir(repository_root);
        match plan.output_mode() {
            OutputMode::Capture => {
                let output =
                    bounded_process::capture(CARGO_FUZZ_PROCESS, &mut command, plan.deadline())?;
                replay(&output)?;
                Ok(output)
            }
            OutputMode::Inherit => {
                bounded_process::status(CARGO_FUZZ_PROCESS, &mut command, plan.deadline())
            }
        }
    }
}

pub(super) fn run(
    repository_root: &Path,
    operation: &'static str,
    plans: &[CommandPlan],
) -> Result<(), ExecutionError> {
    execute_all(repository_root, operation, plans, &mut SystemRunner)
}

fn replay(output: &ProcessOutput) -> Result<(), ProcessError> {
    write_output(
        &mut io::stdout().lock(),
        "write captured child stdout",
        &output.stdout,
    )?;
    write_output(
        &mut io::stderr().lock(),
        "write captured child stderr",
        &output.stderr,
    )
}

fn write_output(
    writer: &mut impl Write,
    action: &'static str,
    bytes: &[u8],
) -> Result<(), ProcessError> {
    writer
        .write_all(bytes)
        .and_then(|()| writer.flush())
        .map_err(|source| ProcessError::Io {
            program: CARGO_FUZZ_PROCESS,
            action,
            source,
        })
}

fn execute_all(
    repository_root: &Path,
    operation: &'static str,
    plans: &[CommandPlan],
    runner: &mut impl CommandRunner,
) -> Result<(), ExecutionError> {
    let mut failures = Vec::new();
    for plan in plans {
        let failure = match runner.execute(repository_root, plan) {
            Ok(output) => classify_output(plan, &output),
            Err(error) => Some(TargetFailureReason::Process(error)),
        };
        if let Some(reason) = failure {
            failures.push(TargetFailure {
                target: plan.target().as_str().to_owned(),
                reason,
            });
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(ExecutionError {
            operation,
            failures,
        })
    }
}

fn classify_output(plan: &CommandPlan, output: &ProcessOutput) -> Option<TargetFailureReason> {
    if !output.succeeded {
        return Some(TargetFailureReason::Exit);
    }
    let marker = plan.refused_output_marker()?.as_bytes();
    if contains(&output.stdout, marker) || contains(&output.stderr, marker) {
        Some(TargetFailureReason::RefusedOutput)
    } else {
        None
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}
