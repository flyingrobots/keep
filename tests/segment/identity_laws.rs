//! Complete-segment nested framing, checksum, and identity laws.

use std::error::Error;

use keep::{
    AdmittedSegment, SegmentHeaderError, SegmentReadError, SegmentRecordAdmissionError,
    SegmentRecordDecodeError, SegmentRecordHeaderError,
};

use super::format_oracle::seal_segment;
use super::record_checksum_oracle::rewrite_record_checksum;
use super::{maximum_policy, one_record_bytes, one_record_prefix};

#[test]
fn duplicate_logical_identity_reports_both_physical_coordinates() -> Result<(), Box<dyn Error>> {
    let prefix = one_record_prefix()?;
    let record = one_record_bytes()?;
    let mut duplicate_prefix = prefix;
    duplicate_prefix.extend_from_slice(&record);
    let encoded = seal_segment(&duplicate_prefix, 2)?;
    let error = refusal(&encoded)?;

    let SegmentReadError::DuplicateRecordIdentity {
        first_index,
        duplicate_index,
        first_offset,
        duplicate_offset,
        ..
    } = error
    else {
        return Err(format!("unexpected duplicate refusal: {error}").into());
    };
    assert_eq!((first_index, first_offset), (0, 64));
    assert_eq!((duplicate_index, duplicate_offset), (1, 209));
    Ok(())
}

#[test]
fn outer_digest_cannot_hide_a_record_checksum_mismatch() -> Result<(), Box<dyn Error>> {
    let mut prefix = one_record_prefix()?;
    let checksum_byte = prefix
        .get_mut(177)
        .ok_or("test record lacks its checksum")?;
    *checksum_byte ^= u8::MAX;
    let encoded = seal_segment(&prefix, 1)?;
    let error = refusal(&encoded)?;

    let SegmentReadError::RecordDecode {
        record_index,
        offset,
        source: SegmentRecordDecodeError::ChecksumMismatch { .. },
    } = error
    else {
        return Err(format!("unexpected record-checksum refusal: {error}").into());
    };
    assert_eq!((record_index, offset), (0, 64));
    Ok(())
}

#[test]
fn ordinary_record_corruption_is_localized_before_the_segment_digest() -> Result<(), Box<dyn Error>>
{
    let mut encoded = seal_segment(&one_record_prefix()?, 1)?;
    let checksum_byte = encoded
        .get_mut(177)
        .ok_or("test record lacks its checksum")?;
    *checksum_byte ^= u8::MAX;
    let error = refusal(&encoded)?;

    let SegmentReadError::RecordDecode {
        record_index,
        offset,
        source: SegmentRecordDecodeError::ChecksumMismatch { .. },
    } = error
    else {
        return Err(format!("record corruption was not localized: {error}").into());
    };
    assert_eq!((record_index, offset), (0, 64));
    Ok(())
}

#[test]
fn checksummed_payload_still_requires_its_declared_logical_identity() -> Result<(), Box<dyn Error>>
{
    let mut prefix = one_record_prefix()?;
    let payload = prefix.get_mut(176).ok_or("test record lacks its payload")?;
    *payload = 1;
    let record = prefix
        .get_mut(64..209)
        .ok_or("test segment lacks its complete record")?;
    rewrite_record_checksum(record)?;
    let encoded = seal_segment(&prefix, 1)?;
    let error = refusal(&encoded)?;

    let SegmentReadError::RecordAdmission {
        record_index,
        offset,
        source: SegmentRecordAdmissionError::ChunkIdentityMismatch { expected, observed },
    } = error
    else {
        return Err(format!("unexpected content-identity refusal: {error}").into());
    };
    assert_eq!((record_index, offset), (0, 64));
    assert_ne!(expected, observed);
    Ok(())
}

#[test]
fn record_header_refusal_precedes_record_checksum_admission() -> Result<(), Box<dyn Error>> {
    let mut prefix = one_record_prefix()?;
    let version = prefix
        .get_mut(80..82)
        .ok_or("test record lacks its version")?;
    version.copy_from_slice(&2_u16.to_be_bytes());
    let encoded = seal_segment(&prefix, 1)?;
    let error = refusal(&encoded)?;

    let SegmentReadError::RecordHeader {
        record_index,
        offset,
        source: SegmentRecordHeaderError::UnsupportedVersion { expected, observed },
    } = error
    else {
        return Err(format!("unexpected record-header refusal: {error}").into());
    };
    assert_eq!((record_index, offset), (0, 64));
    assert_eq!((expected, observed), (1, 2));
    Ok(())
}

#[test]
fn segment_header_refusal_precedes_outer_digest_admission() -> Result<(), Box<dyn Error>> {
    let mut prefix = one_record_prefix()?;
    let magic = prefix
        .first_mut()
        .ok_or("test segment lacks its header magic")?;
    *magic = 0;
    let encoded = seal_segment(&prefix, 1)?;
    let error = refusal(&encoded)?;

    let SegmentReadError::Header {
        source: SegmentHeaderError::InvalidMagic { .. },
    } = error
    else {
        return Err(format!("unexpected segment-header refusal: {error}").into());
    };
    Ok(())
}

fn refusal(encoded: &[u8]) -> Result<SegmentReadError, Box<dyn Error>> {
    match AdmittedSegment::decode(encoded, maximum_policy()) {
        Ok(_admitted) => Err("malformed complete segment was admitted".into()),
        Err(error) => Ok(error),
    }
}
