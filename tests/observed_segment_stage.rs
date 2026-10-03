//! Medium public filesystem laws for sealed observation; oracle: canonical bytes or typed refusal.
//! Delete only when the observation capability is removed or stronger boundary laws replace these.

#![cfg(all(target_os = "linux", feature = "repository-tasks"))]

#[path = "observed_segment_stage/byte_equivalence.rs"]
mod byte_equivalence;
#[path = "observed_segment_stage/durability_failure.rs"]
mod durability_failure;
#[path = "segment_filesystem_stage/sandbox.rs"]
pub mod sandbox;
mod support;

use std::error::Error;
use std::fs;
use std::io;

use keep::{
    AdmittedSegment, CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemCatalogPublisher,
    FilesystemPlatformAdmission, LayoutEntryLimit, ObservedSegmentStage, SegmentReadPolicy,
    SegmentRecordLimit, SegmentStageDurabilityEvent, SegmentStageObserver, SegmentWriteError,
    SegmentWritePhase, StagedSegment,
};

#[test]
fn removing_observation_preserves_canonical_bytes_and_publisher_authority()
-> Result<(), Box<dyn Error>> {
    let directory = sandbox::TestDirectory::create("sealed-observation")?;
    let publisher = publisher(&directory)?;
    let stage = ObservedSegmentStage::new(publisher.create_segment_stage()?, Observer::Prefixes);
    let sealed = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?
        .seal()?
        .without_observer();
    let canonical = support::decode_hex(
        include_str!("../conformance/segment-store/v1/empty-segment.hex").trim_end(),
    )?;
    assert_eq!(
        fs::read(directory.path().join("staging/current.seg"))?,
        canonical
    );
    let admitted = AdmittedSegment::decode(&canonical, read_policy())?;
    let _selection = publisher.select_segment(sealed, &admitted)?;
    drop(publisher);
    directory.remove()?;
    Ok(())
}

#[test]
fn an_excessive_observation_limit_refuses_before_writing() -> Result<(), Box<dyn Error>> {
    let directory = sandbox::TestDirectory::create("invalid-observation-limit")?;
    let publisher = publisher(&directory)?;
    let stage = ObservedSegmentStage::new(publisher.create_segment_stage()?, Observer::Excessive);
    let Err(error) = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM) else {
        return Err("excessive observation limit was admitted".into());
    };
    assert!(
        matches!(error, SegmentWriteError::Write { phase: SegmentWritePhase::Header, bytes_written: 0, ref source }
        if source.kind() == io::ErrorKind::InvalidInput)
    );
    assert_eq!(fs::read(directory.path().join("staging/current.seg"))?, []);
    drop(publisher);
    directory.remove()?;
    Ok(())
}

#[test]
fn a_post_write_interruption_cannot_retry_already_written_bytes() -> Result<(), Box<dyn Error>> {
    let directory = sandbox::TestDirectory::create("post-write-observation-interruption")?;
    let publisher = publisher(&directory)?;
    let stage = ObservedSegmentStage::new(
        publisher.create_segment_stage()?,
        Observer::InterruptAfterWrite,
    );
    let Err(error) = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM) else {
        return Err("post-effect interruption was retried into successful admission".into());
    };
    let SegmentWriteError::Write {
        phase: SegmentWritePhase::Header,
        bytes_written: 0,
        source,
    } = error
    else {
        return Err(format!("unexpected interruption failure: {error}").into());
    };
    assert_eq!(source.kind(), io::ErrorKind::Other);
    let original = source
        .get_ref()
        .and_then(|error| error.downcast_ref::<io::Error>())
        .ok_or("original interruption cause lost")?;
    assert_eq!(original.kind(), io::ErrorKind::Interrupted);
    let canonical = support::decode_hex(
        include_str!("../conformance/segment-store/v1/empty-segment.hex").trim_end(),
    )?;
    assert_eq!(
        fs::read(directory.path().join("staging/current.seg"))?,
        canonical.get(..64).ok_or("missing golden header")?
    );
    drop(publisher);
    directory.remove()?;
    Ok(())
}

enum Observer {
    Prefixes,
    Excessive,
    InterruptAfterWrite,
}

impl SegmentStageObserver for Observer {
    fn before_write(&mut self, requested: usize) -> io::Result<usize> {
        match self {
            Self::Prefixes => Ok(requested.min(7)),
            Self::Excessive => requested
                .checked_add(1)
                .ok_or_else(|| io::Error::other("limit overflow")),
            Self::InterruptAfterWrite => Ok(requested),
        }
    }

    fn after_write(&mut self, _written: usize) -> io::Result<()> {
        match self {
            Self::InterruptAfterWrite => {
                *self = Self::Prefixes;
                Err(io::Error::from(io::ErrorKind::Interrupted))
            }
            Self::Prefixes | Self::Excessive => Ok(()),
        }
    }

    fn durability(&mut self, _event: SegmentStageDurabilityEvent) -> io::Result<()> {
        Ok(())
    }
}

fn publisher(
    directory: &sandbox::TestDirectory,
) -> Result<FilesystemCatalogPublisher, Box<dyn Error>> {
    let admission = FilesystemPlatformAdmission::initialize(directory.path())?;
    Ok(FilesystemCatalogPublisher::open(
        admission,
        CatalogRestartPolicy::new(read_policy(), CatalogRestartByteLimit::new(1_048_576)?),
    )?)
}

const fn read_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
