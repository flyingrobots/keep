//! Small port-level laws: observation cannot conceal real stage durability failure.
//! Oracle: `SegmentStage`'s fallible flush/synchronize contract, not physical disk evidence.

use std::error::Error;
use std::io::{self, Write};

use keep::{
    ObservedSegmentStage, SegmentDurabilityPhase, SegmentRecordLimit, SegmentStage,
    SegmentWriteError, StagedSegment,
};

use super::Observer;

#[derive(Clone, Copy)]
enum Failure {
    Flush,
    Synchronize,
}

struct FailingStage(Failure);

impl Write for FailingStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        match self.0 {
            Failure::Flush => Err(io::Error::from_raw_os_error(5)),
            Failure::Synchronize => Ok(()),
        }
    }
}

impl SegmentStage for FailingStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Err(io::Error::from_raw_os_error(5))
    }
}

#[test]
fn observation_preserves_the_underlying_flush_refusal() -> Result<(), Box<dyn Error>> {
    let stage = ObservedSegmentStage::new(FailingStage(Failure::Flush), Observer::Prefixes);
    let Err(error) = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?.seal() else {
        return Err("observation manufactured sealing despite failed flush".into());
    };
    assert!(
        matches!(error, SegmentWriteError::Flush { phase: SegmentDurabilityPhase::RecordPrefix, ref source } if source.raw_os_error() == Some(5))
    );
    Ok(())
}

#[test]
fn observation_preserves_the_underlying_synchronization_refusal() -> Result<(), Box<dyn Error>> {
    let stage = ObservedSegmentStage::new(FailingStage(Failure::Synchronize), Observer::Prefixes);
    let Err(error) = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?.seal() else {
        return Err("observation manufactured sealing despite failed synchronization".into());
    };
    assert!(
        matches!(error, SegmentWriteError::Synchronize { phase: SegmentDurabilityPhase::RecordPrefix, ref source } if source.raw_os_error() == Some(5))
    );
    Ok(())
}
