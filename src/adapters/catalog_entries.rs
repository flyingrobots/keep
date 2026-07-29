//! Revalidating iterator over one checksummed catalog's fixed-width entries.

use std::slice::ChunksExact;

use super::{
    CatalogDecodeError, DecodedCatalogEntry, catalog_entry_decoder, catalog_header_decoder,
};

pub(super) struct CatalogEntries<'a> {
    chunks: ChunksExact<'a, u8>,
    next_index: u64,
    entry_count: u64,
}

impl<'a> CatalogEntries<'a> {
    pub(super) fn new(encoded: &'a [u8], entry_count: u64) -> Result<Self, CatalogDecodeError> {
        let entries_end = encoded
            .len()
            .checked_sub(catalog_header_decoder::TRAILER_LENGTH)
            .ok_or(CatalogDecodeError::MinimumLength {
                minimum: catalog_header_decoder::MINIMUM_LENGTH,
                observed: encoded.len(),
            })?;
        let entries = encoded
            .get(catalog_header_decoder::HEADER_LENGTH_BYTES..entries_end)
            .ok_or(CatalogDecodeError::MinimumLength {
                minimum: catalog_header_decoder::MINIMUM_LENGTH,
                observed: encoded.len(),
            })?;
        Ok(Self {
            chunks: entries.chunks_exact(catalog_entry_decoder::ENCODED_LENGTH),
            next_index: 0,
            entry_count,
        })
    }
}

impl Iterator for CatalogEntries<'_> {
    type Item = Result<DecodedCatalogEntry, CatalogDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let encoded = self.chunks.next()?;
        let index = self.next_index;
        self.next_index = match index.checked_add(1) {
            Some(next) => next,
            None => {
                return Some(Err(CatalogDecodeError::LengthArithmetic {
                    entry_count: self.entry_count,
                }));
            }
        };
        Some(
            catalog_entry_decoder::decode(encoded)
                .map_err(|source| CatalogDecodeError::Entry { index, source }),
        )
    }
}
