//! Exclusive filesystem segment-stage creation laws.

#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;

use std::error::Error;
use std::fs;
use std::io::ErrorKind;

use keep::{
    AdmittedSegmentRecord, CatalogRestartByteLimit, CatalogRestartPolicy,
    FilesystemCatalogPublisher, FilesystemWriterLock, LayoutEntryLimit, SegmentHeader,
    SegmentReadPolicy, SegmentRecordLimit, SegmentStageCreateError, StagedSegment,
};
use sandbox::TestDirectory;
use support::decode_hex;

const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");

#[test]
fn exclusive_creation_never_truncates_existing_stage() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exclusive-create-refusal")?;
    let publisher = open_publisher(&sandbox)?;
    let staging = sandbox.path().join("staging");
    let stage_path = staging.join("current.seg");
    fs::write(&stage_path, b"preserved evidence")?;

    let error = match publisher.create_segment_stage() {
        Ok(_stage) => return Err("existing stage was replaced".into()),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SegmentStageCreateError::Create { ref source }
            if source.kind() == ErrorKind::AlreadyExists
    ));
    assert_eq!(fs::read(stage_path)?, b"preserved evidence");
    drop(publisher);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn repeated_stage_creation_admits_exactly_one_owner() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exclusive-create-repeat")?;
    let publisher = open_publisher(&sandbox)?;
    let staging = sandbox.path().join("staging");
    let first = publisher.create_segment_stage()?;
    let refusal = match publisher.create_segment_stage() {
        Ok(_second) => return Err("both stage contenders were admitted".into()),
        Err(error) => error,
    };

    assert!(matches!(
        refusal,
        SegmentStageCreateError::Create { ref source }
            if source.kind() == ErrorKind::AlreadyExists
    ));
    assert!(staging.join("current.seg").is_file());
    drop(first);
    drop(publisher);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn exclusive_stage_starts_at_zero_and_retains_exact_sealed_bytes() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exclusive-create-success")?;
    let publisher = open_publisher(&sandbox)?;
    let staging = sandbox.path().join("staging");
    let stage = publisher.create_segment_stage()?;
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
    drop(publisher);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn dropping_an_unsealed_stage_preserves_the_reusable_prefix() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("unsealed-prefix-preservation")?;
    let publisher = open_publisher(&sandbox)?;
    let staging = sandbox.path().join("staging");
    let stage = publisher.create_segment_stage()?;
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
    drop(publisher);
    sandbox.remove()?;
    Ok(())
}

fn open_publisher(sandbox: &TestDirectory) -> Result<FilesystemCatalogPublisher, Box<dyn Error>> {
    fs::write(sandbox.path().join("writer.lock"), [])?;
    fs::create_dir(sandbox.path().join("staging"))?;
    fs::create_dir(sandbox.path().join("segments"))?;
    fs::create_dir(sandbox.path().join("catalogs"))?;
    let lock = FilesystemWriterLock::try_acquire(sandbox.path())?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    Ok(FilesystemCatalogPublisher::open(lock, policy)?)
}
