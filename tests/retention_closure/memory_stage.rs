//! In-memory segment-stage support for closure integration laws.
#![allow(
    clippy::redundant_pub_crate,
    reason = "private integration-test siblings share this segment fixture"
)]

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use keep::{
    AdmittedSegmentRecord, SegmentRecordLimit, SegmentStage, SegmentWriteError, StagedSegment,
};

pub(super) fn segment_bytes(
    records: &[AdmittedSegmentRecord<'_>],
) -> Result<Vec<u8>, SegmentWriteError> {
    let bytes = Rc::new(RefCell::new(Vec::new()));
    let stage = MemoryStage {
        bytes: Rc::clone(&bytes),
    };
    let mut staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    for record in records {
        staged = staged.append(*record)?;
    }
    let _sealed = staged.seal()?;
    Ok(bytes.borrow().clone())
}

struct MemoryStage {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl Write for MemoryStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for MemoryStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
