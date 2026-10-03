//! This module owns preservation of unrelated namespaces in recovery successors.

use super::{AdmittedRetentionManifest, AdmittedRetentionRoot, ObservedRetentionState};

/// Requires every entry outside the staged root namespace to remain exact.
///
/// Both admitted slices are canonically ordered and duplicate-free, so stream
/// equality rejects omissions, additions, and coordinate changes without
/// allocation. An absent current state permits only the staged namespace.
pub(super) fn preserves_unrelated(
    current: Option<&ObservedRetentionState>,
    manifest: &AdmittedRetentionManifest<'_>,
    root: &AdmittedRetentionRoot<'_>,
) -> bool {
    let namespace = root.root().namespace().digest();
    let before = current.map_or(&[][..], |state| state.manifest().entries());
    before
        .iter()
        .filter(|entry| entry.namespace() != namespace)
        .eq(manifest
            .manifest()
            .entries()
            .iter()
            .filter(|entry| entry.namespace() != namespace))
}
