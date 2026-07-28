//! Public staged-to-sealed immutable-segment writer laws.

#[path = "segment_writer/durability_laws.rs"]
mod durability_laws;
#[path = "segment_writer/refusal_laws.rs"]
mod refusal_laws;
#[path = "segment_writer/stage_double.rs"]
pub mod stage_double;
mod support;
#[path = "segment_writer/write_contract_laws.rs"]
mod write_contract_laws;

use std::cell::RefCell;
use std::error::Error;
use std::io::{self, Write};
use std::rc::Rc;

use keep::{AdmittedSegmentRecord, SegmentRecordLimit, SegmentStage, StagedSegment};
use support::decode_hex;

const EMPTY_SEGMENT_HEX: &str = include_str!("../conformance/segment-store/v1/empty-segment.hex");
const ONE_ZERO_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-segment.hex");

#[test]
fn explicit_empty_seal_matches_the_frozen_segment() -> Result<(), Box<dyn Error>> {
    let observation = Rc::new(RefCell::new(StageObservation::default()));
    let stage = RecordingStage::new(Rc::clone(&observation));
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let sealed = staged.seal()?;
    let canonical = decode_hex(
        EMPTY_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;

    assert_eq!(observation.borrow().bytes, canonical);
    assert_eq!(sealed.record_count(), 0);
    assert_eq!(sealed.segment_length(), 192);
    Ok(())
}

#[test]
fn explicit_sealing_matches_the_frozen_segment_and_durability_order() -> Result<(), Box<dyn Error>>
{
    let observation = Rc::new(RefCell::new(StageObservation::default()));
    let stage = RecordingStage::new(Rc::clone(&observation));
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let staged = staged.append(AdmittedSegmentRecord::for_chunk(&[0])?)?;
    let sealed = staged.seal()?;
    let canonical = decode_hex(
        ONE_ZERO_SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("segment fixture must end in one LF")?,
    )?;
    let observed = observation.borrow();

    assert_eq!(observed.bytes, canonical);
    assert_eq!(
        observed.events,
        [
            StageEvent::Write,
            StageEvent::Write,
            StageEvent::Write,
            StageEvent::Write,
            StageEvent::Flush,
            StageEvent::Synchronize,
            StageEvent::Write,
            StageEvent::Flush,
            StageEvent::Synchronize,
        ]
    );
    assert_eq!(sealed.record_count(), 1);
    assert_eq!(sealed.segment_length(), 337);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StageEvent {
    Write,
    Flush,
    Synchronize,
}

#[derive(Default)]
struct StageObservation {
    bytes: Vec<u8>,
    events: Vec<StageEvent>,
}

struct RecordingStage {
    observation: Rc<RefCell<StageObservation>>,
}

impl RecordingStage {
    const fn new(observation: Rc<RefCell<StageObservation>>) -> Self {
        Self { observation }
    }
}

impl Write for RecordingStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut observation = self.observation.borrow_mut();
        observation.events.push(StageEvent::Write);
        observation.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.observation.borrow_mut().events.push(StageEvent::Flush);
        Ok(())
    }
}

impl SegmentStage for RecordingStage {
    fn synchronize(&mut self) -> io::Result<()> {
        self.observation
            .borrow_mut()
            .events
            .push(StageEvent::Synchronize);
        Ok(())
    }
}
