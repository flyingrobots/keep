//! Public exact byte-range read boundaries.

use std::io::Write;

use crate::{AdmittedLayout, BlobId, ByteRange, LayoutDecodePolicy, LayoutId, ReferenceStore};

use super::range_read_execution::read_admitted;
use super::{RangeReadError, RangeReadReceipt};

impl ReferenceStore {
    /// Reads one exact logical byte range from a committed blob.
    ///
    /// The lowest canonical committed [`LayoutId`] is chosen deterministically
    /// when more than one layout names the blob. Only chunks overlapping
    /// `requested` are loaded. Every selected complete chunk is authenticated
    /// before any output, then reauthenticated immediately before its
    /// overlapping bytes are emitted.
    ///
    /// The receipt proves the requested bytes came from authenticated chunks
    /// under an admitted layout. It does not prove the complete blob identity,
    /// unrequested chunks, or unselected storage-profile boundaries. This
    /// synchronous operation allocates no adapter-owned heap memory and does
    /// not flush `output`.
    ///
    /// # Errors
    ///
    /// Returns [`RangeReadError`] for absent state, an out-of-bounds range,
    /// missing or mismatched selected chunks, checked slicing or accounting
    /// failure, broken writer behavior, or output I/O failure.
    pub fn read_range<W>(
        &self,
        target: BlobId,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<RangeReadReceipt, RangeReadError>
    where
        W: Write + ?Sized,
    {
        let layout_id = self
            .first_layout_id(target)
            .ok_or(RangeReadError::BlobMissing { requested: target })?;
        self.read_layout_range(layout_id, requested, output)
    }

    /// Reads one exact logical byte range through a committed layout.
    ///
    /// Verification, allocation, and output behavior are identical to
    /// [`ReferenceStore::read_range`].
    ///
    /// # Errors
    ///
    /// Returns [`RangeReadError`] for an absent layout, invalid range, missing
    /// or mismatched selected content, checked slicing or accounting failure,
    /// broken writer behavior, or output I/O failure.
    pub fn read_layout_range<W>(
        &self,
        layout_id: LayoutId,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<RangeReadReceipt, RangeReadError>
    where
        W: Write + ?Sized,
    {
        let layout = self
            .layout(layout_id)
            .ok_or(RangeReadError::LayoutMissing {
                requested: layout_id,
            })?;
        read_admitted(self, layout_id, layout, requested, output)
    }

    /// Reads one exact range through a caller-supplied admitted layout.
    ///
    /// The layout need not be published in this store, but every selected
    /// chunk must be present and exact. Calculating the canonical layout
    /// identity transiently materializes one record bounded by the admitted
    /// layout's protocol entry limit. The logical blob itself is never
    /// materialized.
    ///
    /// # Errors
    ///
    /// Returns [`RangeReadError`] for canonical encoding failure, an invalid
    /// range, missing or mismatched selected content, checked slicing or
    /// accounting failure, broken writer behavior, or output I/O failure.
    pub fn read_admitted_layout_range<W>(
        &self,
        layout: &AdmittedLayout,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<RangeReadReceipt, RangeReadError>
    where
        W: Write + ?Sized,
    {
        let layout_id = layout
            .encode_record()
            .map_err(RangeReadError::LayoutEncoding)?
            .id();
        read_admitted(self, layout_id, layout, requested, output)
    }

    /// Decodes one canonical layout record and reads an exact logical range.
    ///
    /// Bounded decoding and admission allocate entry metadata within `policy`
    /// before chunk lookup or output. The complete logical blob is never
    /// materialized. Selected-chunk verification and output then follow
    /// [`ReferenceStore::read_admitted_layout_range`].
    ///
    /// # Errors
    ///
    /// Returns [`RangeReadError`] for malformed, noncanonical, or
    /// policy-exceeding layout bytes; an invalid range; absent or mismatched
    /// selected content; checked slicing or accounting failure; broken writer
    /// behavior; or output I/O failure.
    pub fn read_record_range<W>(
        &self,
        encoded: &[u8],
        policy: LayoutDecodePolicy,
        requested: ByteRange,
        output: &mut W,
    ) -> Result<RangeReadReceipt, RangeReadError>
    where
        W: Write + ?Sized,
    {
        let layout =
            AdmittedLayout::decode_record(encoded, policy).map_err(RangeReadError::LayoutDecode)?;
        self.read_admitted_layout_range(&layout, requested, output)
    }
}
