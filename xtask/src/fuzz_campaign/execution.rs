//! This module owns all-target cargo-fuzz execution and failure aggregation.

mod error;
#[cfg(test)]
mod tests;

use std::path::Path;
use std::process::Command;

pub(crate) use error::ExecutionError;
use error::{TargetFailure, TargetFailureReason};

use super::command::CommandPlan;
use super::process::{self, ProcessError, ProcessOutput};

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
        process::capture(&mut command, plan.deadline())
    }
}

pub(super) fn run(
    repository_root: &Path,
    operation: &'static str,
    plans: &[CommandPlan],
) -> Result<(), ExecutionError> {
    execute_all(repository_root, operation, plans, &mut SystemRunner)
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
