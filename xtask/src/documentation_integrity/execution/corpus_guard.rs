//! This module owns corpus revalidation around external tool execution.

use crate::bounded_process::ProcessOutput;
use crate::documentation_integrity::corpus::SourceCorpus;
use crate::documentation_integrity::error::DocumentationError;
use crate::repository_file::RepositoryRoot;

use super::{DocumentationTool, ToolRunner};

pub(super) struct CorpusGuardedRunner<'a, Runner> {
    corpora: &'a [&'a SourceCorpus],
    inner: Runner,
    repository_root: &'a RepositoryRoot,
}

impl<'a, Runner> CorpusGuardedRunner<'a, Runner> {
    pub(super) const fn new(
        inner: Runner,
        repository_root: &'a RepositoryRoot,
        corpora: &'a [&'a SourceCorpus],
    ) -> Self {
        Self {
            corpora,
            inner,
            repository_root,
        }
    }

    fn verify(&self) -> Result<(), DocumentationError> {
        for corpus in self.corpora {
            corpus.verify_unchanged(self.repository_root)?;
        }
        Ok(())
    }
}

impl<Runner> ToolRunner for CorpusGuardedRunner<'_, Runner>
where
    Runner: ToolRunner,
{
    fn capture(
        &mut self,
        tool: DocumentationTool,
        arguments: &[String],
    ) -> Result<ProcessOutput, DocumentationError> {
        self.verify()?;
        let result = self.inner.capture(tool, arguments);
        self.verify()?;
        result
    }
}
