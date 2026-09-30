//! Independent recomputation of every record's own integrity trailer, so a
//! mutation can reach the check behind the checksum.

use std::io;

use crate::support::{domain_hash, invalid_corpus, patch};

/// `framed_blake3_v1(D, B)` from the version-1 segment specification.
fn framed_blake3_v1(domain: &[u8], body: &[u8]) -> Result<[u8; 32], io::Error> {
    let length =
        u64::try_from(body.len()).map_err(|_source| invalid_corpus("framed body exceeds u64"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&1_u16.to_be_bytes());
    hasher.update(&[1_u8]);
    hasher.update(body);
    hasher.update(&length.to_be_bytes());
    Ok(*hasher.finalize().as_bytes())
}

fn slice(bytes: &[u8], start: usize, end: usize) -> Result<&[u8], io::Error> {
    bytes
        .get(start..end)
        .ok_or_else(|| invalid_corpus("recipe span is out of bounds"))
}

fn u64_at(bytes: &[u8], offset: usize) -> Result<u64, io::Error> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| invalid_corpus("field offset overflows"))?;
    let field: [u8; 8] = slice(bytes, offset, end)?
        .try_into()
        .map_err(|_source| invalid_corpus("field width mismatch"))?;
    Ok(u64::from_be_bytes(field))
}

/// Recomputes the seal checksum and the segment digest of one complete
/// segment whose seal is its last 128 bytes.
pub(crate) fn reseal_segment(bytes: &mut [u8]) -> Result<(), io::Error> {
    let seal_offset = bytes
        .len()
        .checked_sub(128)
        .ok_or_else(|| invalid_corpus("segment shorter than its seal"))?;
    let digest_offset = seal_offset.saturating_add(64);
    let checksum_offset = seal_offset.saturating_add(96);
    let digest = framed_blake3_v1(b"KEEP:SEGMENT:DIGEST\0", slice(bytes, 0, digest_offset)?)?;
    patch(bytes, digest_offset, &digest)?;
    let checksum = framed_blake3_v1(
        b"KEEP:SEGMENT:SEAL:SUM\0",
        slice(bytes, seal_offset, checksum_offset)?,
    )?;
    patch(bytes, checksum_offset, &checksum)
}

/// Recomputes the first record's checksum (the record at byte 64) and then
/// reseals the segment.
pub(crate) fn rechecksum_first_record(bytes: &mut [u8]) -> Result<(), io::Error> {
    let record_length = usize::try_from(u64_at(bytes, 96)?)
        .map_err(|_source| invalid_corpus("record length exceeds host width"))?;
    let end = 64_usize
        .checked_add(record_length)
        .ok_or_else(|| invalid_corpus("record span overflows"))?;
    let checksum_offset = end
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("record shorter than its checksum"))?;
    let checksum = framed_blake3_v1(b"KEEP:SEG:RECORD:SUM\0", slice(bytes, 64, checksum_offset)?)?;
    patch(bytes, checksum_offset, &checksum)?;
    reseal_segment(bytes)
}

/// Recomputes a catalog's checksum and digest trailer.
pub(crate) fn reseal_catalog(bytes: &mut [u8]) -> Result<(), io::Error> {
    let digest_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("catalog shorter than its digest"))?;
    let checksum_offset = digest_offset
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("catalog shorter than its trailer"))?;
    let checksum = framed_blake3_v1(b"KEEP:CATALOG:SUM\0", slice(bytes, 0, checksum_offset)?)?;
    patch(bytes, checksum_offset, &checksum)?;
    let digest = framed_blake3_v1(b"KEEP:CATALOG:DIGEST\0", slice(bytes, 0, digest_offset)?)?;
    patch(bytes, digest_offset, &digest)
}

