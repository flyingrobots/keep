//! This module owns the coordinate a verification is about.

use crate::{BlobId, LayoutId};

/// The exact content coordinate a verification establishes or refuses.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VerificationSubject {
    /// One logical blob through the view's deterministic layout choice.
    Blob(BlobId),
    /// One exact committed layout.
    Layout(LayoutId),
}
