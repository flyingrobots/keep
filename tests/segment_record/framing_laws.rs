//! Complete record framing and checksum corruption laws.

use std::error::Error;

use keep::{ChecksummedSegmentRecord, SegmentRecordDecodeError, SegmentRecordHeaderError};

use super::{ONE_ZERO_SEGMENT_HEX, mutate, record_bytes};

#[test]
fn complete_record_refuses_truncated_header_before_header_admission() -> Result<(), Box<dyn Error>>
{
    let record = canonical_chunk_record()?;
    let truncated = record
        .get(..111)
        .ok_or("chunk record lacks a truncated header witness")?;

    assert_eq!(
        ChecksummedSegmentRecord::decode(truncated),
        Err(SegmentRecordDecodeError::TruncatedHeader {
            expected: 112,
            observed: 111,
        })
    );
    Ok(())
}

#[test]
fn complete_record_preserves_malformed_header_as_its_source() -> Result<(), Box<dyn Error>> {
    let record = canonical_chunk_record()?;
    let expected_magic = *b"KEEP:SEG:RECORD\0";
    let mut observed_magic = expected_magic;
    let first = observed_magic
        .first_mut()
        .ok_or("record magic must not be empty")?;
    *first = 0;
    let mutated = mutate(&record, 0, &observed_magic)?;

    let error = ChecksummedSegmentRecord::decode(&mutated)
        .err()
        .ok_or("malformed record header was unexpectedly admitted")?;
    assert_eq!(
        error,
        SegmentRecordDecodeError::Header {
            source: SegmentRecordHeaderError::InvalidMagic {
                expected: expected_magic,
                observed: observed_magic,
            },
        }
    );
    assert!(error.source().is_some());
    Ok(())
}

#[test]
fn complete_record_refuses_truncation_and_trailing_data_exactly() -> Result<(), Box<dyn Error>> {
    let record = canonical_chunk_record()?;
    let truncated = record
        .get(..144)
        .ok_or("chunk record lacks a truncated record witness")?;
    let mut trailing = record.clone();
    trailing.push(0);

    assert_eq!(
        ChecksummedSegmentRecord::decode(truncated),
        Err(SegmentRecordDecodeError::TruncatedRecord {
            expected: 145,
            observed: 144,
        })
    );
    assert_eq!(
        ChecksummedSegmentRecord::decode(&trailing),
        Err(SegmentRecordDecodeError::TrailingData {
            expected: 145,
            observed: 146,
        })
    );
    Ok(())
}

#[test]
fn complete_record_refuses_checksum_disagreement_with_both_coordinates()
-> Result<(), Box<dyn Error>> {
    let record = canonical_chunk_record()?;
    let canonical = ChecksummedSegmentRecord::decode(&record)?;
    let mut corrupted = record.clone();
    let checksum = corrupted
        .get_mut(113..145)
        .ok_or("chunk record checksum is missing")?;
    let first = checksum
        .first_mut()
        .ok_or("chunk record checksum must not be empty")?;
    *first ^= 1;

    let error = ChecksummedSegmentRecord::decode(&corrupted)
        .err()
        .ok_or("corrupted checksum was unexpectedly admitted")?;
    let SegmentRecordDecodeError::ChecksumMismatch { expected, observed } = error else {
        return Err("checksum corruption reached the wrong refusal".into());
    };
    assert_eq!(expected, canonical.checksum());
    assert_eq!(
        observed.as_bytes(),
        corrupted
            .get(113..145)
            .ok_or("corrupted checksum is missing")?
    );
    Ok(())
}

fn canonical_chunk_record() -> Result<Vec<u8>, Box<dyn Error>> {
    record_bytes(ONE_ZERO_SEGMENT_HEX, 64, 145)
}
