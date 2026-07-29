//! This module owns documentation and workflow integrity orchestration.

mod contributor_contract;
mod corpus;
mod dependabot;
mod error;
mod execution;
mod node_toolchain;
mod policy_corpus;
mod repository_text;
mod tool;
mod workflow_contract;

use std::path::{Path, PathBuf};

use crate::repository_file::{RepositoryProcessDirectory, RepositoryRoot};

pub(super) use error::DocumentationError;

/// Runs the pinned-tool malformed-input refusal evidence.
pub(super) fn check_refusals() -> Result<(), DocumentationError> {
    execution::check_refusals()
}

pub(super) fn check(repository_path: &Path) -> Result<(), DocumentationError> {
    check_with(repository_path, execution::run)
}

fn check_with(
    repository_path: &Path,
    run_tools: impl FnOnce(
        &RepositoryProcessDirectory,
        &RepositoryRoot,
        &corpus::SourceCorpus,
        &corpus::SourceCorpus,
    ) -> Result<(), DocumentationError>,
) -> Result<(), DocumentationError> {
    let repository_root = RepositoryRoot::open(repository_path).map_err(|source| {
        DocumentationError::RepositoryRootInspect {
            path: repository_path.to_owned(),
            source,
        }
    })?;
    let process_directory = repository_root.process_directory().map_err(|source| {
        DocumentationError::RepositoryRootInspect {
            path: repository_path.to_owned(),
            source,
        }
    })?;
    verify_root(&repository_root, repository_path)?;
    let policies = policy_corpus::FixedPolicyCorpus::admit(&repository_root, &process_directory)?;
    let markdown = corpus::SourceCorpus::markdown(&repository_root, &process_directory)?;
    let workflows = corpus::SourceCorpus::workflow(&repository_root, &process_directory)?;
    run_tools(&process_directory, &repository_root, &markdown, &workflows)?;
    policies.verify(&repository_root)?;
    verify_root(&repository_root, repository_path)
}

fn verify_root(
    repository_root: &RepositoryRoot,
    repository_path: &Path,
) -> Result<(), DocumentationError> {
    match repository_root.is_current_path() {
        Ok(true) => Ok(()),
        Ok(false) => Err(DocumentationError::RepositoryRootChanged(
            repository_path.to_owned(),
        )),
        Err(source) => Err(DocumentationError::RepositoryRootInspect {
            path: PathBuf::from(repository_path),
            source,
        }),
    }
}

#[cfg(test)]
#[path = "documentation_integrity/tests.rs"]
mod tests;
