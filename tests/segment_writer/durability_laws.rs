//! Exact staged-segment sealing and durability refusal laws.

use std::error::Error;
use std::io::ErrorKind;

use keep::{
    AdmittedSegmentRecord, SegmentDurabilityPhase, SegmentHeader, SegmentRecordLimit, SegmentSeal,
    SegmentWriteError, SegmentWritePhase, StagedSegment,
};

use super::stage_double::{ScriptedStage, WriteAction};
use super::support::decode_hex;
use super::{EMPTY_SEGMENT_HEX, ONE_ZERO_SEGMENT_HEX};

#[test]
fn seal_write_failure_preserves_the_complete_prefix_offset() -> Result<(), Box<dyn Error>> {
    let actions = [
        WriteAction::Full,
        WriteAction::Full,
        WriteAction::Full,
        WriteAction::Full,
        WriteAction::Error(ErrorKind::StorageFull),
    ];
    let (stage, probe) = ScriptedStage::new(&actions, None, None);
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let staged = staged.append(AdmittedSegmentRecord::for_chunk(&[0])?)?;
    let error = seal_refusal(staged)?;
    let canonical = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let prefix_length = canonical
        .len()
        .checked_sub(SegmentSeal::ENCODED_LENGTH)
        .ok_or("segment fixture lacks its seal")?;

    assert_write_error(error, SegmentWritePhase::Seal, 209, ErrorKind::StorageFull)?;
    assert_eq!(
        probe.bytes(),
        canonical
            .get(..prefix_length)
            .ok_or("segment fixture lacks its record prefix")?
    );
    Ok(())
}

#[test]
fn every_flush_and_sync_boundary_has_an_exact_refusal() -> Result<(), Box<dyn Error>> {
    assert_durability_refusal(
        Some(1),
        None,
        SegmentDurabilityPhase::RecordPrefix,
        DurabilityFailure::Flush,
    )?;
    assert_durability_refusal(
        None,
        Some(1),
        SegmentDurabilityPhase::RecordPrefix,
        DurabilityFailure::Synchronize,
    )?;
    assert_durability_refusal(
        Some(2),
        None,
        SegmentDurabilityPhase::SealedSegment,
        DurabilityFailure::Flush,
    )?;
    assert_durability_refusal(
        None,
        Some(2),
        SegmentDurabilityPhase::SealedSegment,
        DurabilityFailure::Synchronize,
    )
}

fn seal_refusal(staged: StagedSegment<ScriptedStage>) -> Result<SegmentWriteError, Box<dyn Error>> {
    match staged.seal() {
        Ok(_sealed) => Err("malformed seal transition was admitted".into()),
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

fn assert_durability_refusal(
    flush_failure: Option<u32>,
    sync_failure: Option<u32>,
    phase: SegmentDurabilityPhase,
    failure: DurabilityFailure,
) -> Result<(), Box<dyn Error>> {
    let (stage, probe) = ScriptedStage::new(&[], flush_failure, sync_failure);
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let error = seal_refusal(staged)?;
    let observed = match error {
        SegmentWriteError::Flush { phase, .. } if failure == DurabilityFailure::Flush => phase,
        SegmentWriteError::Synchronize { phase, .. }
            if failure == DurabilityFailure::Synchronize =>
        {
            phase
        }
        other => return Err(format!("unexpected durability refusal: {other}").into()),
    };
    assert_eq!(observed, phase);
    let canonical = decode_hex(
        EMPTY_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let retained_length = match phase {
        SegmentDurabilityPhase::RecordPrefix => SegmentHeader::ENCODED_LENGTH,
        SegmentDurabilityPhase::SealedSegment => canonical.len(),
    };
    assert_eq!(
        probe.bytes(),
        canonical
            .get(..retained_length)
            .ok_or("segment fixture lacks the retained durability prefix")?
    );
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DurabilityFailure {
    Flush,
    Synchronize,
}
