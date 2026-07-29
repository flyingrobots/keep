//! Canonical version-1 catalog-header emission.

use super::{catalog_decoder, catalog_header_decoder};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

pub(super) const fn encode(
    generation: CatalogGeneration,
    predecessor: Option<CatalogDigest>,
    entry_count: u64,
    catalog_length: CatalogLength,
) -> [u8; catalog_header_decoder::HEADER_LENGTH_BYTES] {
    let mut encoded = [0_u8; catalog_header_decoder::HEADER_LENGTH_BYTES];
    let (magic, remaining) = encoded.split_at_mut(16);
    magic.copy_from_slice(&catalog_decoder::MAGIC);
    let (version, remaining) = remaining.split_at_mut(2);
    version.copy_from_slice(&catalog_decoder::VERSION.to_be_bytes());
    let (flags, remaining) = remaining.split_at_mut(2);
    flags.copy_from_slice(&catalog_decoder::FLAGS.to_be_bytes());
    let (header_length, remaining) = remaining.split_at_mut(2);
    header_length.copy_from_slice(&catalog_header_decoder::HEADER_LENGTH.to_be_bytes());
    let (entry_length, remaining) = remaining.split_at_mut(2);
    entry_length.copy_from_slice(&catalog_header_decoder::ENTRY_LENGTH.to_be_bytes());
    let (generation_field, remaining) = remaining.split_at_mut(8);
    generation_field.copy_from_slice(&generation.get().to_be_bytes());
    let (predecessor_field, remaining) = remaining.split_at_mut(32);
    if let Some(digest) = predecessor {
        predecessor_field.copy_from_slice(digest.as_bytes());
    }
    let (entry_count_field, remaining) = remaining.split_at_mut(8);
    entry_count_field.copy_from_slice(&entry_count.to_be_bytes());
    let (catalog_length_field, remaining) = remaining.split_at_mut(8);
    catalog_length_field.copy_from_slice(&catalog_length.get().to_be_bytes());
    let (checksum_algorithm, remaining) = remaining.split_at_mut(1);
    checksum_algorithm.copy_from_slice(&[catalog_decoder::ALGORITHM]);
    let (digest_algorithm, _reserved) = remaining.split_at_mut(1);
    digest_algorithm.copy_from_slice(&[catalog_decoder::ALGORITHM]);
    encoded
}
