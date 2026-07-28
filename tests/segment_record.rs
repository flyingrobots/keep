//! Public complete segment-record decoding and admission laws.

#[path = "segment_record/admission_laws.rs"]
mod admission_laws;
#[path = "segment_record/framing_laws.rs"]
mod framing_laws;
mod support;

use std::error::Error;

use keep::{
    AdmittedLayout, AdmittedSegmentRecord, ChecksummedSegmentRecord, LayoutDecodePolicy,
    LayoutEntryLimit, SegmentRecordIdentity,
};
use support::decode_hex;

const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const ONE_ZERO_BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");

#[test]
fn canonical_chunk_record_matches_frozen_checksum_and_admission() -> Result<(), Box<dyn Error>> {
    let record = record_bytes(ONE_ZERO_SEGMENT_HEX, 64, 145)?;
    let checksummed = ChecksummedSegmentRecord::decode(&record)?;

    assert_eq!(checksummed.payload(), &[0]);
    assert_eq!(
        checksummed.checksum().as_bytes(),
        record.get(113..145).ok_or("chunk checksum is missing")?
    );
    let admitted = checksummed.admit(LayoutEntryLimit::MAXIMUM)?;
    assert!(matches!(
        admitted.identity(),
        SegmentRecordIdentity::Chunk(_)
    ));

    let prepared = AdmittedSegmentRecord::for_chunk(&[0])?;
    assert_eq!(
        prepared.header().encode(),
        record.get(..112).ok_or("chunk record header is missing")?
    );
    assert_eq!(prepared.payload(), &[0]);
    assert_eq!(
        prepared.checksum().as_bytes(),
        record.get(113..145).ok_or("chunk checksum is missing")?
    );
    Ok(())
}

#[test]
fn canonical_layout_record_matches_frozen_checksum_and_admission() -> Result<(), Box<dyn Error>> {
    let record = record_bytes(ONE_ZERO_BUNDLE_SEGMENT_HEX, 209, 364)?;
    let checksummed = ChecksummedSegmentRecord::decode(&record)?;
    let policy = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);
    let layout = AdmittedLayout::decode_record(checksummed.payload(), policy)?;
    let canonical = layout.encode_record()?;
    let admitted = checksummed.admit(LayoutEntryLimit::MAXIMUM)?;

    assert!(matches!(
        admitted.identity(),
        SegmentRecordIdentity::Layout(_)
    ));
    let prepared = AdmittedSegmentRecord::for_layout(&canonical)?;
    assert_eq!(
        prepared.header().encode(),
        record.get(..112).ok_or("layout record header is missing")?
    );
    assert_eq!(prepared.payload(), canonical.bytes());
    assert_eq!(
        prepared.checksum().as_bytes(),
        record.get(332..364).ok_or("layout checksum is missing")?
    );
    Ok(())
}

fn record_bytes(hex: &str, offset: usize, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let canonical = hex
        .strip_suffix('\n')
        .ok_or("segment fixture must end in one LF")?;
    let segment = decode_hex(canonical)?;
    let end = offset
        .checked_add(length)
        .ok_or("record fixture offset overflow")?;
    segment
        .get(offset..end)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| "segment fixture lacks the requested record".into())
}

fn mutate(canonical: &[u8], offset: usize, replacement: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let end = offset
        .checked_add(replacement.len())
        .ok_or("record mutation end overflow")?;
    let mut mutated = canonical.to_vec();
    let target = mutated
        .get_mut(offset..end)
        .ok_or("record mutation is out of bounds")?;
    target.copy_from_slice(replacement);
    Ok(mutated)
}

fn recompute_checksum(record: &mut [u8], checksum_offset: usize) -> Result<(), Box<dyn Error>> {
    let covered = record
        .get(..checksum_offset)
        .ok_or("record checksum coverage is out of bounds")?;
    let covered_length = u64::try_from(covered.len())?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"KEEP:SEG:RECORD:SUM\0");
    hasher.update(&1u16.to_be_bytes());
    hasher.update(&[1]);
    hasher.update(covered);
    hasher.update(&covered_length.to_be_bytes());
    let checksum = *hasher.finalize().as_bytes();
    let target = record
        .get_mut(checksum_offset..)
        .ok_or("record checksum destination is out of bounds")?;
    if target.len() != checksum.len() {
        return Err("record checksum destination has the wrong width".into());
    }
    target.copy_from_slice(&checksum);
    Ok(())
}
