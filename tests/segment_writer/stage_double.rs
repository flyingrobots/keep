//! Scriptable segment-stage test double.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{self, ErrorKind, Write};
use std::rc::Rc;

use keep::SegmentStage;

/// One scripted result for the next stage write.
#[derive(Clone, Copy, Debug)]
pub enum WriteAction {
    /// Accept the complete supplied slice.
    Full,
    /// Accept at most the specified byte count.
    Limit(usize),
    /// Return one retryable interruption.
    Interrupted,
    /// Report a zero-byte successful write.
    Zero,
    /// Violate the `Write` contract by over-reporting accepted bytes.
    Overreport(usize),
    /// Return an I/O failure of the specified kind.
    Error(ErrorKind),
}

/// Read-only observation handle for a scripted stage.
#[derive(Clone)]
pub struct StageProbe {
    observation: Rc<RefCell<StageObservation>>,
}

impl StageProbe {
    /// Returns a copy of all successfully written bytes.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.observation.borrow().bytes.clone()
    }

    /// Returns the number of successfully written bytes.
    #[must_use]
    pub fn byte_count(&self) -> usize {
        self.observation.borrow().bytes.len()
    }
}

/// Segment stage with deterministic write, flush, and synchronization faults.
pub struct ScriptedStage {
    observation: Rc<RefCell<StageObservation>>,
    writes: VecDeque<WriteAction>,
    flush_failure: Option<u32>,
    sync_failure: Option<u32>,
}

impl ScriptedStage {
    /// Constructs a scripted stage and its independent observation handle.
    #[must_use]
    pub fn new(
        writes: &[WriteAction],
        flush_failure: Option<u32>,
        sync_failure: Option<u32>,
    ) -> (Self, StageProbe) {
        let observation = Rc::new(RefCell::new(StageObservation::default()));
        let probe = StageProbe {
            observation: Rc::clone(&observation),
        };
        (
            Self {
                observation,
                writes: writes.iter().copied().collect(),
                flush_failure,
                sync_failure,
            },
            probe,
        )
    }
}

#[derive(Default)]
struct StageObservation {
    bytes: Vec<u8>,
    flush_count: u32,
    sync_count: u32,
}

impl Write for ScriptedStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.writes.pop_front().unwrap_or(WriteAction::Full) {
            WriteAction::Full => self.accept(bytes, bytes.len()),
            WriteAction::Limit(limit) => self.accept(bytes, limit.min(bytes.len())),
            WriteAction::Interrupted => Err(io::Error::from(ErrorKind::Interrupted)),
            WriteAction::Zero => Ok(0),
            WriteAction::Overreport(additional) => bytes
                .len()
                .checked_add(additional)
                .ok_or_else(|| io::Error::other("scripted write count overflow")),
            WriteAction::Error(kind) => Err(io::Error::from(kind)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut observation = self.observation.borrow_mut();
        observation.flush_count = observation
            .flush_count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("test flush counter overflow"))?;
        if self.flush_failure == Some(observation.flush_count) {
            return Err(io::Error::other("scripted flush failure"));
        }
        Ok(())
    }
}

impl SegmentStage for ScriptedStage {
    fn synchronize(&mut self) -> io::Result<()> {
        let mut observation = self.observation.borrow_mut();
        observation.sync_count = observation
            .sync_count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("test sync counter overflow"))?;
        if self.sync_failure == Some(observation.sync_count) {
            return Err(io::Error::other("scripted synchronization failure"));
        }
        Ok(())
    }
}

impl ScriptedStage {
    fn accept(&self, bytes: &[u8], count: usize) -> io::Result<usize> {
        let accepted = bytes
            .get(..count)
            .ok_or_else(|| io::Error::other("scripted write limit exceeds input"))?;
        self.observation
            .borrow_mut()
            .bytes
            .extend_from_slice(accepted);
        Ok(count)
    }
}
