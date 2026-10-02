//! This boundary module owns the durable store handle: a version-two root
//! that pins snapshots on demand.

use std::io::Write;
use std::path::{Path, PathBuf};

use super::{
    DurableRangeReadReceipt, DurableReadError, DurableReconstructionReceipt, DurableSnapshot,
    DurableStoreError,
};
use crate::adapters::{CatalogRestartPolicy, ReaderAttemptLimit};
use crate::{BlobId, ByteRange, LayoutId};

/// One migrated version-two store to read from.
///
/// The handle holds no fence and no view; every read pins a fresh
/// [`DurableSnapshot`] unless the caller pins one with [`Self::snapshot`]
/// and reads through it. `open` takes no authority and touches nothing;
/// admission happens when a snapshot is pinned.
/// Every convenience read pays the complete snapshot admission and allocation
/// cost described by [`DurableSnapshot`]; callers doing repeated reads should
/// retain an explicit snapshot. No method here publishes or synchronizes data.
#[must_use]
#[derive(Debug)]
pub struct DurableStore {
    root: PathBuf,
    policy: CatalogRestartPolicy,
    limit: ReaderAttemptLimit,
}

impl DurableStore {
    /// Names the store at `root`, reading under `policy` and collecting a
    /// consistent view within `limit` attempts.
    ///
    /// Allocates a copy of the locator, performs no I/O, and does not prove
    /// that the named store exists. This path is not a content identity.
    pub fn open(root: &Path, policy: CatalogRestartPolicy, limit: ReaderAttemptLimit) -> Self {
        Self {
            root: root.to_path_buf(),
            policy,
            limit,
        }
    }

    /// The store root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Pins one consistent, fenced view.
    ///
    /// # Errors
    ///
    /// Returns [`DurableStoreError`] at the exact admission, fence,
    /// collection, or retained-root refusal.
    pub fn snapshot(&self) -> Result<DurableSnapshot, DurableStoreError> {
        DurableSnapshot::open(&self.root, self.policy, self.limit)
    }

    /// Whether some retained root anchors `target` in a fresh view.
    ///
    /// # Errors
    ///
    /// As [`Self::snapshot`].
    pub fn contains_blob(&self, target: BlobId) -> Result<bool, DurableStoreError> {
        self.snapshot()?.contains_blob(target)
    }

    /// Reconstructs `target` against a fresh view.
    ///
    /// # Errors
    ///
    /// Returns snapshot admission through [`DurableOutcome::Store`] or the
    /// exact read failure through [`DurableOutcome::Read`], preserving sources.
    pub fn reconstruct<W>(
        &self,
        target: BlobId,
        output: &mut W,
    ) -> Result<DurableReconstructionReceipt, DurableOutcome>
    where
        W: Write + ?Sized,
    {
        let snapshot = self.snapshot().map_err(DurableOutcome::Store)?;
        snapshot
            .reconstruct(target, output)
            .map_err(DurableOutcome::Read)
    }

    /// Reconstructs the exact committed layout against a fresh view.
    ///
    /// # Errors
    ///
    /// As [`Self::reconstruct`].
    pub fn reconstruct_layout<W>(
        &self,
        layout_id: LayoutId,
        output: &mut W,
    ) -> Result<DurableReconstructionReceipt, DurableOutcome>
    where
        W: Write + ?Sized,
    {
        let snapshot = self.snapshot().map_err(DurableOutcome::Store)?;
        snapshot
            .reconstruct_layout(layout_id, output)
            .map_err(DurableOutcome::Read)
    }

    /// Reads exactly `requested` of `target` against a fresh view.
    ///
    /// # Errors
    ///
    /// As [`Self::reconstruct`].
    pub fn read_range<W>(
        &self,
        target: BlobId,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<DurableRangeReadReceipt, DurableOutcome>
    where
        W: Write + ?Sized,
    {
        let snapshot = self.snapshot().map_err(DurableOutcome::Store)?;
        snapshot
            .read_range(target, requested, output)
            .map_err(DurableOutcome::Read)
    }
}

/// Why a store-level read returned no receipt: the view could not be
/// pinned, or the pinned view refused.
#[derive(Debug)]
pub enum DurableOutcome {
    /// The snapshot could not be pinned.
    Store(DurableStoreError),
    /// The pinned view refused the read.
    Read(DurableReadError),
}

impl std::fmt::Display for DurableOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(source) => write!(formatter, "{source}"),
            Self::Read(source) => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for DurableOutcome {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(source) => Some(source),
            Self::Read(source) => Some(source),
        }
    }
}
