//! This module owns whole-blob authentication before synchronous emission.

use super::chunk_verification::{ChunkSource, emitted_chunk, verified_chunk};
use super::output_write::write_all;
use super::profile_verification::ProfileVerifier;
use super::{ReconstructionFailure, ReconstructionReceipt};
use crate::{AdmittedLayout, BlobHasher, BlobLength, LayoutId};
use std::io::Write;

/// Authenticates the complete blob `layout` names against `store`, then
/// emits it: the one reconstruction core every view shares.
pub(crate) fn reconstruct_admitted<S, W>(
    store: &S,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    output: &mut W,
) -> Result<ReconstructionReceipt, ReconstructionFailure>
where
    S: ChunkSource + ?Sized,
    W: Write + ?Sized,
{
    verify_complete_blob(store, layout_id, layout)?;
    let written = emit_authenticated(store, layout_id, layout, output)?;
    let expected = layout.target().logical_length();
    if written != expected {
        return Err(ReconstructionFailure::WrittenLengthMismatch {
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

fn verify_complete_blob<S: ChunkSource + ?Sized>(
    store: &S,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
) -> Result<(), ReconstructionFailure> {
    let mut hasher = BlobHasher::new();
    let mut profile = ProfileVerifier::new(layout_id, layout)?;
    for (index, entry) in layout.entries().iter().copied().enumerate() {
        let bytes =
            verified_chunk(store, layout_id, index, entry).map_err(ReconstructionFailure::Chunk)?;
        profile.feed(bytes)?;
        hasher
            .update(bytes)
            .map_err(ReconstructionFailure::BlobHash)?;
    }
    profile.finish()?;
    let observed = hasher.finish();
    let expected = layout.target();
    if observed != expected {
        return Err(ReconstructionFailure::BlobIdentityMismatch {
            layout: layout_id,
            expected,
            observed,
        });
    }
    Ok(())
}

fn emit_authenticated<S, W>(
    store: &S,
    layout_id: LayoutId,
    layout: &AdmittedLayout,
    output: &mut W,
) -> Result<BlobLength, ReconstructionFailure>
where
    S: ChunkSource + ?Sized,
    W: Write + ?Sized,
{
    let mut written = 0_u64;
    for (index, entry) in layout.entries().iter().copied().enumerate() {
        let bytes =
            emitted_chunk(store, layout_id, index, entry).map_err(ReconstructionFailure::Chunk)?;
        write_chunk(output, layout_id, bytes, &mut written)?;
    }
    Ok(BlobLength::new(written))
}

fn write_chunk<W>(
    output: &mut W,
    layout_id: LayoutId,
    bytes: &[u8],
    written: &mut u64,
) -> Result<(), ReconstructionFailure>
where
    W: Write + ?Sized,
{
    write_all(output, bytes, written).map_err(|error| ReconstructionFailure::Output {
        layout: layout_id,
        source: error,
    })
}
