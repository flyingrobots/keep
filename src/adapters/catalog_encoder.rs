//! Bounded canonical catalog emission from admitted segments.

use super::checksummed_catalog::CatalogMetadata;
use super::{
    AdmittedSegment, CanonicalCatalog, CatalogEncodeError, CatalogEncodingEntry,
    ChecksummedCatalog, catalog_decoder, catalog_header_decoder, catalog_header_encoding,
    catalog_integrity,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

const TRAILER_LENGTH: u64 = 64;
const DIGEST_LENGTH: u64 = 32;

pub(super) fn encode(
    generation: CatalogGeneration,
    predecessor: Option<CatalogDigest>,
    segments: &[AdmittedSegment<'_>],
) -> Result<CanonicalCatalog, CatalogEncodeError> {
    validate_predecessor(generation, predecessor)?;
    let entry_count = entry_count(segments)?;
    let catalog_length = catalog_length(entry_count)?;
    let mut entries = collect_entries(segments, entry_count)?;
    entries.sort_unstable_by_key(CatalogEncodingEntry::identity);
    refuse_duplicates(&entries)?;
    let encoded = encode_catalog(
        generation,
        predecessor,
        entry_count,
        catalog_length,
        &entries,
    )?;
    admit_encoded(encoded)
}

fn validate_predecessor(
    generation: CatalogGeneration,
    predecessor: Option<CatalogDigest>,
) -> Result<(), CatalogEncodeError> {
    if generation.get() == 1 {
        return predecessor.map_or(Ok(()), |observed| {
            Err(CatalogEncodeError::UnexpectedPredecessor { observed })
        });
    }
    match predecessor {
        Some(digest) if digest.as_bytes() != &[0_u8; 32] => Ok(()),
        Some(_) | None => Err(CatalogEncodeError::MissingPredecessor { generation }),
    }
}

fn entry_count(segments: &[AdmittedSegment<'_>]) -> Result<u64, CatalogEncodeError> {
    let mut count = 0_u64;
    for segment in segments {
        count = count
            .checked_add(u64::from(segment.record_count()))
            .ok_or(CatalogEncodeError::EntryCountArithmetic)?;
    }
    if count > catalog_decoder::MAXIMUM_ENTRY_COUNT {
        return Err(CatalogEncodeError::EntryCountOutOfBounds {
            maximum: catalog_decoder::MAXIMUM_ENTRY_COUNT,
            observed: count,
        });
    }
    Ok(count)
}

fn catalog_length(entry_count: u64) -> Result<CatalogLength, CatalogEncodeError> {
    let entry_bytes = entry_count
        .checked_mul(u64::from(catalog_header_decoder::ENTRY_LENGTH))
        .ok_or(CatalogEncodeError::EntryCountArithmetic)?;
    let length = entry_bytes
        .checked_add(u64::from(catalog_header_decoder::HEADER_LENGTH))
        .and_then(|value| value.checked_add(TRAILER_LENGTH))
        .ok_or(CatalogEncodeError::EntryCountArithmetic)?;
    CatalogLength::new(length).map_err(|source| CatalogEncodeError::CatalogLength { source })
}

fn collect_entries(
    segments: &[AdmittedSegment<'_>],
    entry_count: u64,
) -> Result<Vec<CatalogEncodingEntry>, CatalogEncodeError> {
    let capacity =
        usize::try_from(entry_count).map_err(|_source| CatalogEncodeError::HostLength {
            observed: entry_count,
        })?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(capacity)
        .map_err(|source| CatalogEncodeError::Allocation {
            entry_count,
            source,
        })?;
    for segment in segments {
        collect_segment_entries(segment, &mut entries)?;
    }
    Ok(entries)
}

fn collect_segment_entries(
    segment: &AdmittedSegment<'_>,
    entries: &mut Vec<CatalogEncodingEntry>,
) -> Result<(), CatalogEncodeError> {
    let digest = segment.digest();
    let mut cursor = segment.record_cursor();
    while let Some(located) =
        cursor
            .next_record()
            .map_err(|source| CatalogEncodeError::Segment {
                segment_digest: digest,
                source: Box::new(source),
            })?
    {
        entries.push(CatalogEncodingEntry::from_located(digest, &located));
    }
    cursor
        .finish()
        .map_err(|source| CatalogEncodeError::Segment {
            segment_digest: digest,
            source: Box::new(source),
        })
}

fn refuse_duplicates(entries: &[CatalogEncodingEntry]) -> Result<(), CatalogEncodeError> {
    for pair in entries.windows(2) {
        let [first, second] = pair else {
            continue;
        };
        if first.identity() == second.identity() {
            return Err(CatalogEncodeError::DuplicateIdentity {
                identity: first.identity(),
            });
        }
    }
    Ok(())
}

fn encode_catalog(
    generation: CatalogGeneration,
    predecessor: Option<CatalogDigest>,
    entry_count: u64,
    catalog_length: CatalogLength,
    entries: &[CatalogEncodingEntry],
) -> Result<Vec<u8>, CatalogEncodeError> {
    let host_length = usize::try_from(catalog_length.get()).map_err(|_source| {
        CatalogEncodeError::HostLength {
            observed: catalog_length.get(),
        }
    })?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(host_length)
        .map_err(|source| CatalogEncodeError::Allocation {
            entry_count,
            source,
        })?;
    encoded.extend_from_slice(&catalog_header_encoding::encode(
        generation,
        predecessor,
        entry_count,
        catalog_length,
    ));
    for entry in entries {
        encoded.extend_from_slice(&entry.encode());
    }
    let checksum_length = catalog_length
        .get()
        .checked_sub(TRAILER_LENGTH)
        .ok_or(CatalogEncodeError::EntryCountArithmetic)?;
    let checksum = catalog_integrity::checksum(&encoded, checksum_length);
    encoded.extend_from_slice(&checksum);
    let digest_length = catalog_length
        .get()
        .checked_sub(DIGEST_LENGTH)
        .ok_or(CatalogEncodeError::EntryCountArithmetic)?;
    let digest = catalog_integrity::digest(&encoded, digest_length);
    encoded.extend_from_slice(&digest);
    Ok(encoded)
}

fn admit_encoded(encoded: Vec<u8>) -> Result<CanonicalCatalog, CatalogEncodeError> {
    let verified = ChecksummedCatalog::decode(&encoded)
        .map_err(|source| CatalogEncodeError::Verification { source })?;
    let metadata = CatalogMetadata::new(
        verified.generation(),
        verified.previous_catalog_digest(),
        verified.entry_count(),
        verified.length(),
    );
    let digest = verified.digest();
    Ok(CanonicalCatalog::admitted(encoded, metadata, digest))
}
