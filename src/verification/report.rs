//! This module owns the statement one completed verification makes.

use super::{VerificationDepth, VerificationSubject};
use crate::{BlobId, LayoutId};

/// Exactly what one verification established.
///
/// The report names the subject, the one depth established, and the exact
/// layout and target coordinates that depth was established against. It has
/// no constructor outside Keep and no method that raises its depth, so a
/// report at one depth cannot be presented as a report at a deeper one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the verification report records the depth that was established"]
pub struct VerificationReport {
    subject: VerificationSubject,
    depth: VerificationDepth,
    layout: LayoutId,
    target: BlobId,
    chunks_verified: u64,
}

impl VerificationReport {
    pub(crate) const fn established(
        subject: VerificationSubject,
        depth: VerificationDepth,
        layout: LayoutId,
        target: BlobId,
        chunks_verified: u64,
    ) -> Self {
        Self {
            subject,
            depth,
            layout,
            target,
            chunks_verified,
        }
    }

    /// Returns the subject that was verified.
    pub const fn subject(self) -> VerificationSubject {
        self.subject
    }

    /// Returns the one depth this report establishes; never deeper.
    pub const fn depth(self) -> VerificationDepth {
        self.depth
    }

    /// Returns the exact layout the depth was established through.
    #[must_use]
    pub const fn layout(self) -> LayoutId {
        self.layout
    }

    /// Returns the target blob that layout binds.
    #[must_use]
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// Returns how many chunks were authenticated for this report.
    #[must_use]
    pub const fn chunks_verified(self) -> u64 {
        self.chunks_verified
    }
}
