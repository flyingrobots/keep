//! This module owns caller-supplied layout ingress for fenced durable reads.

use std::io::Write;

use super::snapshot::CatalogChunks;
use super::{
    DurableRangeReadReceipt, DurableReadError, DurableReconstructionReceipt, DurableSnapshot,
};
use crate::authenticated_read::reconstruct_admitted;
use crate::{AdmittedLayout, ByteRange, LayoutDecodePolicy, RangeReadError, ReconstructionError};

impl DurableSnapshot {
    /// Reconstructs through a caller-supplied admitted semantic layout.
    ///
    /// The layout need not be catalogued, but every chunk it names must exist
    /// in this immutable catalog. The core authenticates the complete blob and
    /// profile before output. Canonical identity calculation materializes one
    /// bounded layout record; catalog re-admission has the costs documented on
    /// [`DurableSnapshot`]. No whole-blob buffer or durability effect is added.
    ///
    /// # Errors
    ///
    /// Returns the precise layout-encoding, catalog, reconstruction or output
    /// failure, retaining the original source and accepted-prefix accounting.
    pub fn reconstruct_admitted_layout<W>(
        &self,
        layout: &AdmittedLayout,
        output: &mut W,
    ) -> Result<DurableReconstructionReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let identity = layout
            .encode_record()
            .map_err(|source| {
                DurableReadError::Reconstruction(Box::new(ReconstructionError::LayoutEncoding(
                    source,
                )))
            })?
            .id();
        let catalog = self.catalog()?;
        reconstruct_admitted(&CatalogChunks::new(&catalog), identity, layout, output)
            .map(|receipt| DurableReconstructionReceipt::new(receipt, self.view()))
            .map_err(|source| DurableReadError::Reconstruction(Box::new(source.into())))
    }

    /// Decodes a bounded canonical layout record and reconstructs its exact blob.
    ///
    /// Decoding allocates bounded entry metadata under `policy` before content
    /// lookup or emission. The allocation, verification and blocking contract
    /// then follows [`Self::reconstruct_admitted_layout`].
    ///
    /// # Errors
    ///
    /// Returns exact decoder refusal for invalid or policy-exceeding input,
    /// otherwise the errors of [`Self::reconstruct_admitted_layout`].
    pub fn reconstruct_record<W>(
        &self,
        encoded: &[u8],
        policy: LayoutDecodePolicy,
        output: &mut W,
    ) -> Result<DurableReconstructionReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let layout = AdmittedLayout::decode_record(encoded, policy).map_err(|source| {
            DurableReadError::Reconstruction(Box::new(ReconstructionError::LayoutDecode(source)))
        })?;
        self.reconstruct_admitted_layout(&layout, output)
    }

    /// Resolves a supplied layout identity to an exact catalogued range.
    ///
    /// The supplied layout only determines its canonical identity. That exact
    /// layout must exist in the catalog; range planning uses the committed
    /// record rather than caller-supplied target claims. Identity calculation
    /// allocates one bounded canonical record; remaining costs follow
    /// [`Self::read_layout_range`]. No complete blob buffer is created.
    ///
    /// # Errors
    ///
    /// Returns exact encoding or missing-layout refusal, otherwise the errors
    /// of [`Self::read_layout_range`], without substituting another layout.
    pub fn read_admitted_layout_range<W>(
        &self,
        layout: &AdmittedLayout,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<DurableRangeReadReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let identity = layout
            .encode_record()
            .map_err(|source| {
                DurableReadError::RangeRead(Box::new(RangeReadError::LayoutEncoding(source)))
            })?
            .id();
        self.read_layout_range(identity, requested, output)
    }

    /// Decodes a canonical layout record and resolves its exact catalogued range.
    ///
    /// Decoding admits bounded metadata under `policy`; then
    /// [`Self::read_admitted_layout_range`] establishes catalog membership and
    /// performs verification and blocking output with the same allocation costs.
    ///
    /// # Errors
    ///
    /// Returns exact decoder refusal before catalog lookup or output,
    /// otherwise the errors of [`Self::read_admitted_layout_range`].
    pub fn read_record_range<W>(
        &self,
        encoded: &[u8],
        policy: LayoutDecodePolicy,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<DurableRangeReadReceipt, DurableReadError>
    where
        W: Write + ?Sized,
    {
        let layout = AdmittedLayout::decode_record(encoded, policy).map_err(|source| {
            DurableReadError::RangeRead(Box::new(RangeReadError::LayoutDecode(source)))
        })?;
        self.read_admitted_layout_range(&layout, requested, output)
    }
}
