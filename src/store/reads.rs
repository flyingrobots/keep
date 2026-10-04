//! This boundary module owns the read half of the content-store port.

use std::error::Error;
use std::fmt::Debug;
use std::io::Write;

use crate::{BlobId, ByteRange, LayoutId};

/// Authenticated reads over one admitted view.
///
/// The laws are the reference store's: every emitted byte is
/// authenticated first, an exact layout never substitutes another, a range
/// proves only the requested bytes, and absence is evidence only against a
/// complete view.
pub trait ContentReads {
    /// Why checking visibility could not establish presence or absence.
    type ContainsError: Error + 'static;
    /// The receipt one complete reconstruction returns.
    type ReconstructionReceipt: Copy + Debug + Eq;
    /// The receipt one exact range read returns.
    type RangeReceipt: Copy + Debug + Eq;
    /// Why a reconstruction returned no receipt.
    type ReconstructionError: Error + 'static;
    /// Why a range read returned no receipt.
    type RangeError: Error + 'static;

    /// Whether the view can serve `target` by identity.
    ///
    /// # Errors
    ///
    /// Returns the original admission or operational failure when the view
    /// cannot establish visibility. Failure is never reported as absence.
    fn contains_blob(&self, target: BlobId) -> Result<bool, Self::ContainsError>;

    /// Reconstructs `target` through the view's deterministic layout choice.
    ///
    /// # Errors
    ///
    /// Returns the backend's evidenced refusal or operational failure.
    fn reconstruct(
        &self,
        target: BlobId,
        output: &mut dyn Write,
    ) -> Result<Self::ReconstructionReceipt, Self::ReconstructionError>;

    /// Reconstructs the exact committed layout `layout_id`, never another.
    ///
    /// # Errors
    ///
    /// As [`Self::reconstruct`].
    fn reconstruct_layout(
        &self,
        layout_id: LayoutId,
        output: &mut dyn Write,
    ) -> Result<Self::ReconstructionReceipt, Self::ReconstructionError>;

    /// Reads exactly `requested` of `target`, authenticating only the
    /// overlapping chunks.
    ///
    /// # Errors
    ///
    /// Returns the backend's evidenced refusal or operational failure.
    fn read_range(
        &self,
        target: BlobId,
        requested: ByteRange,
        output: &mut dyn Write,
    ) -> Result<Self::RangeReceipt, Self::RangeError>;

    /// Reads exactly `requested` through the exact committed layout.
    ///
    /// # Errors
    ///
    /// As [`Self::read_range`].
    fn read_layout_range(
        &self,
        layout_id: LayoutId,
        requested: ByteRange,
        output: &mut dyn Write,
    ) -> Result<Self::RangeReceipt, Self::RangeError>;
}
