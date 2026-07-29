//! This module owns the retained fixed-policy identity corpus.

use crate::repository_file::{RepositoryProcessDirectory, RepositoryRoot};

use super::contributor_contract;
use super::dependabot;
use super::error::DocumentationError;
use super::node_toolchain;
use super::repository_text::RepositoryText;
use super::workflow_contract;

const POLICY_SOURCE_COUNT: usize = 7;

/// The fixed policy files whose admitted identities govern one complete check.
pub(super) struct FixedPolicyCorpus {
    sources: [RepositoryText; POLICY_SOURCE_COUNT],
}

impl FixedPolicyCorpus {
    /// Admits every fixed policy file and retains its open identity witness.
    pub(super) fn admit(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
    ) -> Result<Self, DocumentationError> {
        let [contributing, standards] = contributor_contract::check(repository_root)?;
        let [manifest, lock, installer] = node_toolchain::check(repository_root)?;
        let dependabot = dependabot::check(repository_root, process_directory)?;
        let workflow = workflow_contract::check(repository_root)?;
        Ok(Self {
            sources: [
                contributing,
                standards,
                manifest,
                lock,
                installer,
                dependabot,
                workflow,
            ],
        })
    }

    /// Revalidates every retained policy identity at the final success boundary.
    pub(super) fn verify(
        &self,
        repository_root: &RepositoryRoot,
    ) -> Result<(), DocumentationError> {
        for source in &self.sources {
            source.verify(repository_root)?;
        }
        Ok(())
    }
}
