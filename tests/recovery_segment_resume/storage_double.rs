//! This module owns the in-memory reusable-stage storage double.

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use keep::{
    OpenedReusableSegment, RecoverySegmentResumeRequest, RecoverySegmentResumeStorage,
    RecoverySegmentResumeStorageError, SegmentStage,
};

/// In-memory storage that either returns one exact prefix or an injected error.
pub struct MemoryResumeStorage {
    encoded: Box<[u8]>,
    probe: Rc<RefCell<Vec<u8>>>,
    failure: Option<io::Error>,
}

impl MemoryResumeStorage {
    /// Constructs available storage containing `encoded`.
    pub fn available(encoded: &[u8]) -> Self {
        Self {
            encoded: encoded.into(),
            probe: Rc::new(RefCell::new(encoded.to_vec())),
            failure: None,
        }
    }

    /// Constructs storage that refuses reopening before returning a stage.
    pub fn failing(encoded: &[u8]) -> Self {
        Self {
            encoded: encoded.into(),
            probe: Rc::new(RefCell::new(encoded.to_vec())),
            failure: Some(io::Error::other("injected resume failure")),
        }
    }

    /// Returns shared observation of all stage bytes.
    pub fn probe(&self) -> Rc<RefCell<Vec<u8>>> {
        Rc::clone(&self.probe)
    }
}

/// Append-only in-memory stage positioned after its preloaded prefix.
pub struct MemoryResumeStage {
    probe: Rc<RefCell<Vec<u8>>>,
}

impl Write for MemoryResumeStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.probe.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for MemoryResumeStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl RecoverySegmentResumeStorage for MemoryResumeStorage {
    type Stage = MemoryResumeStage;

    fn open_reusable(
        self,
        _request: RecoverySegmentResumeRequest,
    ) -> Result<OpenedReusableSegment<Self::Stage>, RecoverySegmentResumeStorageError> {
        if let Some(source) = self.failure {
            return Err(RecoverySegmentResumeStorageError::storage(source));
        }
        Ok(OpenedReusableSegment::new(
            MemoryResumeStage { probe: self.probe },
            self.encoded,
        ))
    }
}
