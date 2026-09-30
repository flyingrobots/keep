//! This boundary module owns the write half of the content-store port:
//! stage under limits, then commit for a receipt.

use std::error::Error;
use std::fmt::Debug;
use std::io::Read;

use super::StagingLimits;
use crate::{BlobId, LayoutId};

/// What one committed staging proves: the target and the exact layout it
/// became visible under. Each backend's receipt is its own type.
pub trait CommitReceipt: Copy + Debug + Eq {
    /// The blob made visible.
    fn target(&self) -> BlobId;
    /// The exact committed layout.
    fn layout_id(&self) -> LayoutId;
}

/// Content staged into one store but not yet visible.
///
/// The staging borrows its store for its lifetime, so the store cannot
/// change underneath it, and commit needs no second reference.
pub trait StagedContent: Sized {
    /// The commit receipt.
    type Receipt: CommitReceipt;
    /// Why the commit returned no receipt.
    type Error: Error + 'static;

    /// The blob identity the staging established.
    fn target(&self) -> BlobId;

    /// The exact layout the staging will commit under.
    fn layout_id(&self) -> LayoutId;

    /// Makes the staged content visible, or leaves it invisible.
    ///
    /// # Errors
    ///
    /// Returns the backend's refusal; nothing becomes visible on failure.
    fn commit(self) -> Result<Self::Receipt, Self::Error>;
}

/// Staging an unknown-length source under count-and-byte limits.
pub trait ContentStaging {
    /// The staged, not-yet-visible content, borrowing this store.
    type Staged<'store>: StagedContent
    where
        Self: 'store;
    /// Why staging returned nothing.
    type Error: Error + 'static;

    /// Chunks, hashes, and holds `source` without making it visible,
    /// refusing before any excess over `limits` is materialized.
    ///
    /// # Errors
    ///
    /// Returns the backend's refusal or the source's failure.
    fn stage<'store>(
        &'store mut self,
        source: &mut dyn Read,
        limits: StagingLimits,
    ) -> Result<Self::Staged<'store>, Self::Error>;

    /// As [`Self::stage`], refusing when the source does not hash to
    /// `expected`.
    ///
    /// # Errors
    ///
    /// As [`Self::stage`], plus the identity mismatch.
    fn stage_expected<'store>(
        &'store mut self,
        source: &mut dyn Read,
        expected: BlobId,
        limits: StagingLimits,
    ) -> Result<Self::Staged<'store>, Self::Error>;
}
