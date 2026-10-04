//! Whole-blob authentication followed by exact synchronous emission.

use std::io::Write;

use crate::{AdmittedLayout, BlobId, LayoutDecodePolicy, LayoutId, ReferenceStore};

use super::{ReconstructionError, ReconstructionReceipt};
use crate::authenticated_read::reconstruct_admitted;

impl ReferenceStore {
    /// Reconstructs the exact bytes named by `target`.
    ///
    /// The lowest canonical committed [`LayoutId`] is chosen deterministically
    /// when more than one layout names the blob. Reconstruction first verifies
    /// every chunk, the registered storage-profile boundaries, and the complete
    /// logical [`BlobId`] without writing, hashing each chunk exactly once. It
    /// then emits each verified immutable chunk by identity without hashing it
    /// again: the in-memory view cannot change under `&self`, so no
    /// unauthenticated byte reaches `output` and no chunk pays for two hashes.
    ///
    /// Short writes are completed and interrupted writes are retried. This
    /// synchronous blocking operation allocates no adapter-owned heap memory,
    /// does not flush `output`, and makes no durability claim. Any allocation
    /// performed by `output` belongs to the caller-provided writer.
    ///
    /// # Errors
    ///
    /// Returns [`ReconstructionError`] for absent state, chunk or blob
    /// mismatch, broken writer behavior, checked accounting failure, or output
    /// I/O failure.
    pub fn reconstruct<W>(
        &self,
        target: BlobId,
        output: &mut W,
    ) -> Result<ReconstructionReceipt, ReconstructionError>
    where
        W: Write + ?Sized,
    {
        let layout_id = self
            .first_layout_id(target)
            .ok_or(ReconstructionError::BlobMissing { requested: target })?;
        self.reconstruct_layout(layout_id, output)
    }

    /// Reconstructs through one exact committed canonical layout.
    ///
    /// Authentication and output behavior are identical to
    /// [`ReferenceStore::reconstruct`].
    ///
    /// # Errors
    ///
    /// Returns [`ReconstructionError`] for an absent layout, missing or
    /// mismatched content, broken writer behavior, accounting failure, or
    /// output I/O failure.
    pub fn reconstruct_layout<W>(
        &self,
        layout_id: LayoutId,
        output: &mut W,
    ) -> Result<ReconstructionReceipt, ReconstructionError>
    where
        W: Write + ?Sized,
    {
        let layout = self
            .layout(layout_id)
            .ok_or(ReconstructionError::LayoutMissing {
                requested: layout_id,
            })?;
        reconstruct_admitted(self, layout_id, layout, output).map_err(ReconstructionError::from)
    }

    /// Reconstructs through a caller-supplied admitted semantic layout.
    ///
    /// The layout need not be published in this store, but every chunk it
    /// names must be present and exact. The canonical layout identity is
    /// calculated before content verification by materializing one canonical
    /// record bounded by the admitted layout's protocol entry limit.
    ///
    /// # Errors
    ///
    /// Returns [`ReconstructionError`] for canonical encoding failure, absent
    /// or mismatched content, broken writer behavior, accounting failure, or
    /// output I/O failure.
    pub fn reconstruct_admitted_layout<W>(
        &self,
        layout: &AdmittedLayout,
        output: &mut W,
    ) -> Result<ReconstructionReceipt, ReconstructionError>
    where
        W: Write + ?Sized,
    {
        let layout_id = layout
            .encode_record()
            .map_err(ReconstructionError::LayoutEncoding)?
            .id();
        reconstruct_admitted(self, layout_id, layout, output).map_err(ReconstructionError::from)
    }

    /// Decodes and reconstructs one exact canonical layout record.
    ///
    /// Bounded decoding and semantic admission allocate entry metadata within
    /// `policy` before chunk lookup or output. Canonical identity calculation
    /// transiently materializes one bounded record. Authentication and emission
    /// then follow
    /// [`ReferenceStore::reconstruct_admitted_layout`].
    ///
    /// # Errors
    ///
    /// Returns [`ReconstructionError`] for malformed, noncanonical, or
    /// policy-exceeding layout bytes; absent or mismatched content; broken
    /// writer behavior; accounting failure; or output I/O failure.
    pub fn reconstruct_record<W>(
        &self,
        encoded: &[u8],
        policy: LayoutDecodePolicy,
        output: &mut W,
    ) -> Result<ReconstructionReceipt, ReconstructionError>
    where
        W: Write + ?Sized,
    {
        let layout = AdmittedLayout::decode_record(encoded, policy)
            .map_err(ReconstructionError::LayoutDecode)?;
        self.reconstruct_admitted_layout(&layout, output)
    }
}

#[cfg(test)]
#[path = "reconstruction_tests.rs"]
mod tests;
