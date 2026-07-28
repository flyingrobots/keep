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
    /// unrequested chunks, or any storage-profile boundary. This
    /// synchronous operation allocates no adapter-owned heap memory and does
    /// not flush `output`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// use keep::{
    ///     ByteLength, ByteOffset, ByteRange, LayoutEntryLimit, ReferenceStore,
    ///     ReferenceStoreCapacity,
    /// };
    ///
    /// let bytes = b"one exact requested range";
    /// let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    /// let mut source = Cursor::new(bytes);
    /// let published = store
    ///     .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
    ///     .commit(&mut store)?;
    /// let requested = ByteRange::new(ByteOffset::new(4), ByteLength::new(5))?;
    /// let mut output = Vec::new();
    ///
    /// let receipt = store.read_range(published.target(), requested, &mut output)?;
    ///
    /// assert_eq!(output, b"exact");
    /// assert_eq!(receipt.requested(), requested);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Resolves a caller-supplied admitted layout to one committed range.
    ///
    /// The supplied layout is used only to calculate its canonical identity,
    /// which must name a committed layout in this store. Range planning,
    /// receipt coordinates, and chunk lookup use that committed layout, so a
    /// caller cannot associate stored bytes with an uncommitted target.
    /// Calculating the identity transiently materializes one record bounded by
    /// the admitted layout's protocol entry limit. The logical blob itself is
    /// never materialized.
    ///
    /// # Errors
    ///
    /// Returns [`RangeReadError`] for canonical encoding failure, an absent
    /// committed layout, an invalid range, missing or mismatched selected
    /// content, checked slicing or accounting failure, broken writer behavior,
    /// or output I/O failure.
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
        self.read_layout_range(layout_id, requested, output)
    }

    /// Decodes one canonical layout record and reads an exact logical range.
    ///
    /// Bounded decoding and admission allocate entry metadata within `policy`.
    /// The decoded record is used only to resolve a canonical identity that
    /// must name a committed layout in this store. The complete logical blob
    /// is never materialized. Selected-chunk verification and output then
    /// follow [`ReferenceStore::read_admitted_layout_range`].
    ///
    /// # Errors
    ///
    /// Returns [`RangeReadError`] for malformed, noncanonical, or
    /// policy-exceeding layout bytes; an absent committed layout; an invalid
    /// range; absent or mismatched selected content; checked slicing or
    /// accounting failure; broken writer behavior; or output I/O failure.
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