/// Recomputes a publication head's checksum.
pub(crate) fn reseal_publication_head(bytes: &mut [u8]) -> Result<(), io::Error> {
    let checksum = framed_blake3_v1(b"KEEP:CATHEAD:SUM\0", slice(bytes, 0, 96)?)?;
    patch(bytes, 96, &checksum)
}

/// Recomputes a fixed-width version-2 record's trailing checksum under
/// `domain` over every byte before it.
pub(crate) fn reseal_fixed_v2(bytes: &mut [u8], domain: &[u8]) -> Result<(), io::Error> {
    let checksum_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("record shorter than its checksum"))?;
    let checksum = domain_hash(domain, slice(bytes, 0, checksum_offset)?);
    patch(bytes, checksum_offset, &checksum)
}

/// Recomputes a variable-length version-2 record's digest-then-checksum
/// trailer.
pub(crate) fn reseal_digested_v2(
    bytes: &mut [u8],
    digest_domain: &[u8],
    checksum_domain: &[u8],
) -> Result<(), io::Error> {
    let checksum_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("record shorter than its checksum"))?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("record shorter than its trailer"))?;
    let digest = domain_hash(digest_domain, slice(bytes, 0, digest_offset)?);
    patch(bytes, digest_offset, &digest)?;
    let checksum = domain_hash(checksum_domain, slice(bytes, 0, checksum_offset)?);
    patch(bytes, checksum_offset, &checksum)
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<usize, io::Error> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| invalid_corpus("field offset overflows"))?;
    let field: [u8; 2] = slice(bytes, offset, end)?
        .try_into()
        .map_err(|_source| invalid_corpus("field width mismatch"))?;
    Ok(usize::from(u16::from_be_bytes(field)))
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<[u8; 4], io::Error> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| invalid_corpus("field offset overflows"))?;
    slice(bytes, offset, end)?
        .try_into()
        .map_err(|_source| invalid_corpus("field width mismatch"))
}

