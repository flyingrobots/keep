//! Canonical publication-head emission from one verified catalog.

use super::{CanonicalPublicationHead, ChecksummedCatalog, publication_head_decoder as format};

pub(super) fn encode(catalog: ChecksummedCatalog<'_>) -> CanonicalPublicationHead {
    let mut encoded = [0_u8; format::ENCODED_LENGTH];
    let (covered, checksum) = encoded.split_at_mut(format::CHECKSUM_INPUT_LENGTH);
    let (magic, remaining) = covered.split_at_mut(16);
    magic.copy_from_slice(&format::MAGIC);
    let (version, remaining) = remaining.split_at_mut(2);
    version.copy_from_slice(&format::VERSION.to_be_bytes());
    let (flags, remaining) = remaining.split_at_mut(2);
    flags.copy_from_slice(&format::FLAGS.to_be_bytes());
    let (head_length, remaining) = remaining.split_at_mut(2);
    head_length.copy_from_slice(&format::HEAD_LENGTH.to_be_bytes());
    let (checksum_algorithm, remaining) = remaining.split_at_mut(1);
    checksum_algorithm.copy_from_slice(&[format::ALGORITHM]);
    let (digest_algorithm, remaining) = remaining.split_at_mut(1);
    digest_algorithm.copy_from_slice(&[format::ALGORITHM]);
    let (generation, remaining) = remaining.split_at_mut(8);
    generation.copy_from_slice(&catalog.generation().get().to_be_bytes());
    let (catalog_length, remaining) = remaining.split_at_mut(8);
    catalog_length.copy_from_slice(&catalog.length().get().to_be_bytes());
    let (catalog_digest, remaining) = remaining.split_at_mut(32);
    catalog_digest.copy_from_slice(catalog.digest().as_bytes());
    let (_reserved, _complete) = remaining.split_at_mut(24);
    checksum.copy_from_slice(&format::checksum(covered));
    CanonicalPublicationHead::admitted(encoded)
}
