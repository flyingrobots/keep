//! Whole-blob authentication followed by exact synchronous emission.

use std::io::Write;

use crate::{
    AdmittedLayout, BlobHasher, BlobId, BlobLength, LayoutDecodePolicy, LayoutId, ReferenceStore,
};

use super::chunk_verification::{ChunkVerificationError, verified_chunk};
use super::output_write::{OutputWriteError, write_all};
use super::profile_verification::ProfileVerifier;
use super::{ReconstructionError, ReconstructionReceipt};

impl ReferenceStore {
    /// Reconstructs the exact bytes named by `target`.
    ///
    /// The lowest canonical committed [`LayoutId`] is chosen deterministically
    /// when more than one layout names the blob. Reconstruction first verifies
    /// every chunk, the registered storage-profile boundaries, and the complete
    /// logical [`BlobId`] without writing. It then reverifies each immutable
    /// reference-store chunk immediately before emitting it, so no
    /// unauthenticated byte reaches `output`.
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
        reconstruct_admitted(self, layout_id, layout, output)
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
        reconstruct_admitted(self, layout_id, layout, output)
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

fn reconstruct_admitted<W>(
    store: &ReferenceStore,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    output: &mut W,
) -> Result<ReconstructionReceipt, ReconstructionError>
where
    W: Write + ?Sized,
{
    verify_complete_blob(store, layout_id, layout)?;
    let written = emit_authenticated(store, layout_id, layout, output)?;
    let expected = layout.target().logical_length();
    if written != expected {
        return Err(ReconstructionError::WrittenLengthMismatch {
            layout: layout_id,
            expected,
            observed: written,
        });
    }
    Ok(ReconstructionReceipt::new(
        layout.target(),
        layout_id,
        written,
    ))
}

fn verify_complete_blob(
    store: &ReferenceStore,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
) -> Result<(), ReconstructionError> {
    let mut hasher = BlobHasher::new();
    let mut profile = ProfileVerifier::new(layout_id, layout)?;
    for (index, entry) in layout.entries().iter().copied().enumerate() {
        let bytes =
            verified_chunk(store, layout_id, index, entry).map_err(reconstruction_chunk_error)?;
        profile.feed(bytes)?;
        hasher
            .update(bytes)
            .map_err(ReconstructionError::BlobHash)?;
    }
    profile.finish()?;
    let observed = hasher.finish();
    let expected = layout.target();
    if observed != expected {
        return Err(ReconstructionError::BlobIdentityMismatch {
            layout: layout_id,
            expected,
            observed,
        });
    }
    Ok(())
}

fn emit_authenticated<W>(
    store: &ReferenceStore,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    output: &mut W,
) -> Result<BlobLength, ReconstructionError>
where
    W: Write + ?Sized,
{
    let mut written = 0_u64;
    for (index, entry) in layout.entries().iter().copied().enumerate() {
        let bytes =
            verified_chunk(store, layout_id, index, entry).map_err(reconstruction_chunk_error)?;
        write_chunk(output, layout_id, bytes, &mut written)?;
    }
    Ok(BlobLength::new(written))
}

fn write_chunk<W>(
    output: &mut W,
    layout_id: LayoutId,
    bytes: &[u8],
    written: &mut u64,
) -> Result<(), ReconstructionError>
where
    W: Write + ?Sized,
{
    write_all(output, bytes, written).map_err(|error| reconstruction_output_error(layout_id, error))
}

const fn reconstruction_chunk_error(error: ChunkVerificationError) -> ReconstructionError {
    match error {
        ChunkVerificationError::Missing {
            layout,
            index,
            requested,
        } => ReconstructionError::ChunkMissing {
            layout,
            index,
            requested,
        },
        ChunkVerificationError::Hash {
            layout,
            index,
            expected,
            source,
        } => ReconstructionError::ChunkHash {
            layout,
            index,
            expected,
            source,
        },
        ChunkVerificationError::IdentityMismatch {
            layout,
            index,
            expected,
            observed,
        } => ReconstructionError::ChunkIdentityMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}

fn reconstruction_output_error(layout: LayoutId, error: OutputWriteError) -> ReconstructionError {
    match error {
        OutputWriteError::WriteZero { bytes_written } => ReconstructionError::WriteZero {
            layout,
            bytes_written: BlobLength::new(bytes_written),
        },
        OutputWriteError::InvalidWriteCount {
            maximum,
            observed,
            bytes_written,
        } => ReconstructionError::InvalidWriteCount {
            layout,
            maximum,
            observed,
            bytes_written: BlobLength::new(bytes_written),
        },
        OutputWriteError::Write {
            bytes_written,
            source,
        } => ReconstructionError::Write {
            layout,
            bytes_written: BlobLength::new(bytes_written),
            source,
        },
        OutputWriteError::LengthOverflow {
            bytes_written,
            incoming,
        } => ReconstructionError::WrittenLengthOverflow {
            layout,
            bytes_written: BlobLength::new(bytes_written),
            incoming,
        },
    }
}

#[cfg(test)]
#[path = "reconstruction_tests.rs"]
mod tests;