/// Recomputes one `domain || count || body` set digest into `digest_offset`.
fn reseal_set_digest(
    bytes: &mut [u8],
    domain: &[u8],
    count_offset: usize,
    digest_offset: usize,
    body_offset: usize,
    body_length: usize,
) -> Result<(), io::Error> {
    let count = u32_at(bytes, count_offset)?;
    let end = body_offset
        .checked_add(body_length)
        .ok_or_else(|| invalid_corpus("set body overflows"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&count);
    hasher.update(slice(bytes, body_offset, end)?);
    let digest = *hasher.finalize().as_bytes();
    patch(bytes, digest_offset, &digest)
}

/// The width of the variable body a header-declared count covers, from the
/// record's total length minus its header and 64-byte trailer.
fn body_length(bytes: &[u8], body_offset: usize) -> Result<usize, io::Error> {
    bytes
        .len()
        .checked_sub(body_offset)
        .and_then(|length| length.checked_sub(64))
        .ok_or_else(|| invalid_corpus("record shorter than its body"))
}

/// Applies the record's complete recompute recipe: inner set digests, then
/// the record's own digest and checksum trailer.
///
/// # Errors
///
/// Returns a corpus error for an unknown record or a malformed span.
pub(crate) fn recompute(record: &str, bytes: &mut [u8]) -> Result<(), io::Error> {
    match record {
        "retention-root" => {
            let namespace_length = u16_at(bytes, 40)?;
            let anchors = 192_usize
                .checked_add(namespace_length)
                .ok_or_else(|| invalid_corpus("namespace length overflows"))?;
            let length = body_length(bytes, anchors)?;
            reseal_set_digest(
                bytes,
                b"keep.retention-anchor-set/v2\0",
                44,
                148,
                anchors,
                length,
            )?;
        }
        "retention-manifest" => {
            let length = body_length(bytes, 160)?;
            reseal_set_digest(
                bytes,
                b"keep.retention-manifest-entries/v2\0",
                44,
                80,
                160,
                length,
            )?;
        }
        "gc-intent" => {
            let length = body_length(bytes, 320)?;
            reseal_set_digest(bytes, b"keep.gc-candidate-set/v2\0", 44, 288, 320, length)?;
        }
        _ => {}
    }
    recompute_trailer(record, bytes)
}

/// Applies only the record's own digest and checksum trailer recipe.
///
/// # Errors
///
/// Returns a corpus error for an unknown record or a malformed span.
pub(crate) fn recompute_trailer(record: &str, bytes: &mut [u8]) -> Result<(), io::Error> {
    match record {
        "segment-header" | "segment-seal" | "segment" => reseal_segment(bytes),
        "segment-record" => rechecksum_first_record(bytes),
        "catalog" | "catalog-entry" | "catalog-binding" => reseal_catalog(bytes),
        "publication-head" | "head-binding" => reseal_publication_head(bytes),
        "format-marker" => reseal_fixed_v2(bytes, b"keep.segment-store-marker-checksum/v2\0"),
        "migration-intent" => reseal_fixed_v2(bytes, b"keep.store-migration-intent-checksum/v2\0"),
        "migration-receipt" => {
            reseal_fixed_v2(bytes, b"keep.store-migration-receipt-checksum/v2\0")
        }
        "retention-head" => reseal_fixed_v2(bytes, b"keep.retention-head-checksum/v2\0"),
        "gc-receipt" => reseal_fixed_v2(bytes, b"keep.gc-retirement-receipt-checksum/v2\0"),
        "disposition" => reseal_fixed_v2(bytes, b"keep.recovery-disposition-receipt-checksum/v2\0"),
        "retention-root" => reseal_digested_v2(
            bytes,
            b"keep.retention-root/v2\0",
            b"keep.retention-root-checksum/v2\0",
        ),
        "retention-manifest" => reseal_digested_v2(
            bytes,
            b"keep.retention-manifest/v2\0",
            b"keep.retention-manifest-checksum/v2\0",
        ),
        "gc-intent" => reseal_digested_v2(
            bytes,
            b"keep.gc-retirement-intent/v2\0",
            b"keep.gc-retirement-intent-checksum/v2\0",
        ),
        _ => Err(invalid_corpus("unknown record for recompute")),
    }
}

/// Recomputes only the outermost checksum, leaving an inner digest as
/// mutated, so a digest field's own refusal is reachable.
pub(crate) fn recompute_checksum_only(record: &str, bytes: &mut [u8]) -> Result<(), io::Error> {
    match record {
        "segment-seal" => {
            let seal_offset = bytes
                .len()
                .checked_sub(128)
                .ok_or_else(|| invalid_corpus("segment shorter than its seal"))?;
            let checksum_offset = seal_offset.saturating_add(96);
            let checksum = framed_blake3_v1(
                b"KEEP:SEGMENT:SEAL:SUM\0",
                slice(bytes, seal_offset, checksum_offset)?,
            )?;
            patch(bytes, checksum_offset, &checksum)
        }
        "catalog" => {
            let digest_offset = bytes
                .len()
                .checked_sub(32)
                .ok_or_else(|| invalid_corpus("catalog shorter than its digest"))?;
            let checksum_offset = digest_offset
                .checked_sub(32)
                .ok_or_else(|| invalid_corpus("catalog shorter than its trailer"))?;
            let checksum =
                framed_blake3_v1(b"KEEP:CATALOG:SUM\0", slice(bytes, 0, checksum_offset)?)?;
            patch(bytes, checksum_offset, &checksum)
        }
        "retention-root" => reseal_fixed_v2(bytes, b"keep.retention-root-checksum/v2\0"),
        "retention-manifest" => reseal_fixed_v2(bytes, b"keep.retention-manifest-checksum/v2\0"),
        "gc-intent" => reseal_fixed_v2(bytes, b"keep.gc-retirement-intent-checksum/v2\0"),
        other => recompute_trailer(other, bytes),
    }
}
