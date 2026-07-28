//! Exclusive filesystem segment-stage creation laws.

#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;

use std::error::Error;
use std::fs;
use std::io::ErrorKind;

use keep::{
    AdmittedSegmentRecord, FilesystemSegmentStage, SegmentHeader, SegmentRecordLimit,
    SegmentStageCreateError, StagedSegment,
};
use sandbox::TestDirectory;
use support::decode_hex;

const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");

#[test]
fn exclusive_creation_never_truncates_existing_stage() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exclusive-create-refusal")?;
    let staging = sandbox.path().join("staging");
    fs::create_dir(&staging)?;
    let stage_path = staging.join("current.seg");
    fs::write(&stage_path, b"preserved evidence")?;

    let error = match FilesystemSegmentStage::create(&staging) {
        Ok(_stage) => return Err("existing stage was replaced".into()),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SegmentStageCreateError::Create { ref source }
            if source.kind() == ErrorKind::AlreadyExists
    ));
    assert_eq!(fs::read(stage_path)?, b"preserved evidence");
    sandbox.remove()?;
    Ok(())
}

#[test]
fn exclusive_stage_starts_at_zero_and_retains_exact_sealed_bytes() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exclusive-create-success")?;
    let staging = sandbox.path().join("staging");
    fs::create_dir(&staging)?;
    let stage = FilesystemSegmentStage::create(&staging)?;
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let staged = staged.append(AdmittedSegmentRecord::for_chunk(&[0])?)?;
    let sealed = staged.seal()?;
    drop(sealed);
    let canonical = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;

    assert_eq!(fs::read(staging.join("current.seg"))?, canonical);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn dropping_an_unsealed_stage_preserves_the_reusable_prefix() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("unsealed-prefix-preservation")?;
    let staging = sandbox.path().join("staging");
    fs::create_dir(&staging)?;
    let stage = FilesystemSegmentStage::create(&staging)?;
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    drop(staged);
    let empty_segment = decode_hex(
        EMPTY_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let header = empty_segment
        .get(..SegmentHeader::ENCODED_LENGTH)
        .ok_or("empty segment fixture lacks its header")?;

    assert_eq!(fs::read(staging.join("current.seg"))?, header);
    sandbox.remove()?;
    Ok(())
}
