//! This module owns extraction of typed namespace refusals from store admission.

use keep::{FilesystemNamespaceRefusal, FilesystemStoreAdmissionError};

/// The namespace refusal one store admission failure carries, if any.
///
/// Admission preserves the refusal as the source of its namespace boundary,
/// so laws assert the exact contradiction instead of an error string.
#[must_use]
pub(crate) fn namespace_refusal(
    error: &FilesystemStoreAdmissionError,
) -> Option<FilesystemNamespaceRefusal> {
    let FilesystemStoreAdmissionError::Namespace { source } = error else {
        return None;
    };
    source
        .get_ref()?
        .downcast_ref::<FilesystemNamespaceRefusal>()
        .copied()
}
