//! This module owns transparent stage observation with private writable authority.

use std::io::{self, Write};

use super::{SealedSegment, SegmentStage, SegmentStageDurabilityEvent, SegmentStageObserver};

/// A repository stage decorator whose observer never receives its inner stage.
///
/// Owns the stage until close or sealing. Construction allocates nothing; actual
/// writes and durability operations block according to the supplied stage and
/// observer. The supplied stage must satisfy [`SegmentStage`]'s ownership contract.
#[must_use]
pub struct ObservedSegmentStage<S, O> {
    stage: S,
    observer: O,
}

impl<S, O> ObservedSegmentStage<S, O> {
    /// Installs observation before the stage is consumed by the writer.
    pub const fn new(stage: S, observer: O) -> Self {
        Self { stage, observer }
    }
}

impl<S: SegmentStage, O: SegmentStageObserver> Write for ObservedSegmentStage<S, O> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let allowed = self.observer.before_write(bytes.len())?;
        let prefix = bytes.get(..allowed).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "observer write limit exceeds input",
            )
        })?;
        let written = self.stage.write(prefix)?;
        if written > prefix.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stage exceeded observed write limit",
            ));
        }
        self.observer.after_write(written).map_err(|source| {
            // Interrupted means no write happened to Write::write_all. The
            // observer runs after real effects, so preserve the cause without
            // allowing a caller to silently repeat those bytes.
            if source.kind() == io::ErrorKind::Interrupted {
                io::Error::other(source)
            } else {
                source
            }
        })?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.observer
            .durability(SegmentStageDurabilityEvent::BeforeFlush)?;
        self.stage.flush()?;
        self.observer
            .durability(SegmentStageDurabilityEvent::AfterFlush)
    }
}

impl<S: SegmentStage, O: SegmentStageObserver> SegmentStage for ObservedSegmentStage<S, O> {
    fn synchronize(&mut self) -> io::Result<()> {
        self.observer
            .durability(SegmentStageDurabilityEvent::BeforeSynchronize)?;
        self.stage.synchronize()?;
        self.observer
            .durability(SegmentStageDurabilityEvent::AfterSynchronize)
    }
}

impl<S: SegmentStage, O: SegmentStageObserver> SealedSegment<ObservedSegmentStage<S, O>> {
    /// Removes observation while keeping the sealed stage inaccessible to callers.
    ///
    /// Drops only the observer. The original stage, exact metadata and completed
    /// durability evidence stay together; no user callback receives the stage.
    /// Filesystem publication still checks actual bytes and publisher authority.
    pub fn without_observer(self) -> SealedSegment<S> {
        let (wrapped, count, length, digest) = self.into_parts();
        let ObservedSegmentStage { stage, observer } = wrapped;
        drop(observer);
        SealedSegment::admitted(stage, count, length, digest)
    }
}
