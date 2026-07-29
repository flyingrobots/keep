//! This module owns documentation and workflow integrity orchestration.

mod contributor_contract;
mod corpus;
mod dependabot;
mod error;
mod execution;
mod node_toolchain;
mod repository_text;
mod tool;
mod workflow_contract;

use std::path::{Path, PathBuf};

use crate::repository_file::RepositoryRoot;

pub(super) use error::DocumentationError;

/// Runs the pinned-tool malformed-input refusal evidence.
pub(super) fn check_refusals() -> Result<(), DocumentationError> {
    execution::check_refusals()
}

pub(super) fn check(repository_path: &Path) -> Result<(), DocumentationError> {
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
    contributor_contract::check(&repository_root)?;
    node_toolchain::check(&repository_root)?;
    dependabot::check(&repository_root, &process_directory)?;
    workflow_contract::check(&repository_root)?;
    let markdown = corpus::SourceCorpus::markdown(&repository_root, &process_directory)?;
    let workflows = corpus::SourceCorpus::workflow(&repository_root, &process_directory)?;
    execution::run(&process_directory, &repository_root, &markdown, &workflows)?;
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
