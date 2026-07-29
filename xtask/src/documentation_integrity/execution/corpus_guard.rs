//! This module owns corpus revalidation around external tool execution.

use crate::bounded_process::ProcessOutput;
use crate::documentation_integrity::corpus::SourceCorpus;
use crate::documentation_integrity::error::DocumentationError;
use crate::repository_file::RepositoryProcessDirectory;
use crate::repository_file::RepositoryRoot;

use super::snapshot::DocumentationSnapshot;
use super::{DirectoryToolRunner, DocumentationTool, ToolRunner};

/// Executes documentation tools against one admitted immutable source snapshot.
///
/// Construction copies bounded source bytes from retained descriptors and
/// materializes the reviewed repository namespace in a private temporary
/// directory. Every capture revalidates current source identities and exact Git
/// corpus membership before and after the inner runner executes from that
/// snapshot. Post-execution corpus drift takes precedence over a tool result.
///
/// Construction and capture perform bounded Git child-process and repository
/// I/O. The snapshot is verification evidence only and has no durability role;
/// callers must invoke [`Self::close`] to remove it explicitly.
pub(super) struct CorpusGuardedRunner<'a, Runner> {
    corpora: &'a [&'a SourceCorpus],
    inner: Runner,
    process_directory: &'a RepositoryProcessDirectory,
    repository_root: &'a RepositoryRoot,
    snapshot: DocumentationSnapshot,
}

impl<'a, Runner> CorpusGuardedRunner<'a, Runner> {
    /// Builds the source snapshot and binds `inner` to its execution authority.
    ///
    /// The call allocates bounded path and source inventories, copies admitted
    /// bytes, verifies fixed configuration and corpus identities, and returns
    /// the exact [`DocumentationError`] from any Git, filesystem, admission, or
    /// snapshot failure.
    pub(super) fn new(
        inner: Runner,
        process_directory: &'a RepositoryProcessDirectory,
        repository_root: &'a RepositoryRoot,
        corpora: &'a [&'a SourceCorpus],
    ) -> Result<Self, DocumentationError> {
        let snapshot = DocumentationSnapshot::create(repository_root, process_directory, corpora)?;
        Ok(Self {
            corpora,
            inner,
            process_directory,
            repository_root,
            snapshot,
        })
    }

    /// Releases snapshot descriptors and removes the exact temporary tree.
    ///
    /// Removal failure is returned as [`DocumentationError::Snapshot`]. This
    /// cleanup boundary does not mutate the source repository.
    pub(super) fn close(self) -> Result<(), DocumentationError> {
        self.snapshot.close()
    }

    fn verify(&self) -> Result<(), DocumentationError> {
        for corpus in self.corpora {
            corpus.verify_unchanged(self.repository_root, self.process_directory)?;
        }
        Ok(())
    }
}

impl<Runner> ToolRunner for CorpusGuardedRunner<'_, Runner>
where
    Runner: DirectoryToolRunner,
{
    fn capture(
        &mut self,
        tool: DocumentationTool,
        arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError> {
        self.verify()?;
        let result = self.inner.capture_in(
            self.snapshot.repository_root(),
            self.snapshot.process_directory(),
            tool,
            arguments,
        );
        self.verify()?;
        result
    }
}
