//! This module owns crash injection around the production segment-stage port.

use std::io::{self, Write};

use keep::SegmentStage;
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};

const HEADER_END: usize = 64;
const HEADER_INTERRUPTION: usize = 32;
const RECORD_END: usize = 209;
const RECORD_INTERRUPTION: usize = 136;
const SEAL_END: usize = 337;
const SEAL_INTERRUPTION: usize = 273;

pub(super) struct CrashSegmentStage<'control, S> {
    inner: S,
    control: &'control mut CrashControl,
    bytes_written: usize,
}

impl<'control, S> CrashSegmentStage<'control, S> {
    pub(super) const fn new(inner: S, control: &'control mut CrashControl) -> Self {
        Self {
            inner,
            control,
            bytes_written: 0,
        }
    }

    pub(super) fn into_inner(self) -> S {
        self.inner
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

impl<S> Write for CrashSegmentStage<'_, S>
where
    S: SegmentStage,
{
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
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
        let allowed = remaining.min(bytes.len());
        let prefix = bytes
            .get(..allowed)
            .ok_or_else(|| io::Error::other("segment write prefix exceeded input"))?;
        let written = self.inner.write(prefix)?;
        self.bytes_written = self
            .bytes_written
            .checked_add(written)
            .ok_or_else(|| io::Error::other("segment write count overflowed"))?;
        if position == Some(DurabilityCrashPosition::During) && self.bytes_written == interruption
            || position == Some(DurabilityCrashPosition::After) && self.bytes_written == end
        {
            self.control.await_process_death()?;
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        let point = self.durability_point(
            DurabilityCrashPoint::FlushSegmentRecordPrefix,
            DurabilityCrashPoint::FlushSealedSegment,
        )?;
        self.control.before(point, DuringTiming::Before)?;
        self.inner.flush()?;
        self.control.after(point, DuringTiming::Before)
    }
}

impl<S> SegmentStage for CrashSegmentStage<'_, S>
where
    S: SegmentStage,
{
    fn synchronize(&mut self) -> io::Result<()> {
        let point = self.durability_point(
            DurabilityCrashPoint::SynchronizeSegmentRecordPrefix,
            DurabilityCrashPoint::SynchronizeSealedSegment,
        )?;
        self.control.before(point, DuringTiming::Before)?;
        self.inner.synchronize()?;
        self.control.after(point, DuringTiming::Before)
    }
}
