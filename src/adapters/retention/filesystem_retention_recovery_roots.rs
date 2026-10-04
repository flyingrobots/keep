//! This module owns selected root reopening before filesystem recovery effects.

use std::io;

use cap_std::fs::Dir;

use super::filesystem_retention_current::{verify_committed, verify_predecessor};
use super::{RetentionRecoveryEvidence, RetentionStageAssessment};

/// Reopens the published root selected for the complete staged root's namespace.
///
/// A successor must authenticate the predecessor selected by the published
/// manifest, just as forward publication does. If that selection is already
/// the staged root, recovery is cleaning up a committed transition and instead
/// requires the selected bytes to equal that root. Initial namespaces have no
/// published predecessor. The pure plan has already refused contradictory
/// stage coordinates; this admission performs bounded reads and no mutation.
pub(super) fn admit(roots: &Dir, evidence: &RetentionRecoveryEvidence<'_, '_>) -> io::Result<()> {
    let Some(current) = evidence.current() else {
        return Ok(());
    };
    let RetentionStageAssessment::Complete(candidate) = &evidence.stages().root else {
        return Ok(());
    };
    let namespace = candidate.root().namespace().digest();
    let entries = current.manifest().entries();
    let Some(entry) = entries
        .binary_search_by_key(&namespace, |entry| entry.namespace())
        .ok()
        .and_then(|index| entries.get(index))
    else {
        return Ok(());
    };
    if entry.root_generation() == candidate.root().generation()
        && entry.root_digest() == candidate.digest()
    {
        verify_committed(roots, current, candidate)
    } else {
        verify_predecessor(roots, current, candidate)
    }
}
