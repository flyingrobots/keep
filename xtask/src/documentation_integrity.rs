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

pub(super) fn check(repository_path: &Path) -> Result<(), DocumentationError> {
    let repository_root = RepositoryRoot::open(repository_path).map_err(|source| {
        DocumentationError::RepositoryRootInspect {
            path: repository_path.to_owned(),
            source,
        }
    })?;
    verify_root(&repository_root, repository_path)?;
    contributor_contract::check(&repository_root)?;
    node_toolchain::check(&repository_root)?;
    dependabot::check(repository_path, &repository_root)?;
    workflow_contract::check(&repository_root)?;
    let markdown = corpus::SourceCorpus::markdown(repository_path)?;
    let workflows = corpus::SourceCorpus::workflow(repository_path)?;
    execution::run(repository_path, markdown.paths(), workflows.paths())?;
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
