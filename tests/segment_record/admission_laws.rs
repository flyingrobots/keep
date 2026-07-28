//! Complete record content-identity and resource admission laws.

use std::error::Error;

use keep::{
    AdmittedSegmentRecord, ChecksummedSegmentRecord, ChunkHashError, ChunkId, LayoutDecodeError,
    LayoutEntryLimit, SegmentRecordAdmissionError,
};

use super::{
    ONE_ZERO_BUNDLE_SEGMENT_HEX, ONE_ZERO_SEGMENT_HEX, mutate, recompute_checksum, record_bytes,
};

#[test]
fn empty_chunk_preparation_preserves_the_chunk_hash_failure() -> Result<(), Box<dyn Error>> {
    let error = AdmittedSegmentRecord::for_chunk(&[])
        .err()
        .ok_or("empty chunk record was unexpectedly prepared")?;
    assert!(matches!(
        &error,
        SegmentRecordAdmissionError::ChunkHash {
            source: ChunkHashError::Empty,
        }
    ));
    assert!(error.source().is_some());
    Ok(())
}

#[test]
fn checksummed_chunk_still_refuses_a_different_payload_identity() -> Result<(), Box<dyn Error>> {
    let record = record_bytes(ONE_ZERO_SEGMENT_HEX, 64, 145)?;
    let mut mutated = mutate(&record, 112, &[1])?;
    recompute_checksum(&mut mutated, 113)?;
    let checksummed = ChecksummedSegmentRecord::decode(&mutated)?;

    let error = checksummed
        .admit(LayoutEntryLimit::MAXIMUM)
        .err()
        .ok_or("mismatched chunk identity was unexpectedly admitted")?;
    let SegmentRecordAdmissionError::ChunkIdentityMismatch { expected, observed } = error else {
        return Err("chunk identity mismatch reached the wrong refusal".into());
    };
    assert_eq!(expected, ChunkId::hash_bytes(&[0])?);
    assert_eq!(observed, ChunkId::hash_bytes(&[1])?);
    Ok(())
}

#[test]
fn checksummed_layout_preserves_its_nested_structural_failure() -> Result<(), Box<dyn Error>> {
    let record = record_bytes(ONE_ZERO_BUNDLE_SEGMENT_HEX, 209, 364)?;
    let expected_magic = *b"KEEP:LAYOUT:V1\0\0";
    let mut observed_magic = expected_magic;
    let first = observed_magic
        .first_mut()
        .ok_or("layout magic must not be empty")?;
    *first = 0;
    let mut mutated = mutate(&record, 112, &observed_magic)?;
    recompute_checksum(&mut mutated, 332)?;
    let checksummed = ChecksummedSegmentRecord::decode(&mutated)?;

    let error = checksummed
        .admit(LayoutEntryLimit::MAXIMUM)
        .err()
        .ok_or("malformed layout payload was unexpectedly admitted")?;
    let SegmentRecordAdmissionError::Layout { source } = &error else {
        return Err("malformed layout reached the wrong refusal".into());
    };
    assert!(matches!(
        source,
        LayoutDecodeError::InvalidMagic {
            observed
        } if observed == &observed_magic
    ));
    assert!(error.source().is_some());
    Ok(())
}

#[test]
fn layout_record_admission_obeys_the_caller_resource_cap() -> Result<(), Box<dyn Error>> {
    let record = record_bytes(ONE_ZERO_BUNDLE_SEGMENT_HEX, 209, 364)?;
    let checksummed = ChecksummedSegmentRecord::decode(&record)?;
    let zero_entries = LayoutEntryLimit::new(0)?;

    let error = checksummed
        .admit(zero_entries)
        .err()
        .ok_or("one-entry layout ignored the zero-entry cap")?;
    let SegmentRecordAdmissionError::Layout { source } = error else {
        return Err("layout resource refusal reached the wrong boundary".into());
    };
    assert!(matches!(
        source,
        LayoutDecodeError::ConfiguredEntryLimitExceeded {
            maximum: 0,
            observed: 1,
        }
    ));
    Ok(())
}
