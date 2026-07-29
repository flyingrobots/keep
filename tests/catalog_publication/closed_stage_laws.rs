//! Closed-stage authority laws for immutable-pool publication.

use std::cell::{Cell, RefCell};
use std::error::Error;
use std::io::{self, Write};
use std::rc::Rc;

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, LayoutEntryLimit, SegmentPublication,
    SegmentPublicationError, SegmentReadPolicy, SegmentRecordLimit, SegmentStage, StagedSegment,
};

use super::{SEGMENT_HEX, fixture};
use crate::support::require_error;

#[test]
fn publication_token_exists_only_after_the_writable_stage_is_dropped() -> Result<(), Box<dyn Error>>
{
    let bytes = Rc::new(RefCell::new(Vec::new()));
    let dropped = Rc::new(Cell::new(false));
    let stage = ObservableStage::new(Rc::clone(&bytes), Rc::clone(&dropped));
    let record = AdmittedSegmentRecord::for_chunk(&[0])?;
    let sealed = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?
        .append(record)?
        .seal()?;

    assert!(!dropped.get());
    let closed = sealed.close();
    assert!(dropped.get());
    let encoded = bytes.borrow();
    let admitted = AdmittedSegment::decode(&encoded, policy())?;
    let _selection = SegmentPublication::one(closed, &admitted)?;
    Ok(())
}

#[test]
fn closed_stage_metadata_must_match_the_admitted_bytes() -> Result<(), Box<dyn Error>> {
    let bytes = Rc::new(RefCell::new(Vec::new()));
    let dropped = Rc::new(Cell::new(false));
    let stage = ObservableStage::new(bytes, dropped);
    let closed = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?
        .seal()?
        .close();
    let admitted_bytes = fixture(SEGMENT_HEX)?;
    let admitted = AdmittedSegment::decode(&admitted_bytes, policy())?;
    let error = require_error(
        SegmentPublication::one(closed, &admitted),
        "mismatched closed stage was selected for publication",
    )?;

    assert!(matches!(
        error,
        SegmentPublicationError::RecordCount {
            expected: 0,
            observed: 1,
        }
    ));
    Ok(())
}

struct ObservableStage {
    bytes: Rc<RefCell<Vec<u8>>>,
    dropped: Rc<Cell<bool>>,
}

impl ObservableStage {
    const fn new(bytes: Rc<RefCell<Vec<u8>>>, dropped: Rc<Cell<bool>>) -> Self {
        Self { bytes, dropped }
    }
}

impl Write for ObservableStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for ObservableStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for ObservableStage {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

const fn policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
