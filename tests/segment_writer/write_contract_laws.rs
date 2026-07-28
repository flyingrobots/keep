//! Exact low-level staged-segment write contract laws.

use std::error::Error;
use std::io::ErrorKind;

use keep::{
    AdmittedSegmentRecord, SegmentRecordLimit, SegmentWriteError, SegmentWritePhase, StagedSegment,
};

use super::ONE_ZERO_SEGMENT_HEX;
use super::stage_double::{ScriptedStage, WriteAction};
use super::support::decode_hex;

#[test]
fn short_and_interrupted_writes_still_produce_the_frozen_segment() -> Result<(), Box<dyn Error>> {
    let (stage, probe) = ScriptedStage::new(
        &[WriteAction::Interrupted, WriteAction::Limit(1)],
        None,
        None,
    );
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let staged = staged.append(AdmittedSegmentRecord::for_chunk(&[0])?)?;
    let _sealed = staged.seal()?;
    let canonical = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;

    assert_eq!(probe.bytes(), canonical);
    Ok(())
}

#[test]
fn partial_header_failure_preserves_phase_offset_and_source() -> Result<(), Box<dyn Error>> {
    let actions = [
        WriteAction::Limit(1),
        WriteAction::Error(ErrorKind::PermissionDenied),
    ];
    let (stage, _probe) = ScriptedStage::new(&actions, None, None);
    let error = begin_refusal(stage)?;
    let SegmentWriteError::Write {
        phase,
        bytes_written,
        source,
    } = error
    else {
        return Err(format!("unexpected header write refusal: {error}").into());
    };

    assert_eq!(phase, SegmentWritePhase::Header);
    assert_eq!(bytes_written, 1);
    assert_eq!(source.kind(), ErrorKind::PermissionDenied);
    Ok(())
}

#[test]
fn zero_progress_header_write_has_an_exact_refusal() -> Result<(), Box<dyn Error>> {
    let (stage, _probe) = ScriptedStage::new(&[WriteAction::Zero], None, None);
    let error = begin_refusal(stage)?;

    assert!(matches!(
        error,
        SegmentWriteError::WriteZero {
            phase: SegmentWritePhase::Header,
            bytes_written: 0,
        }
    ));
    Ok(())
}

#[test]
fn overreported_header_write_count_has_an_exact_refusal() -> Result<(), Box<dyn Error>> {
    let (stage, _probe) = ScriptedStage::new(&[WriteAction::Overreport(1)], None, None);
    let error = begin_refusal(stage)?;

    assert!(matches!(
        error,
        SegmentWriteError::InvalidWriteCount {
            phase: SegmentWritePhase::Header,
            maximum: 64,
            observed: 65,
            bytes_written: 0,
        }
    ));
    Ok(())
}

fn begin_refusal(stage: ScriptedStage) -> Result<SegmentWriteError, Box<dyn Error>> {
    match StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM) {
        Ok(_staged) => Err("malformed header write was admitted".into()),
        Err(error) => Ok(error),
    }
}
