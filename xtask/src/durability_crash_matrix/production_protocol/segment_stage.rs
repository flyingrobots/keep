//! This module owns crash injection around the production segment-stage port.

use std::io;

use keep::{SegmentStageDurabilityEvent, SegmentStageObserver};
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};

const HEADER_END: usize = 64;
const HEADER_INTERRUPTION: usize = 32;
const RECORD_END: usize = 209;
const RECORD_INTERRUPTION: usize = 136;
const SEAL_END: usize = 337;
const SEAL_INTERRUPTION: usize = 273;

pub(super) struct CrashSegmentObserver<'control> {
    control: &'control mut CrashControl,
    bytes_written: usize,
}

impl<'control> CrashSegmentObserver<'control> {
    pub(super) const fn new(control: &'control mut CrashControl) -> Self {
        Self {
            control,
            bytes_written: 0,
        }
    }

    fn write_boundary(&self) -> io::Result<(DurabilityCrashPoint, usize, usize)> {
        match self.bytes_written {
            0..HEADER_END => Ok((
                DurabilityCrashPoint::WriteSegmentHeader,
                HEADER_INTERRUPTION,
                HEADER_END,
            )),
            HEADER_END..RECORD_END => Ok((
                DurabilityCrashPoint::AppendSegmentRecord,
                RECORD_INTERRUPTION,
                RECORD_END,
            )),
            RECORD_END..SEAL_END => Ok((
                DurabilityCrashPoint::AppendSegmentSeal,
                SEAL_INTERRUPTION,
                SEAL_END,
            )),
            _ => Err(io::Error::other(
                "segment stage wrote beyond the canonical crash fixture",
            )),
        }
    }

    fn durability_point(
        &self,
        prefix: DurabilityCrashPoint,
        sealed: DurabilityCrashPoint,
    ) -> io::Result<DurabilityCrashPoint> {
        match self.bytes_written {
            RECORD_END => Ok(prefix),
            SEAL_END => Ok(sealed),
            _ => Err(io::Error::other(
                "segment durability operation occurred at an unknown length",
            )),
        }
    }
}

impl SegmentStageObserver for CrashSegmentObserver<'_> {
    fn before_write(&mut self, requested: usize) -> io::Result<usize> {
        let (point, interruption, end) = self.write_boundary()?;
        let position = self.control.position(point);
        if position == Some(DurabilityCrashPosition::Before) {
            self.control.await_process_death()?;
        }
        let limit = if position == Some(DurabilityCrashPosition::During) {
            interruption
        } else {
            end
        };
        let remaining = limit
            .checked_sub(self.bytes_written)
            .ok_or_else(|| io::Error::other("segment crash boundary moved backward"))?;
        Ok(remaining.min(requested))
    }

    fn after_write(&mut self, written: usize) -> io::Result<()> {
        let (point, interruption, end) = self.write_boundary()?;
        let position = self.control.position(point);
        self.bytes_written = self
            .bytes_written
            .checked_add(written)
            .ok_or_else(|| io::Error::other("segment write count overflowed"))?;
        if position == Some(DurabilityCrashPosition::During) && self.bytes_written == interruption
            || position == Some(DurabilityCrashPosition::After) && self.bytes_written == end
        {
            self.control.await_process_death()?;
        }
        Ok(())
    }

    fn durability(&mut self, event: SegmentStageDurabilityEvent) -> io::Result<()> {
        let point = match event {
            SegmentStageDurabilityEvent::BeforeFlush | SegmentStageDurabilityEvent::AfterFlush => {
                self.durability_point(
                    DurabilityCrashPoint::FlushSegmentRecordPrefix,
                    DurabilityCrashPoint::FlushSealedSegment,
                )?
            }
            SegmentStageDurabilityEvent::BeforeSynchronize
            | SegmentStageDurabilityEvent::AfterSynchronize => self.durability_point(
                DurabilityCrashPoint::SynchronizeSegmentRecordPrefix,
                DurabilityCrashPoint::SynchronizeSealedSegment,
            )?,
        };
        match event {
            SegmentStageDurabilityEvent::BeforeFlush
            | SegmentStageDurabilityEvent::BeforeSynchronize => {
                self.control.before(point, DuringTiming::Before)
            }
            SegmentStageDurabilityEvent::AfterFlush
            | SegmentStageDurabilityEvent::AfterSynchronize => {
                self.control.after(point, DuringTiming::Before)
            }
        }
    }
}
