//! This module owns the CI documentation-job execution contract.

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;

const CI_PATH: &str = ".github/workflows/ci.yml";
const DOCUMENTATION_JOB: &str = "  documentation:";
const XTASK_COMMAND: &str = "run: cargo xtask documentation-integrity-check";

pub(super) fn check(repository_root: &RepositoryRoot) -> Result<(), DocumentationError> {
    let workflow = repository_text::read(repository_root, CI_PATH)?;
    admit(&workflow)
}

fn admit(workflow: &str) -> Result<(), DocumentationError> {
    let job = documentation_job(workflow)?;
    if !job.contains("run: rustup show") {
        return Err(contract(
            "documentation job installs the pinned Rust toolchain",
        ));
    }
    if job.matches(XTASK_COMMAND).count() != 1 {
        return Err(contract(
            "documentation job runs the Rust integrity command exactly once",
        ));
    }
    if job.contains("python3") {
        return Err(contract("documentation job contains no Python execution"));
    }
    Ok(())
}

fn documentation_job(workflow: &str) -> Result<String, DocumentationError> {
    let mut lines = workflow
        .lines()
        .skip_while(|line| *line != DOCUMENTATION_JOB);
    if lines.next().is_none() {
        return Err(contract("workflow defines the documentation job"));
    }
    let job: Vec<_> = lines
        .take_while(|line| line.starts_with("    ") || line.is_empty() || !line.starts_with("  "))
        .collect();
    if job.is_empty() {
        Err(contract("documentation job is not empty"))
    } else {
        Ok(job.join("\n"))
    }
}

const fn contract(requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract {
        path: CI_PATH,
        requirement,
    }
}

#[cfg(test)]
#[path = "workflow_contract/tests.rs"]
mod tests;
