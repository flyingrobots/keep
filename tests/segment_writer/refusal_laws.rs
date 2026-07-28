//! Exact staged-segment write and durability refusal laws.

use std::error::Error;
use std::io::ErrorKind;

use keep::{
    AdmittedSegmentRecord, SegmentHeader, SegmentRecordLimit, SegmentSeal, SegmentWriteError,
    SegmentWritePhase, StagedSegment,
};

use super::ONE_ZERO_SEGMENT_HEX;
use super::stage_double::{ScriptedStage, WriteAction};
use super::support::decode_hex;

#[test]
fn every_record_write_boundary_preserves_phase_offset_and_source() -> Result<(), Box<dyn Error>> {
    let canonical = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let cases = [
        (
            &[
                WriteAction::Full,
                WriteAction::Error(ErrorKind::StorageFull),
            ][..],
            SegmentWritePhase::RecordHeader,
            64,
            ErrorKind::StorageFull,
        ),
        (
            &[
                WriteAction::Full,
                WriteAction::Full,
                WriteAction::Error(ErrorKind::PermissionDenied),
            ][..],
            SegmentWritePhase::RecordPayload,
            176,
            ErrorKind::PermissionDenied,
        ),
        (
            &[
                WriteAction::Full,
                WriteAction::Full,
                WriteAction::Full,
                WriteAction::Error(ErrorKind::StorageFull),
            ][..],
            SegmentWritePhase::RecordChecksum,
            177,
            ErrorKind::StorageFull,
        ),
    ];
    for (actions, phase, bytes_written, kind) in cases {
        let (stage, probe) = ScriptedStage::new(actions, None, None);
        let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
        let error = append_refusal(staged)?;
        assert_write_error(error, phase, bytes_written, kind)?;
        let retained = usize::try_from(bytes_written)?;
        assert_eq!(
            probe.bytes(),
            canonical
                .get(..retained)
                .ok_or("segment fixture lacks the retained prefix")?
        );
    }
    Ok(())
}

#[test]
fn duplicate_and_count_refusals_write_no_record_prefix_bytes() -> Result<(), Box<dyn Error>> {
    let canonical = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let record_prefix_length = canonical
        .len()
        .checked_sub(SegmentSeal::ENCODED_LENGTH)
        .ok_or("segment fixture lacks its seal")?;
    let record = AdmittedSegmentRecord::for_chunk(&[0])?;
    let (stage, duplicate_probe) = ScriptedStage::new(&[], None, None);
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let staged = staged.append(record)?;
    let duplicate = match staged.append(record) {
        Ok(_staged) => return Err("duplicate record was appended".into()),
        Err(error) => error,
    };
    assert!(matches!(
        duplicate,
        SegmentWriteError::DuplicateRecordIdentity { .. }
    ));
    assert_eq!(
        duplicate_probe.bytes(),
        canonical
            .get(..record_prefix_length)
            .ok_or("segment fixture lacks its record prefix")?
    );

    let zero = SegmentRecordLimit::new(0)?;
    let (stage, limit_probe) = ScriptedStage::new(&[], None, None);
    let staged = StagedSegment::begin(stage, zero)?;
    let limited = match staged.append(record) {
        Ok(_staged) => return Err("record above configured limit was appended".into()),
        Err(error) => error,
    };
    assert!(matches!(
        limited,
        SegmentWriteError::RecordCountLimit {
            maximum: 0,
            observed: 1,
        }
    ));
    assert_eq!(
        limit_probe.bytes(),
        canonical
            .get(..SegmentHeader::ENCODED_LENGTH)
            .ok_or("segment fixture lacks its header")?
    );
    Ok(())
}

fn append_refusal(
    staged: StagedSegment<ScriptedStage>,
) -> Result<SegmentWriteError, Box<dyn Error>> {
    match staged.append(AdmittedSegmentRecord::for_chunk(&[0])?) {
        Ok(_staged) => Err("malformed record write was admitted".into()),
        Err(error) => Ok(error),
    }
}

fn assert_write_error(
    error: SegmentWriteError,
    phase: SegmentWritePhase,
    bytes_written: u64,
    kind: ErrorKind,
) -> Result<(), Box<dyn Error>> {
    let SegmentWriteError::Write {
        phase: observed_phase,
        bytes_written: observed_bytes,
        source,
    } = error
    else {
        return Err(format!("unexpected stage write refusal: {error}").into());
    };
    assert_eq!(observed_phase, phase);
    assert_eq!(observed_bytes, bytes_written);
    assert_eq!(source.kind(), kind);
    Ok(())
}
