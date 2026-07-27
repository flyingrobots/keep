//! Whole-blob authentication followed by exact synchronous emission.

use std::io::{ErrorKind, Write};

use crate::{
    AdmittedLayout, BlobHasher, BlobId, BlobLength, ChunkId, LayoutDecodePolicy, LayoutEntry,
    LayoutId, ReferenceStore,
};

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
        let bytes = verified_chunk(store, layout_id, index, entry)?;
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
        let bytes = verified_chunk(store, layout_id, index, entry)?;
        write_chunk(output, layout_id, bytes, &mut written)?;
    }
    Ok(BlobLength::new(written))
}

fn verified_chunk(
    store: &ReferenceStore,
    layout_id: LayoutId,
    index: usize,
    entry: LayoutEntry,
) -> Result<&[u8], ReconstructionError> {
    let expected = entry.chunk_id();
    let bytes = store
        .chunk(expected)
        .ok_or(ReconstructionError::ChunkMissing {
            layout: layout_id,
            index,
            requested: expected,
        })?;
    let observed = ChunkId::hash_bytes(bytes).map_err(|source| ReconstructionError::ChunkHash {
        layout: layout_id,
        index,
        expected,
        source,
    })?;
    if observed != expected {
        return Err(ReconstructionError::ChunkIdentityMismatch {
            layout: layout_id,
            index,
            expected,
            observed,
        });
    }
    Ok(bytes)
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
    let mut remaining = bytes;
    while !remaining.is_empty() {
        match output.write(remaining) {
            Ok(0) => {
                return Err(ReconstructionError::WriteZero {
                    layout: layout_id,
                    bytes_written: BlobLength::new(*written),
                });
            }
            Ok(observed) => {
                let accepted = remaining.get(observed..).ok_or_else(|| {
                    ReconstructionError::InvalidWriteCount {
                        layout: layout_id,
                        maximum: remaining.len(),
                        observed,
                        bytes_written: BlobLength::new(*written),
                    }
                })?;
                *written = checked_written(layout_id, *written, observed)?;
                remaining = accepted;
            }
            Err(source) if source.kind() == ErrorKind::Interrupted => {}
            Err(source) => {
                return Err(ReconstructionError::Write {
                    layout: layout_id,
                    bytes_written: BlobLength::new(*written),
                    source,
                });
            }
        }
    }
    Ok(())
}

fn checked_written(
    layout_id: LayoutId,
    written: u64,
    incoming: usize,
) -> Result<u64, ReconstructionError> {
    let incoming_u64 =
        u64::try_from(incoming).map_err(|_source| ReconstructionError::WrittenLengthOverflow {
            layout: layout_id,
            bytes_written: written,
            incoming,
        })?;
    written
        .checked_add(incoming_u64)
        .ok_or(ReconstructionError::WrittenLengthOverflow {
            layout: layout_id,
            bytes_written: written,
            incoming,
        })
}

#[cfg(test)]
#[path = "reconstruction_tests.rs"]
mod tests;
