//! This module owns documentation corpus byte-budget admission.

use super::{AdmittedSource, CorpusKind};
use crate::documentation_integrity::error::DocumentationError;

/// Maximum admitted bytes for one documentation source.
pub(super) const CORPUS_FILE_MAX_BYTES: u64 = 4_194_304;
/// Maximum admitted bytes for one complete documentation corpus.
pub(super) const CORPUS_MAX_BYTES: u64 = 67_108_864;

/// Checked aggregate byte accounting for one corpus.
#[derive(Default)]
pub(super) struct CorpusByteBudget {
    observed: u64,
}

impl CorpusByteBudget {
    /// Admits one bounded source and advances the aggregate count.
    pub(super) fn admit(
        &mut self,
        kind: CorpusKind,
        source: &AdmittedSource,
    ) -> Result<(), DocumentationError> {
        if source.bytes > CORPUS_FILE_MAX_BYTES {
            return Err(DocumentationError::CorpusFileTooLarge {
                corpus: kind.label(),
                path: source.path.clone(),
                maximum: CORPUS_FILE_MAX_BYTES,
                observed: source.bytes,
            });
        }
        let observed = self
            .observed
            .checked_add(source.bytes)
            .ok_or_else(|| DocumentationError::CorpusSizeOverflow(kind.label()))?;
        if observed > CORPUS_MAX_BYTES {
            return Err(DocumentationError::CorpusTooLarge {
                corpus: kind.label(),
                maximum: CORPUS_MAX_BYTES,
                observed,
            });
        }
        self.observed = observed;
        Ok(())
    }
}
